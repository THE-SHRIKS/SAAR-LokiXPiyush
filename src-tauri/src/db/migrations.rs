use rusqlite::{Connection, Result, Transaction};

pub const CURRENT_SCHEMA_VERSION: i32 = 4;

pub struct Migrator;

impl Migrator {
    /// Initializes SQLite PRAGMAs and applies schema migrations in strict order.
    pub fn migrate(conn: &mut Connection) -> Result<()> {
        // Enforce foreign key constraints and WAL crash resilience
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        // Create migrations tracking table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
            [],
        )?;

        let current_version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations;",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            let tx = conn.transaction()?;
            Self::apply_migration_v1(&tx)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (1, datetime('now'));",
                [],
            )?;
            tx.commit()?;
        }

        if current_version < 2 {
            let tx = conn.transaction()?;
            Self::apply_migration_v2(&tx)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (2, datetime('now'));",
                [],
            )?;
            tx.commit()?;
        }

        if current_version < 3 {
            let tx = conn.transaction()?;
            Self::apply_migration_v3(&tx)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (3, datetime('now'));",
                [],
            )?;
            tx.commit()?;
        }

        if current_version < 4 {
            let tx = conn.transaction()?;
            Self::apply_migration_v4(&tx)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (4, datetime('now'));",
                [],
            )?;
            tx.commit()?;
        }

        Ok(())
    }

    fn apply_migration_v4(tx: &Transaction) -> Result<()> {
        // 12. Report Exports Audit Trail
        tx.execute(
            "CREATE TABLE IF NOT EXISTS report_exports (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                period TEXT NOT NULL,
                currencies TEXT NOT NULL,
                export_format TEXT NOT NULL,
                file_path TEXT NOT NULL,
                record_snapshot_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_exports_ws ON report_exports(workspace_id);",
            [],
        )?;

        Ok(())
    }

    fn apply_migration_v3(tx: &Transaction) -> Result<()> {
        // 10. Local Full-Text Search (FTS5) Virtual Index
        tx.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
                entity_type,
                entity_id,
                workspace_id UNINDEXED,
                title,
                content
            );",
            [],
        )?;

        // 11. Model-Assisted Classification Suggestions
        tx.execute(
            "CREATE TABLE IF NOT EXISTS transaction_categories (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                transaction_id TEXT NOT NULL,
                category TEXT NOT NULL,
                confidence_score REAL NOT NULL,
                rationale TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'PROPOSED',
                created_at TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
            );",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_cat_tx ON transaction_categories(transaction_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_cat_ws ON transaction_categories(workspace_id);",
            [],
        )?;

        Ok(())
    }

    fn apply_migration_v2(tx: &Transaction) -> Result<()> {
        // 8. Invoice Payment Links (Reconciliation engine)
        tx.execute(
            "CREATE TABLE IF NOT EXISTS invoice_payment_links (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                invoice_id TEXT NOT NULL,
                transaction_id TEXT NOT NULL,
                matched_amount_minor INTEGER NOT NULL,
                confidence_score REAL NOT NULL,
                match_type TEXT NOT NULL,
                explanation TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'PROPOSED',
                created_at TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
                FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // 9. Reconciliation Anomalies & Duplicate Candidates
        tx.execute(
            "CREATE TABLE IF NOT EXISTS reconciliation_anomalies (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                anomaly_type TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // Performance indexes for reconciliation
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_ws ON invoice_payment_links(workspace_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_inv ON invoice_payment_links(invoice_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_links_tx ON invoice_payment_links(transaction_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_anomalies_ws ON reconciliation_anomalies(workspace_id);",
            [],
        )?;

        Ok(())
    }

    fn apply_migration_v1(tx: &Transaction) -> Result<()> {
        // 1. Workspaces
        tx.execute(
            "CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                base_currency TEXT NOT NULL DEFAULT 'USD',
                salt BLOB NOT NULL,
                verification_token BLOB NOT NULL,
                schema_version INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );",
            [],
        )?;

        // 2. Documents
        tx.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size_bytes INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                doc_type TEXT NOT NULL DEFAULT 'UNKNOWN',
                page_count INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'STORED',
                imported_at TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // 3. Document Fields
        tx.execute(
            "CREATE TABLE IF NOT EXISTS document_fields (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                field_key TEXT NOT NULL,
                original_text TEXT NOT NULL,
                normalized_value TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                page_num INTEGER NOT NULL DEFAULT 1,
                bounding_box_json TEXT,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // 4. Invoices
        tx.execute(
            "CREATE TABLE IF NOT EXISTS invoices (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                vendor_name TEXT NOT NULL,
                invoice_number TEXT,
                invoice_date TEXT,
                due_date TEXT,
                currency TEXT NOT NULL DEFAULT 'USD',
                total_minor INTEGER NOT NULL,
                tax_minor INTEGER NOT NULL DEFAULT 0,
                review_state TEXT NOT NULL DEFAULT 'UNREVIEWED',
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // 5. Line Items
        tx.execute(
            "CREATE TABLE IF NOT EXISTS line_items (
                id TEXT PRIMARY KEY,
                invoice_id TEXT NOT NULL,
                description TEXT NOT NULL,
                amount_minor INTEGER NOT NULL,
                FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // 6. Transactions
        tx.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                source_doc_id TEXT,
                posted_date TEXT NOT NULL,
                description TEXT NOT NULL,
                amount_minor INTEGER NOT NULL,
                currency TEXT NOT NULL DEFAULT 'USD',
                external_ref TEXT,
                matched_invoice_id TEXT,
                review_state TEXT NOT NULL DEFAULT 'UNREVIEWED',
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                FOREIGN KEY (source_doc_id) REFERENCES documents(id) ON DELETE SET NULL,
                FOREIGN KEY (matched_invoice_id) REFERENCES invoices(id) ON DELETE SET NULL
            );",
            [],
        )?;

        // 7. Audit Log (Immutable append-only)
        tx.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                details_json TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );",
            [],
        )?;

        // Performance & integrity indexes
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_docs_ws ON documents(workspace_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tx_ws_date ON transactions(workspace_id, posted_date);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_invoices_doc ON invoices(document_id);",
            [],
        )?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_ws_time ON audit_log(workspace_id, timestamp);",
            [],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_migration_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();

        // First run: apply migration v1
        Migrator::migrate(&mut conn).expect("Initial migration should succeed");

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations;", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        // Second run: must be idempotent and succeed without error
        Migrator::migrate(&mut conn).expect("Subsequent migration should be idempotent");
    }

    #[test]
    fn test_foreign_key_constraints_enforced() {
        let mut conn = Connection::open_in_memory().unwrap();
        Migrator::migrate(&mut conn).unwrap();

        // Attempt to insert document referencing non-existent workspace must fail
        let res = conn.execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, imported_at)
             VALUES ('doc-1', 'nonexistent-ws', 'test.pdf', 1024, 'abc123sha', datetime('now'));",
            [],
        );

        assert!(
            res.is_err(),
            "Foreign key constraint failure must reject invalid workspace reference"
        );
    }

    #[test]
    fn test_fts5_virtual_table_support() {
        let conn = Connection::open_in_memory().unwrap();
        let res = conn.execute(
            "CREATE VIRTUAL TABLE test_fts USING fts5(title, content);",
            [],
        );
        assert!(
            res.is_ok(),
            "FTS5 virtual table must be supported: {:?}",
            res
        );
    }
}
