//! Persistent conversation storage. Threads and messages live inside the
//! encrypted workspace database, so chat history survives restarts and is
//! scoped to the workspace that owns it.

use crate::ai::GroundedChatResponse;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThreadRecord {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRecord {
    pub id: String,
    pub thread_id: String,
    pub sender: String,
    pub text: String,
    pub response: Option<GroundedChatResponse>,
    pub created_at: String,
}

pub fn list_threads(conn: &Connection, workspace_id: &str) -> Result<Vec<ChatThreadRecord>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.pinned, t.created_at, t.updated_at,
                (SELECT COUNT(*) FROM chat_messages m WHERE m.thread_id = t.id)
         FROM chat_threads t
         WHERE t.workspace_id = ?1
         ORDER BY t.pinned DESC, t.updated_at DESC;",
    )?;

    let rows = stmt.query_map(params![workspace_id], |r| {
        Ok(ChatThreadRecord {
            id: r.get(0)?,
            title: r.get(1)?,
            pinned: r.get::<_, i64>(2)? != 0,
            created_at: r.get(3)?,
            updated_at: r.get(4)?,
            message_count: r.get(5)?,
        })
    })?;

    rows.collect()
}

pub fn get_thread(
    conn: &Connection,
    workspace_id: &str,
    thread_id: &str,
) -> Result<Option<ChatThreadRecord>> {
    conn.query_row(
        "SELECT t.id, t.title, t.pinned, t.created_at, t.updated_at,
                (SELECT COUNT(*) FROM chat_messages m WHERE m.thread_id = t.id)
         FROM chat_threads t
         WHERE t.id = ?1 AND t.workspace_id = ?2;",
        params![thread_id, workspace_id],
        |r| {
            Ok(ChatThreadRecord {
                id: r.get(0)?,
                title: r.get(1)?,
                pinned: r.get::<_, i64>(2)? != 0,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
                message_count: r.get(5)?,
            })
        },
    )
    .optional()
}

pub fn create_thread(conn: &Connection, workspace_id: &str, id: &str, title: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO chat_threads (id, workspace_id, title, pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, datetime('now'), datetime('now'));",
        params![id, workspace_id, title],
    )?;
    Ok(())
}

pub fn message_count(conn: &Connection, thread_id: &str) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM chat_messages WHERE thread_id = ?1;",
        params![thread_id],
        |r| r.get(0),
    )
}

/// Returns true when the thread exists and belongs to the given workspace.
pub fn thread_belongs_to(conn: &Connection, workspace_id: &str, thread_id: &str) -> Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM chat_threads WHERE id = ?1 AND workspace_id = ?2;",
            params![thread_id, workspace_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn rename_thread(conn: &Connection, thread_id: &str, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE chat_threads SET title = ?2 WHERE id = ?1;",
        params![thread_id, title],
    )?;
    Ok(())
}

pub fn set_pinned(conn: &Connection, thread_id: &str, pinned: bool) -> Result<()> {
    conn.execute(
        "UPDATE chat_threads SET pinned = ?2 WHERE id = ?1;",
        params![thread_id, pinned as i64],
    )?;
    Ok(())
}

pub fn delete_thread(conn: &Connection, thread_id: &str) -> Result<()> {
    conn.execute("DELETE FROM chat_threads WHERE id = ?1;", params![thread_id])?;
    Ok(())
}

pub fn touch_thread(conn: &Connection, thread_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE chat_threads SET updated_at = datetime('now') WHERE id = ?1;",
        params![thread_id],
    )?;
    Ok(())
}

pub fn list_messages(
    conn: &Connection,
    workspace_id: &str,
    thread_id: &str,
) -> Result<Vec<ChatMessageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, thread_id, sender, body, response_json, created_at
         FROM chat_messages
         WHERE workspace_id = ?1 AND thread_id = ?2
         ORDER BY created_at ASC, rowid ASC;",
    )?;

    let rows = stmt.query_map(params![workspace_id, thread_id], |r| {
        let response_json: Option<String> = r.get(4)?;
        // A stored response that no longer deserializes is dropped rather than
        // failing the whole thread load; the message text is still shown.
        let response = response_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<GroundedChatResponse>(raw).ok());
        Ok(ChatMessageRecord {
            id: r.get(0)?,
            thread_id: r.get(1)?,
            sender: r.get(2)?,
            text: r.get(3)?,
            response,
            created_at: r.get(5)?,
        })
    })?;

    rows.collect()
}

