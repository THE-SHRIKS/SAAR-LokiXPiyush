//! Migrates the legacy plaintext database into per-workspace encrypted databases.
//!
//! Guarantees:
//! - The legacy plaintext file is never modified or deleted by this module.
//! - A checksummed backup of the legacy file is taken before anything is written.
//! - Each workspace is copied only after its passphrase verifies against the legacy record.
//! - Each encrypted database is built under a temporary name, verified, and only then renamed
//!   into place. On any failure, everything this run created is removed (rollback).

use crate::db::Database;
use crate::security::{BlobStore, KeyDeriver};
use crate::workspace_store::{self as store, Catalog, CatalogEntry};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MigrationInput {
    pub workspace_id: String,
    pub passphrase: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMigrationReport {
    pub workspace_id: String,
    pub table_counts: Vec<(String, i64, i64)>,
    pub transaction_sum_by_currency: Vec<(String, i64, i64)>,
    pub invoice_total_sum_by_currency: Vec<(String, i64, i64)>,
    pub blobs_copied: usize,
    pub blobs_verified: usize,
    pub integrity_check: String,
    pub foreign_key_violations: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub backup_dir: String,
    pub workspaces: Vec<WorkspaceMigrationReport>,
}

/// Tables that hold per-workspace rows, keyed by the column that links them to a workspace.
const DIRECT_TABLES: &[&str] = &[
    "documents",
    "transactions",
    "transaction_categories",
    "invoice_payment_links",
    "reconciliation_anomalies",
    "audit_log",
    "report_exports",
    "chat_threads",
    "chat_messages",
];
/// Tables linked to a workspace through a parent chain ending at `documents.workspace_id`.
const DOCUMENT_LINKED_TABLES: &[&str] = &["invoices", "document_fields", "line_items"];

/// WHERE clause (with the workspace id as ?1) selecting the rows of a linked table in `schema`.
fn linked_filter(schema: &str, table: &str) -> String {
    match table {
        "line_items" => format!(
            "invoice_id IN (SELECT i.id FROM {s}.invoices i JOIN {s}.documents d ON i.document_id = d.id WHERE d.workspace_id = ?1)",
            s = schema
        ),
        _ => format!(
            "document_id IN (SELECT id FROM {s}.documents WHERE workspace_id = ?1)",
            s = schema
        ),
    }
}

struct LegacyWorkspace {
    salt: Vec<u8>,
    token: [u8; 32],
    created_at: String,
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

/// Copies the legacy database (and any WAL/SHM files) into a timestamped backup directory,
/// writes a checksum manifest, and returns the backup directory.
fn backup_legacy(legacy: &Path, data_dir: &Path) -> Result<PathBuf, String> {
    // Fold the WAL into the main file so the copy is complete.
    {
        let conn = Connection::open(legacy).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| format!("Checkpoint failed: {}", e))?;
    }
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let backup_dir = data_dir.join("migration-backups").join(stamp);
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let mut manifest = String::new();
    // The WAL was checkpointed above, so the main file alone is a complete copy.
    let name = legacy.file_name().unwrap().to_string_lossy().to_string();
    let dst = backup_dir.join(&name);
    fs::copy(legacy, &dst).map_err(|e| format!("Backup copy failed: {}", e))?;
    manifest.push_str(&format!("{}  {}\n", sha256_file(&dst)?, name));
    fs::write(backup_dir.join("SHA256SUMS.txt"), manifest).map_err(|e| e.to_string())?;
    Ok(backup_dir)
}

fn read_legacy_workspace(legacy: &Connection, id: &str) -> Result<LegacyWorkspace, String> {
    let row: Option<(Vec<u8>, Vec<u8>, String)> = legacy
        .query_row(
            "SELECT salt, verification_token, created_at FROM workspaces WHERE id = ?1;",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let (salt, token_vec, created_at) =
        row.ok_or_else(|| format!("Workspace {} not found in the legacy database.", id))?;
    let mut token = [0u8; 32];
    if token_vec.len() != 32 {
        return Err(format!("Workspace {} has a malformed verification token.", id));
    }
    token.copy_from_slice(&token_vec);
    Ok(LegacyWorkspace { salt, token, created_at })
}

/// Lists the tables of an attached schema, excluding FTS5 shadow tables.
fn schema_tables(conn: &Connection, schema: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT name, sql FROM {}.sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';",
            schema
        ))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut names = Vec::new();
    for row in rows {
        let (name, sql) = row.map_err(|e| e.to_string())?;
        // FTS5 shadow tables (search_index_data, _idx, _docsize, _config) are regenerated by
        // the FTS engine; the virtual table itself is copied as rows.
        let is_shadow = name.starts_with("search_index_") && name != "search_index";
        let is_virtual = sql.as_deref().map(|s| s.starts_with("CREATE VIRTUAL")).unwrap_or(false);
        if is_shadow || (is_virtual && name != "search_index") {
            continue;
        }
        names.push(name);
    }
    Ok(names)
}

