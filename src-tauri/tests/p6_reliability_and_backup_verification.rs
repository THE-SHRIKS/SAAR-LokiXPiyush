use std::fs;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

use saar_lib::db::Database;
use saar_lib::export::ExportService;
use saar_lib::security::{BackupError, BackupService, BlobStore, KeyDeriver, LogScrubber};

fn setup_test_workspace(
    temp_dir: &PathBuf,
) -> (Database, BlobStore, String, PathBuf, PathBuf, [u8; 32]) {
    let db_path = temp_dir.join("test_opendeck.db");
    let blob_dir = temp_dir.join("blobs");
    fs::create_dir_all(&blob_dir).expect("Create blob dir");

    let db = Database::open(&db_path).expect("Open database");
    let blob_store = BlobStore::new(&blob_dir).expect("Open blob store");

    let ws_id = Uuid::new_v4().to_string();
    let salt = KeyDeriver::generate_salt();
    let master_key = KeyDeriver::derive_master_key("P6SecurePassphrase!", &salt).expect("KDF");
    let token = KeyDeriver::compute_verification_token(&master_key);
    db.create_workspace(&ws_id, "THE SHRIKS Phase 6 Corp", "USD", &salt, &token)
        .expect("Create workspace");

    let master_key_bytes = *master_key;
    (db, blob_store, ws_id, db_path, blob_dir, master_key_bytes)
}

