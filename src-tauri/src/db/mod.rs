pub mod migrations;

use migrations::Migrator;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens or creates the SQLite database at the specified path and runs migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        Migrator::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Creates an in-memory database instance (primarily for automated integration tests).
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Migrator::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Access inner connection for transaction blocks.
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Access inner connection for read queries.
    pub fn conn_ref(&self) -> &Connection {
        &self.conn
    }

    /// Appends an immutable record to the audit trail.
    pub fn record_audit_event(
        &self,
        workspace_id: &str,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        details_json: &str,
    ) -> Result<()> {
        let event_id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO audit_log (id, workspace_id, entity_type, entity_id, action, details_json, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'));",
            params![event_id, workspace_id, entity_type, entity_id, action, details_json],
        )?;
        Ok(())
    }

    /// Registers a new workspace record with Argon2id salt and verification token.
    pub fn create_workspace(
        &self,
        id: &str,
        display_name: &str,
        base_currency: &str,
        salt: &[u8],
        verification_token: &[u8],
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO workspaces (id, display_name, base_currency, salt, verification_token, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'));",
            params![id, display_name, base_currency, salt, verification_token],
        )?;

        self.record_audit_event(id, "workspace", id, "CREATE", &format!("{{\"display_name\":\"{}\"}}", display_name))?;
        Ok(())
    }

    /// Retrieves workspace salt and verification token by workspace ID.
    pub fn get_workspace_auth_params(&self, workspace_id: &str) -> Result<(Vec<u8>, [u8; 32])> {
        let mut stmt = self.conn.prepare(
            "SELECT salt, verification_token FROM workspaces WHERE id = ?1 LIMIT 1;"
        )?;

        let mut rows = stmt.query(params![workspace_id])?;
        if let Some(row) = rows.next()? {
            let salt: Vec<u8> = row.get(0)?;
            let token_vec: Vec<u8> = row.get(1)?;
            let mut token = [0u8; 32];
            token.copy_from_slice(&token_vec[..32]);
            Ok((salt, token))
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_rollback_on_error() {
        let mut db = Database::open_in_memory().unwrap();
        let salt = vec![1u8; 16];
        let token = [2u8; 32];

        // 1. Create valid workspace
        db.create_workspace("ws-1", "Test Workspace", "USD", &salt, &token).unwrap();

        // 2. Perform transaction that fails mid-way
        let conn = db.connection_mut();
        let tx_result: Result<()> = (|| {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, imported_at)
                 VALUES ('doc-valid', 'ws-1', 'valid.pdf', 100, 'hash1', datetime('now'));",
                [],
            )?;

            // This next statement will violate FK and cause an error
            tx.execute(
                "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, imported_at)
                 VALUES ('doc-invalid', 'nonexistent-workspace', 'invalid.pdf', 200, 'hash2', datetime('now'));",
                [],
            )?;

            tx.commit()?;
            Ok(())
        })();

        assert!(tx_result.is_err(), "Transaction must fail on error");

        // Verify that 'doc-valid' was rolled back and does not exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents WHERE id = 'doc-valid';", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "Failed transaction must roll back all changes completely");
    }

    #[test]
    fn test_audit_log_records_creation() {
        let db = Database::open_in_memory().unwrap();
        let salt = vec![0u8; 16];
        let token = [1u8; 32];

        db.create_workspace("ws-audit", "Audit Test WS", "USD", &salt, &token).unwrap();

        let count: i64 = db.conn
            .query_row("SELECT COUNT(*) FROM audit_log WHERE workspace_id = 'ws-audit';", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "Audit log must contain creation record");
    }
}
