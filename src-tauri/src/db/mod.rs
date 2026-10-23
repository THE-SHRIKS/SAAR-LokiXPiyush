pub mod chat;
pub mod migrations;

use migrations::Migrator;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub id: String,
    pub display_name: String,
    pub base_currency: String,
    pub created_at: String,
}

impl Database {
    /// Opens or creates the SQLite database at the specified path and runs migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        Migrator::migrate(&mut conn)?;
        let db = Self { conn };
        let _ = db.recover_interrupted_operations();
        Ok(db)
    }

    /// Opens or creates an encrypted (SQLCipher) workspace database with a raw 256-bit key.
    ///
    /// SQLCipher reads the file header with the key, so a wrong key fails at the first read.
    /// The key is applied before any other statement touches the file.
    pub fn open_encrypted<P: AsRef<Path>>(path: P, key: &[u8; 32]) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex::encode(key)))?;
        // Keep temporary tables and sort spill in memory, never as plaintext temp files.
        conn.execute_batch("PRAGMA temp_store = MEMORY;")?;
        // Key check: fails with "file is not a database" when the key does not match.
        conn.query_row("SELECT count(*) FROM sqlite_master;", [], |r| r.get::<_, i64>(0))?;
        Migrator::migrate(&mut conn)?;
        let db = Self { conn };
        let _ = db.recover_interrupted_operations();
        Ok(db)
    }

    /// Creates an in-memory database instance (primarily for automated integration tests).
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Migrator::migrate(&mut conn)?;
        let db = Self { conn };
        let _ = db.recover_interrupted_operations();
        Ok(db)
    }

    /// Recovers from unexpected system termination by marking in-flight operations as failed.
    pub fn recover_interrupted_operations(&self) -> Result<usize> {
        let count = self.conn.execute(
            "UPDATE documents 
             SET status = 'FAILED' 
             WHERE status IN ('PROCESSING', 'EXTRACTING');",
            [],
        )?;
        Ok(count)
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

        self.record_audit_event(
            id,
            "workspace",
            id,
            "CREATE",
            &format!("{{\"display_name\":\"{}\"}}", display_name),
        )?;
        Ok(())
    }

    /// Lists every workspace stored in this database, newest first. Contains no key material.
    pub fn list_workspace_summaries(&self) -> Result<Vec<WorkspaceSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, display_name, base_currency, created_at
             FROM workspaces
             ORDER BY created_at DESC;",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(WorkspaceSummary {
                id: r.get(0)?,
                display_name: r.get(1)?,
                base_currency: r.get(2)?,
                created_at: r.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Returns the display metadata for one workspace.
    pub fn get_workspace_summary(&self, workspace_id: &str) -> Result<WorkspaceSummary> {
        self.conn.query_row(
            "SELECT id, display_name, base_currency, created_at FROM workspaces WHERE id = ?1;",
            params![workspace_id],
            |r| {
                Ok(WorkspaceSummary {
                    id: r.get(0)?,
                    display_name: r.get(1)?,
                    base_currency: r.get(2)?,
                    created_at: r.get(3)?,
                })
            },
        )
    }

    /// Retrieves workspace salt and verification token by workspace ID.
    pub fn get_workspace_auth_params(&self, workspace_id: &str) -> Result<(Vec<u8>, [u8; 32])> {
        let mut stmt = self
            .conn
            .prepare("SELECT salt, verification_token FROM workspaces WHERE id = ?1 LIMIT 1;")?;

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
        db.create_workspace("ws-1", "Test Workspace", "USD", &salt, &token)
            .unwrap();

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
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE id = 'doc-valid';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            count, 0,
            "Failed transaction must roll back all changes completely"
        );
    }

    #[test]
    fn test_audit_log_records_creation() {
        let db = Database::open_in_memory().unwrap();
        let salt = vec![0u8; 16];
        let token = [1u8; 32];

        db.create_workspace("ws-audit", "Audit Test WS", "USD", &salt, &token)
            .unwrap();

        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE workspace_id = 'ws-audit';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Audit log must contain creation record");
    }

    #[test]
    fn test_sqlcipher_encryption_verified() {
        let dir = std::env::temp_dir().join(format!("saar_test_cipher_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("encrypted.db");

        let key = hex::encode([0x42u8; 32]);

        // 1. Create and populate encrypted DB
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key)).unwrap();
            let cipher_ver: String = conn
                .query_row("PRAGMA cipher_version;", [], |r| r.get(0))
                .unwrap();
            assert!(!cipher_ver.is_empty(), "SQLCipher must report cipher_version");

            conn.execute(
                "CREATE TABLE secret_financials (id TEXT PRIMARY KEY, amount_minor INTEGER);",
                [],
            ).unwrap();
            conn.execute(
                "INSERT INTO secret_financials VALUES ('TX-SECRET-999', 987654321);",
                [],
            ).unwrap();
            // Force checkpoint to ensure data is written to disk
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        }

        // 2. Read raw bytes directly from disk
        let raw_bytes = std::fs::read(&db_path).unwrap();
        // A normal SQLite DB starts with "SQLite format 3\0". An encrypted SQLCipher DB MUST NOT!
        let sqlite_magic = b"SQLite format 3\0";
        assert!(
            raw_bytes.len() >= 16,
            "Encrypted database file must have content"
        );
        assert_ne!(
            &raw_bytes[..16],
            sqlite_magic,
            "Encrypted SQLCipher file must NOT contain plaintext SQLite format 3 header!"
        );

        // Raw bytes must not contain the plaintext strings or numbers
        let raw_str = String::from_utf8_lossy(&raw_bytes);
        assert!(
            !raw_str.contains("TX-SECRET-999"),
            "Raw database file must not expose plaintext financial records!"
        );

        // 3. Opening without key must fail
        {
            let conn_unauth = Connection::open(&db_path).unwrap();
            let query_res = conn_unauth.query_row(
                "SELECT count(*) FROM secret_financials;",
                [],
                |r| r.get::<_, i64>(0),
            );
            assert!(
                query_res.is_err(),
                "Reading encrypted DB without passphrase/key must strictly fail"
            );
        }

        // 4. Opening with wrong key must fail
        {
            let wrong_key = hex::encode([0x99u8; 32]);
            let conn_wrong = Connection::open(&db_path).unwrap();
            conn_wrong.execute_batch(&format!("PRAGMA key = \"x'{}'\";", wrong_key)).unwrap();
            let query_res = conn_wrong.query_row(
                "SELECT count(*) FROM secret_financials;",
                [],
                |r| r.get::<_, i64>(0),
            );
            assert!(
                query_res.is_err(),
                "Reading encrypted DB with wrong key must strictly fail"
            );
        }

        // 5. Opening with correct key must succeed
        {
            let conn_correct = Connection::open(&db_path).unwrap();
            conn_correct.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key)).unwrap();
            let amount: i64 = conn_correct
                .query_row(
                    "SELECT amount_minor FROM secret_financials WHERE id = 'TX-SECRET-999';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(amount, 987654321);
        }

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }
}
