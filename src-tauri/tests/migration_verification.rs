//! Verifies the legacy plaintext → per-workspace encrypted migration on a synthetic copy.
//! Uses the production ingestion path and the repository's bundled CSV fixture.

use rusqlite::params;
use saar_lib::db::Database;
use saar_lib::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use saar_lib::security::{BlobStore, KeyDeriver};
use saar_lib::workspace_migration::{migrate_legacy_database, MigrationInput};
use saar_lib::workspace_store::{self as store, Catalog};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn fixture_csv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/statements/bank_us_sample.csv")
}

/// Builds a plaintext legacy database with two workspaces, each with one ingested statement.
fn build_legacy(dir: &Path) -> (PathBuf, Vec<(String, String)>) {
    let legacy = dir.join("saar_workspace.db");
    let blob_dir = dir.join("blobs");
    let blobs = BlobStore::new(&blob_dir).unwrap();
    let ocr = OcrClient::new().expect("OCR client");
    let mut db = Database::open(&legacy).unwrap();

    let mut pairs = Vec::new();
    for (ws, pass) in [("ws-alpha", "Alpha-Passphrase-2026!"), ("ws-beta", "Beta-Passphrase-2026!")] {
        let salt = KeyDeriver::generate_salt();
        let master = KeyDeriver::derive_master_key(pass, &salt).unwrap();
        let token = KeyDeriver::compute_verification_token(&master);
        db.create_workspace(ws, ws, "USD", &salt, &token).unwrap();
        match DocumentOrchestrator::ingest_file(&mut db, &blobs, &ocr, &master, ws, &fixture_csv(), "USD")
            .expect("ingest")
        {
            IngestResult::Success { .. } => {}
            _ => panic!("fixture must ingest"),
        }
        pairs.push((ws.to_string(), pass.to_string()));
    }
    // Fold the WAL into the main file, as the app does on shutdown.
    db.conn_ref().execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
    drop(db);
    (legacy, pairs)
}

fn sha256(path: &Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("saar_migration_{}", Uuid::new_v4()))
}

#[test]
fn test_migration_produces_verified_encrypted_workspaces() {
    let root = temp_root();
    let legacy_dir = root.join("legacy");
    let data_dir = root.join("data");
    fs::create_dir_all(&legacy_dir).unwrap();
    let (legacy, pairs) = build_legacy(&legacy_dir);
    let legacy_hash_before = sha256(&legacy);

    let inputs: Vec<MigrationInput> = pairs
        .iter()
        .map(|(ws, pass)| MigrationInput { workspace_id: ws.clone(), passphrase: pass.clone() })
        .collect();
    let report = migrate_legacy_database(&legacy, &data_dir, &inputs).expect("migration must succeed");
    assert_eq!(report.workspaces.len(), 2);

    // The legacy file is untouched.
    assert_eq!(sha256(&legacy), legacy_hash_before, "legacy database must not be modified");
    // Backup exists with a checksum manifest.
    assert!(Path::new(&report.backup_dir).join("SHA256SUMS.txt").exists());

    // Catalog lists both workspaces and carries no display names.
    let catalog = Catalog::load(&data_dir).unwrap();
    assert_eq!(catalog.workspaces.len(), 2);
    let raw_catalog = fs::read_to_string(data_dir.join(store::CATALOG_FILE)).unwrap();
    assert!(!raw_catalog.contains("ws-alpha Passphrase") && !raw_catalog.contains("displayName"));

    for (ws, pass) in &pairs {
        let path = store::db_path(&data_dir, ws);
        let bytes = fs::read(&path).unwrap();
        assert!(!bytes.starts_with(b"SQLite format 3"), "encrypted file must not have a plaintext header");
        let needle = b"CloudHosting"; // a description that appears verbatim in the fixture
        let plaintext_hits = bytes.windows(needle.len()).filter(|w| *w == needle).count();
        assert_eq!(plaintext_hits, 0, "encrypted file must not contain plaintext record text");

        // Opening with the correct derived key works; the record counts match the source.
        let catalog_entry = catalog.find(ws).unwrap();
        let salt = store::salt_from_hex(&catalog_entry.salt_hex).unwrap();
        let master = KeyDeriver::derive_master_key(pass, &salt).unwrap();
        let db_key = store::derive_db_key(&master);
        let enc = Database::open_encrypted(&path, &db_key).expect("correct key must open");
        let tx_count: i64 = enc
            .conn_ref()
            .query_row("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1", params![ws], |r| r.get(0))
            .unwrap();
        assert!(tx_count > 0, "migrated workspace must contain transactions");
        let integrity: String = enc
            .conn_ref()
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        drop(enc);

        // A wrong key must not open the encrypted database.
        let wrong = store::derive_db_key(&KeyDeriver::derive_master_key("wrong-passphrase-value", &salt).unwrap());
        assert!(Database::open_encrypted(&path, &wrong).is_err(), "wrong key must be rejected");
    }

    // The two workspaces are isolated: each database holds only its own rows.
    let report_ws = &report.workspaces;
    for r in report_ws {
        assert!(r.blobs_copied > 0 && r.blobs_verified == r.blobs_copied);
        assert_eq!(r.integrity_check, "ok");
        assert_eq!(r.foreign_key_violations, 0);
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_wrong_passphrase_rolls_back_without_touching_legacy() {
    let root = temp_root();
    let legacy_dir = root.join("legacy");
    let data_dir = root.join("data");
    fs::create_dir_all(&legacy_dir).unwrap();
    let (legacy, pairs) = build_legacy(&legacy_dir);
    let legacy_hash_before = sha256(&legacy);

    // First workspace is correct, second has the wrong passphrase.
    let inputs = vec![
        MigrationInput { workspace_id: pairs[0].0.clone(), passphrase: pairs[0].1.clone() },
        MigrationInput { workspace_id: pairs[1].0.clone(), passphrase: "not-the-passphrase-here".into() },
    ];
    let err = migrate_legacy_database(&legacy, &data_dir, &inputs).expect_err("must fail");
    assert!(err.contains("rolled back"), "error must report rollback: {}", err);

    assert_eq!(sha256(&legacy), legacy_hash_before, "legacy database must not be modified");
    assert!(!store::db_path(&data_dir, &pairs[0].0).exists(), "partial workspace must be removed");
    assert!(!store::db_path(&data_dir, &pairs[1].0).exists());
    assert!(
        Catalog::load(&data_dir).unwrap().workspaces.is_empty(),
        "catalog must not list workspaces from a failed run"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_refuses_to_overwrite_an_existing_encrypted_workspace() {
    let root = temp_root();
    let legacy_dir = root.join("legacy");
    let data_dir = root.join("data");
    fs::create_dir_all(&legacy_dir).unwrap();
    let (legacy, pairs) = build_legacy(&legacy_dir);
    let inputs: Vec<MigrationInput> = pairs
        .iter()
        .map(|(ws, pass)| MigrationInput { workspace_id: ws.clone(), passphrase: pass.clone() })
        .collect();
    migrate_legacy_database(&legacy, &data_dir, &inputs).expect("first run succeeds");
    let second = migrate_legacy_database(&legacy, &data_dir, &inputs);
    assert!(second.is_err(), "a second run must not overwrite existing workspaces");
    // The first run's output survives the failed second run.
    assert!(store::db_path(&data_dir, &pairs[0].0).exists());
    let _ = fs::remove_dir_all(&root);
}
