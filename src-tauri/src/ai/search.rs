use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entity_type: String, // "DOCUMENT", "TRANSACTION", "INVOICE"
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
}

pub struct SearchEngine;

impl SearchEngine {
    /// Indexes a document and its extracted text content into SQLite FTS5.
    pub fn index_document(
        conn: &Connection,
        workspace_id: &str,
        document_id: &str,
        file_name: &str,
        content: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO search_index (entity_type, entity_id, workspace_id, title, content)
             VALUES ('DOCUMENT', ?1, ?2, ?3, ?4);",
            params![document_id, workspace_id, file_name, content],
        )?;

        Ok(())
    }

    /// Indexes a ledger transaction into SQLite FTS5.
    pub fn index_transaction(
        conn: &Connection,
        workspace_id: &str,
        transaction_id: &str,
        description: &str,
        external_ref: Option<&str>,
        amount_formatted: &str,
    ) -> Result<()> {
        let content = format!(
            "{} {} {}",
            description,
            external_ref.unwrap_or(""),
            amount_formatted
        );

        conn.execute(
            "INSERT INTO search_index (entity_type, entity_id, workspace_id, title, content)
             VALUES ('TRANSACTION', ?1, ?2, ?3, ?4);",
            params![transaction_id, workspace_id, description, content],
        )?;

        Ok(())
    }

    /// Performs natural language search across the active workspace with BM25 ranking and snippet generation.
    pub fn search(
        conn: &Connection,
        workspace_id: &str,
        raw_query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let clean_query = sanitize_fts_query(raw_query);
        if clean_query.is_empty() {
            return Ok(Vec::new());
        }

        // Use FTS5 match query with snippet() function across all columns (-1)
        let mut stmt = conn.prepare(
            "SELECT entity_type, entity_id, title, snippet(search_index, -1, '<mark>', '</mark>', '...', 16), rank
             FROM search_index
             WHERE search_index MATCH ?1 AND workspace_id = ?2
             ORDER BY rank
             LIMIT ?3;",
        )?;

        let rows = stmt.query_map(params![clean_query, workspace_id, limit as i64], |r| {
            Ok(SearchResult {
                entity_type: r.get(0)?,
                entity_id: r.get(1)?,
                title: r.get(2)?,
                snippet: r.get(3)?,
                rank: r.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }

        Ok(results)
    }
}

/// Escapes special FTS5 operators to prevent syntax errors from raw user input.
fn sanitize_fts_query(query: &str) -> String {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|w| {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.is_empty() {
                String::new()
            } else {
                format!("\"{}*\"", clean)
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    tokens.join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fts5_indexing_and_search() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::Migrator::migrate(&mut Connection::open_in_memory().unwrap()).unwrap();

        // Create FTS table directly for memory test
        conn.execute(
            "CREATE VIRTUAL TABLE search_index USING fts5(entity_type, entity_id, workspace_id UNINDEXED, title, content);",
            [],
        ).unwrap();

        SearchEngine::index_document(
            &conn,
            "ws-1",
            "doc-1",
            "gst_invoice_acme.pdf",
            "ACME Tech Services Pvt Ltd Tax Invoice ACME-2026-0915 Amount 118,000 INR",
        ).unwrap();

        SearchEngine::index_transaction(
            &conn,
            "ws-1",
            "tx-1",
            "CLOUDHOSTING CORP INV-88912",
            Some("INV-88912"),
            "$120.00",
        ).unwrap();

        let res = SearchEngine::search(&conn, "ws-1", "ACME", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].entity_id, "doc-1");
        assert!(res[0].snippet.contains("<mark>ACME</mark>"));

        let res_tx = SearchEngine::search(&conn, "ws-1", "CloudHosting", 10).unwrap();
        assert_eq!(res_tx.len(), 1);
        assert_eq!(res_tx[0].entity_id, "tx-1");
    }
}