pub fn insert_message(
    conn: &Connection,
    workspace_id: &str,
    thread_id: &str,
    id: &str,
    sender: &str,
    body: &str,
    response: Option<&GroundedChatResponse>,
) -> Result<()> {
    let response_json = match response {
        Some(r) => Some(
            serde_json::to_string(r)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ),
        None => None,
    };

    conn.execute(
        "INSERT INTO chat_messages (id, thread_id, workspace_id, sender, body, response_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));",
        params![id, thread_id, workspace_id, sender, body, response_json],
    )?;
    Ok(())
}

/// Derives a short thread title from the first question in a conversation.
pub fn title_from_query(query: &str) -> String {
    let collapsed = query.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut title: String = collapsed.chars().take(48).collect();
    if collapsed.chars().count() > 48 {
        title.push('…');
    }
    if title.is_empty() {
        "New conversation".to_string()
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{GroundedChatResponse, QueryIntent};
    use crate::db::Database;

    fn setup() -> Database {
        let db = Database::open_in_memory().expect("in-memory db");
        db.create_workspace("ws-a", "Workspace A", "USD", &[1u8; 16], &[2u8; 32])
            .expect("workspace a");
        db.create_workspace("ws-b", "Workspace B", "EUR", &[3u8; 16], &[4u8; 32])
            .expect("workspace b");
        db
    }

    #[test]
    fn test_thread_and_messages_round_trip_with_response() {
        let db = setup();
        let conn = db.conn_ref();
        create_thread(conn, "ws-a", "t1", "First question").unwrap();

        let response = GroundedChatResponse {
            answer: "Net savings are $10.00 [tx:abc].".to_string(),
            intent: QueryIntent::GeneralFinancial,
            citations: vec!["tx:abc".to_string()],
            grounded: true,
            verification_note: "Verified".to_string(),
        };
        insert_message(conn, "ws-a", "t1", "m1", "user", "What is my net?", None).unwrap();
        insert_message(conn, "ws-a", "t1", "m2", "assistant", &response.answer, Some(&response))
            .unwrap();

        let messages = list_messages(conn, "ws-a", "t1").unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].sender, "user");
        assert!(messages[0].response.is_none());
        let stored = messages[1].response.as_ref().expect("stored response");
        assert_eq!(stored.citations, vec!["tx:abc".to_string()]);
        assert!(stored.grounded);

        let thread = get_thread(conn, "ws-a", "t1").unwrap().expect("thread");
        assert_eq!(thread.message_count, 2);
    }

    #[test]
    fn test_threads_are_scoped_to_their_workspace() {
        let db = setup();
        let conn = db.conn_ref();
        create_thread(conn, "ws-a", "t1", "Private to A").unwrap();

        assert!(thread_belongs_to(conn, "ws-a", "t1").unwrap());
        assert!(!thread_belongs_to(conn, "ws-b", "t1").unwrap());
        assert!(list_threads(conn, "ws-b").unwrap().is_empty());
        assert!(list_messages(conn, "ws-b", "t1").unwrap().is_empty());
    }

    #[test]
    fn test_rename_pin_and_delete_cascade() {
        let db = setup();
        let conn = db.conn_ref();
        create_thread(conn, "ws-a", "t1", "Old title").unwrap();
        insert_message(conn, "ws-a", "t1", "m1", "user", "hello", None).unwrap();

        rename_thread(conn, "t1", "Renamed").unwrap();
        set_pinned(conn, "t1", true).unwrap();
        let thread = get_thread(conn, "ws-a", "t1").unwrap().unwrap();
        assert_eq!(thread.title, "Renamed");
        assert!(thread.pinned);

        delete_thread(conn, "t1").unwrap();
        assert!(get_thread(conn, "ws-a", "t1").unwrap().is_none());
        assert_eq!(message_count(conn, "t1").unwrap(), 0);
    }

    #[test]
    fn test_insert_rejects_unknown_sender() {
        let db = setup();
        let conn = db.conn_ref();
        create_thread(conn, "ws-a", "t1", "Title").unwrap();
        let err = insert_message(conn, "ws-a", "t1", "m1", "system", "nope", None);
        assert!(err.is_err(), "sender must be user or assistant");
    }

    #[test]
    fn test_title_from_query_is_bounded_and_nonempty() {
        assert_eq!(title_from_query("   "), "New conversation");
        assert_eq!(title_from_query("  short   question "), "short question");
        let long = "x".repeat(80);
        let title = title_from_query(&long);
        assert_eq!(title.chars().count(), 49);
        assert!(title.ends_with('…'));
    }
}