#[test]
fn test_p6_01_source_traceable_csv_and_pdf_exports() {
    let temp_dir = std::env::temp_dir().join(format!("opendeck_p6_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Create temp dir");

    let (mut db, blob_store, ws_id, _db_path, _blob_dir, master_key) =
        setup_test_workspace(&temp_dir);

    // 1. Populate sample transactions and invoice links
    let doc_id = Uuid::new_v4().to_string();
    let doc_bytes = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n";
    let stored_blob = blob_store
        .store_blob(&master_key, doc_bytes)
        .expect("Store blob");

    db.conn_ref()
        .execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, status, page_count, imported_at)
             VALUES (?1, ?2, 'vendor_invoice_101.pdf', ?3, ?4, 'INVOICE', 'PARSED', 1, datetime('now'))",
            rusqlite::params![
                doc_id,
                ws_id,
                doc_bytes.len() as i64,
                stored_blob.sha256_hex,
            ],
        )
        .expect("Insert document");

    let inv_id = Uuid::new_v4().to_string();
    db.conn_ref()
        .execute(
            "INSERT INTO invoices (id, document_id, vendor_name, invoice_number, invoice_date, total_minor, currency, review_state)
             VALUES (?1, ?2, 'Cloud Infra Ltd', 'INV-2026-8899', '2026-09-15', 125000, 'USD', 'PARSED')",
            rusqlite::params![inv_id, doc_id],
        )
        .expect("Insert invoice");

    let tx_id_1 = Uuid::new_v4().to_string();
    let tx_id_2 = Uuid::new_v4().to_string();
    let tx_id_3 = Uuid::new_v4().to_string();

    db.conn_ref().execute(
        "INSERT INTO transactions (id, workspace_id, posted_date, description, amount_minor, currency, source_doc_id, review_state)
         VALUES 
         (?1, ?4, '2026-09-16', 'Cloud Infra Ltd Hosting', -125000, 'USD', ?5, 'MATCHED'),
         (?2, ?4, '2026-09-20', 'Client Retainer Payment', 350000, 'USD', ?5, 'UNLINKED'),
         (?3, ?4, '2026-09-22', 'Office Supplies Depot', -4500, 'USD', ?5, 'UNLINKED')",
        rusqlite::params![tx_id_1, tx_id_2, tx_id_3, ws_id, doc_id],
    ).expect("Insert transactions");

    let link_id = Uuid::new_v4().to_string();
    db.conn_ref().execute(
        "INSERT INTO invoice_payment_links (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 125000, 0.98, 'EXACT_AMOUNT_MATCH', 'Exact amount and date match', 'APPROVED', datetime('now'))",
        rusqlite::params![link_id, ws_id, inv_id, tx_id_1],
    ).expect("Insert link");

    // 2. Test PDF Export
    let pdf_dest = temp_dir.join("reports").join("reconciliation_audit.pdf");
    let pdf_result =
        ExportService::export_reconciliation(db.connection_mut(), &ws_id, "pdf", &pdf_dest)
            .expect("Generate PDF export");

    assert_eq!(pdf_result.export_format, "PDF");
    assert_eq!(pdf_result.record_count, 3);
    assert!(!pdf_result.record_snapshot_hash.is_empty());
    assert!(pdf_dest.exists());
    assert!(pdf_result.file_size_bytes > 500);

    // 3. Verify PDF validity with external Linux tools: pdfinfo & pdftotext
    let pdfinfo_output = Command::new("pdfinfo")
        .arg(&pdf_dest)
        .output()
        .expect("Execute pdfinfo");
    assert!(
        pdfinfo_output.status.success(),
        "pdfinfo failed: {}",
        String::from_utf8_lossy(&pdfinfo_output.stderr)
    );

    let pdftotext_output = Command::new("pdftotext")
        .arg(&pdf_dest)
        .arg("-")
        .output()
        .expect("Execute pdftotext");
    assert!(pdftotext_output.status.success());
    let extracted_text = String::from_utf8_lossy(&pdftotext_output.stdout);

    // Verify key fields in extracted PDF text
    assert!(extracted_text.contains("SAAR FINANCIAL INTELLIGENCE"));
    assert!(extracted_text.contains("THE SHRIKS Phase 6 Corp"));
    assert!(extracted_text.contains("INV-2026-8899"));
    assert!(extracted_text.contains("USD 3500.00")); // Inflow
    assert!(extracted_text.contains("USD 1295.00")); // Outflow (1250.00 + 45.00)
    assert!(extracted_text.contains("APPROVED"));

    // 4. Test CSV Export
    let csv_dest = temp_dir.join("reports").join("reconciliation_audit.csv");
    let csv_result =
        ExportService::export_reconciliation(db.connection_mut(), &ws_id, "csv", &csv_dest)
            .expect("Generate CSV export");

    assert_eq!(csv_result.export_format, "CSV");
    assert_eq!(csv_result.record_count, 3);
    assert!(csv_dest.exists());

    let csv_content = fs::read_to_string(&csv_dest).expect("Read CSV");
    assert!(csv_content.contains("date,description,amount_minor,amount_formatted,currency,match_status,invoice_number,provenance_id"));
    assert!(csv_content.contains("Cloud Infra Ltd Hosting"));
    assert!(csv_content.contains("-125000"));
    assert!(csv_content.contains("-1250.00"));
    assert!(csv_content.contains("APPROVED"));
    assert!(csv_content.contains("INV-2026-8899"));

    // 5. Verify database recorded export in report_exports and audit_log
    let exports_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM report_exports WHERE workspace_id = ?1",
            [&ws_id],
            |row| row.get(0),
        )
        .expect("Query export count");
    assert_eq!(exports_count, 2);

    let audit_count: i64 = db.conn_ref().query_row(
        "SELECT COUNT(*) FROM audit_log WHERE workspace_id = ?1 AND action = 'EXPORT_GENERATED'",
        [&ws_id],
        |row| row.get(0),
    ).expect("Query audit count");
    assert_eq!(audit_count, 2);

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_p6_02_encrypted_backup_wrong_passphrase_and_restore() {
    let base_temp = std::env::temp_dir().join(format!("opendeck_backup_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&base_temp).expect("Create base temp");

    let source_dir = base_temp.join("source_profile");
    let (db, blob_store, ws_id, db_path, blob_dir, master_key) = setup_test_workspace(&source_dir);

    // 1. Create realistic blobs and database records in source workspace
    let sample_invoice_bytes = b"%PDF-1.4\n% Real Financial Invoice Payload\n2500.00 USD\n";
    let stored_blob = blob_store
        .store_blob(&master_key, sample_invoice_bytes)
        .expect("Store blob");

    let doc_id = Uuid::new_v4().to_string();
    db.conn_ref().execute(
        "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, status, page_count, imported_at)
         VALUES (?1, ?2, 'audited_invoice.pdf', ?3, ?4, 'INVOICE', 'PARSED', 1, datetime('now'))",
        rusqlite::params![
            doc_id,
            ws_id,
            sample_invoice_bytes.len() as i64,
            stored_blob.sha256_hex,
        ],
    ).expect("Insert doc");

    let tx_id = Uuid::new_v4().to_string();
    db.conn_ref().execute(
        "INSERT INTO transactions (id, workspace_id, posted_date, description, amount_minor, currency, source_doc_id, review_state)
         VALUES (?1, ?2, '2026-10-01', 'Audited Enterprise Wire', 250000, 'USD', ?3, 'UNLINKED')",
        rusqlite::params![tx_id, ws_id, doc_id],
    ).expect("Insert tx");

    // Checkpoint SQLite so all pages are in db_path
    let _ = db
        .conn_ref()
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

    // 2. Create encrypted .opendeck backup
    let archive_path = base_temp.join("backups").join("vault_2026.opendeck");
    let correct_passphrase = "MasterFinancialBackupKey2026!#$";

    let manifest = BackupService::create_backup(
        &db_path,
        &blob_dir,
        &ws_id,
        "THE SHRIKS Phase 6 Corp",
        &archive_path,
        correct_passphrase,
    )
    .expect("Create backup");

    assert!(archive_path.exists());
    assert_eq!(manifest.blob_count, 1);
    assert!(!manifest.db_sha256.is_empty());
    assert!(manifest.total_raw_bytes > 0);

    // Verify magic header
    let archive_raw = fs::read(&archive_path).expect("Read archive");
    assert_eq!(&archive_raw[0..8], b"SAARSAFE");
    assert_eq!(archive_raw[8..10], 1u16.to_be_bytes());

    // 3. Test wrong passphrase rejection
    let restore_target_dir = base_temp.join("restored_profile");
    let target_db = restore_target_dir.join("opendeck_workspace.db");
    let target_blobs = restore_target_dir.join("blobs");

    let wrong_result = BackupService::restore_backup(
        &archive_path,
        &target_db,
        &target_blobs,
        "CompletelyWrongPassphrase999!",
    );

    match wrong_result {
        Err(BackupError::AuthenticationFailed) => {
            // Expected: AES-256-GCM authentication tag rejected decrypt immediately
        }
        other => panic!("Expected AuthenticationFailed error, got: {:?}", other),
    }

    // Verify target directory was NOT created or contaminated
    assert!(
        !target_db.exists(),
        "Target DB must not exist after failed auth"
    );

    // 4. Test tamper detection (bit flip in ciphertext)
    let mut tampered_raw = archive_raw.clone();
    let last_idx = tampered_raw.len() - 5;
    tampered_raw[last_idx] ^= 0xFF; // Flip bits in ciphertext
    let tampered_path = base_temp.join("tampered.opendeck");
    fs::write(&tampered_path, &tampered_raw).expect("Write tampered archive");

    let tampered_result = BackupService::restore_backup(
        &tampered_path,
        &target_db,
        &target_blobs,
        correct_passphrase,
    );
    match tampered_result {
        Err(BackupError::AuthenticationFailed) => {
            // Tamper detected!
        }
        other => panic!(
            "Expected AuthenticationFailed on tampered ciphertext, got: {:?}",
            other
        ),
    }

    // 5. Test successful restore into fresh target profile
    let restored_manifest =
        BackupService::restore_backup(&archive_path, &target_db, &target_blobs, correct_passphrase)
            .expect("Restore backup with correct key");

    assert_eq!(restored_manifest.db_sha256, manifest.db_sha256);
    assert_eq!(restored_manifest.blob_count, 1);
    assert!(target_db.exists());

    // 6. Verify restored database integrity and exact rows
    let restored_db = Database::open(&target_db).expect("Open restored database");
    let integrity: String = restored_db
        .conn_ref()
        .query_row("PRAGMA integrity_check;", [], |row| row.get(0))
        .expect("Integrity check");
    assert_eq!(integrity, "ok");

    let restored_tx_desc: String = restored_db
        .conn_ref()
        .query_row(
            "SELECT description FROM transactions WHERE id = ?1",
            [&tx_id],
            |row| row.get(0),
        )
        .expect("Query restored tx");
    assert_eq!(restored_tx_desc, "Audited Enterprise Wire");

    // 7. Verify restored blob store can read and decrypt the original blob
    let restored_blob_store = BlobStore::new(&target_blobs).expect("Open restored blob store");
    let decrypted_bytes = restored_blob_store
        .read_blob(&master_key, &stored_blob.sha256_hex)
        .expect("Retrieve restored blob");
    assert_eq!(decrypted_bytes.as_slice(), sample_invoice_bytes);

    let _ = fs::remove_dir_all(base_temp);
}

#[test]
fn test_p6_03_log_scrubber_and_system_recovery() {
    // 1. Log Scrubber verification
    let raw_log = "Error in auth: passphrase='SuperSecretPassphrase!' with apiKey=\"AKIA12345678\" \
                   and card 4111-2222-3333-4444 for PAN ABCDE1234F and Aadhaar 9876 5432 1098 \
                   at /home/theshriksloki/OpenDeck/sensitive.db and C:\\Users\\Administrator\\data.db";

    let scrubbed = LogScrubber::scrub(raw_log);

    // Verify secrets redacted
    assert!(!scrubbed.contains("SuperSecretPassphrase!"));
    assert!(!scrubbed.contains("AKIA12345678"));
    assert!(!scrubbed.contains("4111-2222-3333-4444"));
    assert!(!scrubbed.contains("ABCDE1234F"));
    assert!(!scrubbed.contains("9876 5432 1098"));
    assert!(!scrubbed.contains("/home/theshriksloki/"));
    assert!(!scrubbed.contains("C:\\Users\\Administrator\\"));

    // Verify replacement tags
    assert!(scrubbed.contains("passphrase: [REDACTED]"));
    assert!(scrubbed.contains("apiKey: [REDACTED]"));
    assert!(scrubbed.contains("[REDACTED_CC]"));
    assert!(scrubbed.contains("[REDACTED_PAN]"));
    assert!(scrubbed.contains("[REDACTED_AADHAAR]"));
    assert!(scrubbed.contains("/home/[REDACTED_USER]/OpenDeck/sensitive.db"));
    assert!(scrubbed.contains("C:\\Users\\[REDACTED_USER]\\data.db"));

    // 2. Crash Recovery verification
    let temp_dir = std::env::temp_dir().join(format!("opendeck_recovery_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Create temp dir");
    let (db, _blob_store, ws_id, db_path, _blob_dir, _master_key) = setup_test_workspace(&temp_dir);

    // Simulate interrupted in-flight documents
    let doc_in_flight_1 = Uuid::new_v4().to_string();
    let doc_in_flight_2 = Uuid::new_v4().to_string();
    let doc_completed = Uuid::new_v4().to_string();

    db.conn_ref().execute(
        "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, status, page_count, imported_at)
         VALUES 
         (?1, ?4, 'crashed_1.pdf', 100, 'hash1', 'INVOICE', 'PROCESSING', 1, datetime('now')),
         (?2, ?4, 'crashed_2.pdf', 100, 'hash2', 'INVOICE', 'EXTRACTING', 1, datetime('now')),
         (?3, ?4, 'done.pdf', 100, 'hash3', 'INVOICE', 'PARSED', 1, datetime('now'))",
        rusqlite::params![doc_in_flight_1, doc_in_flight_2, doc_completed, ws_id],
    ).expect("Insert docs");

    // Close and simulate application restart by re-opening Database at db_path
    drop(db);
    let restarted_db = Database::open(&db_path).expect("Reopen db");

    let status_1: String = restarted_db
        .conn_ref()
        .query_row(
            "SELECT status FROM documents WHERE id = ?1",
            [&doc_in_flight_1],
            |r| r.get(0),
        )
        .expect("Query status 1");
    let status_2: String = restarted_db
        .conn_ref()
        .query_row(
            "SELECT status FROM documents WHERE id = ?1",
            [&doc_in_flight_2],
            |r| r.get(0),
        )
        .expect("Query status 2");
    let status_3: String = restarted_db
        .conn_ref()
        .query_row(
            "SELECT status FROM documents WHERE id = ?1",
            [&doc_completed],
            |r| r.get(0),
        )
        .expect("Query status 3");

    // Interrupted documents recovered to FAILED; completed documents remain PARSED
    assert_eq!(status_1, "FAILED");
    assert_eq!(status_2, "FAILED");
    assert_eq!(status_3, "PARSED");

    let _ = fs::remove_dir_all(temp_dir);
}