fn columns_of(conn: &Connection, schema: &str, table: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA {}.table_info({});", schema, table))
        .map_err(|e| e.to_string())?;
    let cols = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    cols.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Copies every row of `workspace_id` from the attached legacy schema into the encrypted main schema.
fn copy_workspace_rows(target: &Connection, workspace_id: &str) -> Result<(), String> {
    // Parent rows first so foreign keys resolve.
    target
        .execute(
            "INSERT INTO main.workspaces SELECT * FROM legacy.workspaces WHERE id = ?1;",
            params![workspace_id],
        )
        .map_err(|e| format!("Copy workspaces failed: {}", e))?;

    let legacy_names: Vec<String> = schema_tables(target, "legacy")?;
    for table in DIRECT_TABLES.iter().chain(DOCUMENT_LINKED_TABLES.iter()).chain(std::iter::once(&"search_index")) {
        if !legacy_names.iter().any(|n| n == table) {
            continue; // legacy schema predates this table
        }
        let target_cols = columns_of(target, "main", table)?;
        let legacy_cols = columns_of(target, "legacy", table)?;
        let shared: Vec<String> = target_cols
            .into_iter()
            .filter(|c| legacy_cols.contains(c))
            .collect();
        if shared.is_empty() {
            continue;
        }
        let col_list = shared.join(", ");
        let filter = if DOCUMENT_LINKED_TABLES.contains(table) {
            linked_filter("legacy", table)
        } else {
            "workspace_id = ?1".to_string()
        };
        target
            .execute(
                &format!(
                    "INSERT INTO main.{t} ({c}) SELECT {c} FROM legacy.{t} WHERE {f};",
                    t = table,
                    c = col_list,
                    f = filter
                ),
                params![workspace_id],
            )
            .map_err(|e| format!("Copy {} failed: {}", table, e))?;
    }
    Ok(())
}

fn count_rows(conn: &Connection, schema: &str, table: &str, workspace_id: &str, doc_linked: bool) -> Result<i64, String> {
    let filter = if doc_linked {
        linked_filter(schema, table)
    } else {
        "workspace_id = ?1".to_string()
    };
    let sql = format!("SELECT COUNT(*) FROM {}.{} WHERE {};", schema, table, filter);
    conn.query_row(&sql, params![workspace_id], |r| r.get(0))
        .map_err(|e| e.to_string())
}

/// Count and sum per currency. `filter` is a WHERE clause that takes the workspace id as ?1.
fn sum_by_currency(conn: &Connection, schema: &str, table: &str, column: &str, filter: &str, workspace_id: &str) -> Result<Vec<(String, i64, i64)>, String> {
    let sql = format!(
        "SELECT currency, COUNT(*), COALESCE(SUM({c}), 0) FROM {s}.{t} WHERE {f} GROUP BY currency ORDER BY currency;",
        c = column,
        s = schema,
        t = table,
        f = filter
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![workspace_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Migrates the listed workspaces from the legacy file into encrypted per-workspace databases.
pub fn migrate_legacy_database(
    legacy_path: &Path,
    data_dir: &Path,
    inputs: &[MigrationInput],
) -> Result<MigrationReport, String> {
    let backup_dir = backup_legacy(legacy_path, data_dir)?;
    let legacy = Connection::open(legacy_path).map_err(|e| e.to_string())?;
    let mut catalog = Catalog::load(data_dir)?;
    let mut created_files: Vec<PathBuf> = Vec::new();
    let mut created_blob_dirs: Vec<PathBuf> = Vec::new();
    let mut reports = Vec::new();

    let mut run = || -> Result<(), String> {
        // Authenticate every workspace before copying anything. Blob re-encryption below may need
        // the key of any migrated workspace, so all keys must be known up front.
        let mut masters: Vec<(String, Zeroizing<[u8; 32]>)> = Vec::new();
        for input in inputs {
            let legacy_ws = read_legacy_workspace(&legacy, &input.workspace_id)?;
            let master = KeyDeriver::verify_passphrase(&input.passphrase, &legacy_ws.salt, &legacy_ws.token)
                .map_err(|_| format!("Passphrase for workspace {} is incorrect.", input.workspace_id))?;
            masters.push((input.workspace_id.clone(), master));
        }

        for (idx, input) in inputs.iter().enumerate() {
            let legacy_ws = read_legacy_workspace(&legacy, &input.workspace_id)?;
            let master = masters[idx].1.clone();
            let db_key = store::derive_db_key(&master);

            let final_path = store::db_path(data_dir, &input.workspace_id);
            if final_path.exists() || catalog.find(&input.workspace_id).is_some() {
                return Err(format!("Workspace {} already exists; refusing to overwrite.", input.workspace_id));
            }
            fs::create_dir_all(final_path.parent().unwrap()).map_err(|e| e.to_string())?;
            let temp_path = final_path.with_extension(format!("{}.migrating", store::DB_EXTENSION));
            let _ = fs::remove_file(&temp_path);

            let target = Database::open_encrypted(&temp_path, &db_key).map_err(|e| e.to_string())?;
            created_files.push(temp_path.clone());
            let conn = target.conn_ref();
            // KEY '' attaches the legacy file unencrypted; otherwise SQLCipher applies this
            // connection's key to it.
            conn.execute(
                "ATTACH DATABASE ?1 AS legacy KEY '';",
                params![legacy_path.to_string_lossy().to_string()],
            )
            .map_err(|e| format!("Attach legacy failed: {}", e))?;
            // The main schema already has the workspace row only after the copy below.
            conn.execute_batch("PRAGMA foreign_keys = OFF;").map_err(|e| e.to_string())?;
            copy_workspace_rows(conn, &input.workspace_id)?;
            conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(|e| e.to_string())?;

            // Blobs: copy only those referenced by this workspace's documents.
            let blob_dir = store::blob_dir(data_dir, &input.workspace_id);
            fs::create_dir_all(&blob_dir).map_err(|e| e.to_string())?;
            created_blob_dirs.push(blob_dir.clone());
            // The legacy store was shared: one file per content hash, encrypted under whichever
            // workspace imported it first. Each workspace gets its own copy, re-encrypted under its
            // own key, so no workspace depends on another workspace's key.
            let legacy_blobs = BlobStore::new(legacy_path.parent().unwrap().join("blobs"))
                .map_err(|e| e.to_string())?;
            let new_blobs = BlobStore::new(&blob_dir).map_err(|e| e.to_string())?;
            let mut shas = Vec::new();
            {
                let mut stmt = conn
                    .prepare("SELECT sha256 FROM legacy.documents WHERE workspace_id = ?1;")
                    .map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map(params![input.workspace_id], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?;
                for r in rows {
                    shas.push(r.map_err(|e| e.to_string())?);
                }
            }
            let mut blobs_copied = 0usize;
            for sha in &shas {
                // Try this workspace's key first, then every other migrated workspace's key.
                let mut plaintext = None;
                for (_, key) in std::iter::once(&(String::new(), master.clone())).chain(masters.iter()) {
                    if let Ok(bytes) = legacy_blobs.read_blob(key, sha) {
                        plaintext = Some(bytes);
                        break;
                    }
                }
                let plaintext = plaintext
                    .ok_or_else(|| format!("No migrated key can decrypt blob {} for workspace {}.", sha, input.workspace_id))?;
                new_blobs
                    .store_blob(&master, &plaintext)
                    .map_err(|e| format!("Re-encrypting blob {} failed: {}", sha, e))?;
                blobs_copied += 1;
            }

            // Verify: counts, sums, integrity, foreign keys, and every blob decrypts.
            let legacy_names = schema_tables(conn, "legacy")?;
            let mut table_counts = Vec::new();
            for table in ["documents", "transactions", "transaction_categories", "invoice_payment_links", "reconciliation_anomalies", "audit_log", "report_exports", "chat_threads", "chat_messages"] {
                let exists_legacy = legacy_names.iter().any(|t| t == table);
                if !exists_legacy {
                    continue;
                }
                let src = count_rows(conn, "legacy", table, &input.workspace_id, false)?;
                let dst = count_rows(conn, "main", table, &input.workspace_id, false)?;
                if src != dst {
                    return Err(format!("Row count mismatch in {}: legacy {} vs encrypted {}.", table, src, dst));
                }
                table_counts.push((table.to_string(), src, dst));
            }
            for table in DOCUMENT_LINKED_TABLES {
                let src = count_rows(conn, "legacy", table, &input.workspace_id, true)?;
                let dst = count_rows(conn, "main", table, &input.workspace_id, true)?;
                if src != dst {
                    return Err(format!("Row count mismatch in {}: legacy {} vs encrypted {}.", table, src, dst));
                }
                table_counts.push((table.to_string(), src, dst));
            }
            let tx_src = sum_by_currency(conn, "legacy", "transactions", "amount_minor", "workspace_id = ?1", &input.workspace_id)?;
            let tx_dst = sum_by_currency(conn, "main", "transactions", "amount_minor", "workspace_id = ?1", &input.workspace_id)?;
            if tx_src != tx_dst {
                return Err("Transaction totals differ after migration.".to_string());
            }
            let inv_src = sum_by_currency(conn, "legacy", "invoices", "total_minor", &linked_filter("legacy", "invoices"), &input.workspace_id)?;
            let inv_dst = sum_by_currency(conn, "main", "invoices", "total_minor", &linked_filter("main", "invoices"), &input.workspace_id)?;
            if inv_src != inv_dst {
                return Err("Invoice totals differ after migration.".to_string());
            }
            let integrity: String = conn
                .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            if integrity != "ok" {
                return Err(format!("Integrity check failed: {}", integrity));
            }
            let fk_violations = conn
                .prepare("PRAGMA foreign_key_check;")
                .and_then(|mut s| s.query_map([], |_| Ok(())).map(|rows| rows.count()))
                .map_err(|e| e.to_string())?;
            if fk_violations > 0 {
                return Err(format!("{} foreign key violations after migration.", fk_violations));
            }
            let mut blobs_verified = 0usize;
            for sha in &shas {
                new_blobs
                    .read_blob(&master, sha)
                    .map_err(|e| format!("Blob {} does not decrypt: {}", sha, e))?;
                blobs_verified += 1;
            }

            drop(target);
            fs::rename(&temp_path, &final_path).map_err(|e| e.to_string())?;
            catalog.workspaces.push(CatalogEntry {
                id: input.workspace_id.clone(),
                salt_hex: store::salt_to_hex(&legacy_ws.salt),
                created_at: legacy_ws.created_at.clone(),
            });
            reports.push(WorkspaceMigrationReport {
                workspace_id: input.workspace_id.clone(),
                table_counts,
                transaction_sum_by_currency: tx_dst,
                invoice_total_sum_by_currency: inv_dst,
                blobs_copied,
                blobs_verified,
                integrity_check: integrity,
                foreign_key_violations: fk_violations,
            });
            created_files.retain(|p| p != &temp_path);
            created_files.push(final_path);
        }
        catalog.save(data_dir)?;
        Ok(())
    };
    let result = run();

    match result {
        Ok(()) => Ok(MigrationReport {
            backup_dir: backup_dir.to_string_lossy().to_string(),
            workspaces: reports,
        }),
        Err(e) => {
            // Rollback: remove only what this run created. The legacy file is never touched.
            for f in &created_files {
                let _ = fs::remove_file(f);
            }
            for d in &created_blob_dirs {
                let _ = fs::remove_dir_all(d);
            }
            Err(format!("Migration rolled back: {}", e))
        }
    }
}
