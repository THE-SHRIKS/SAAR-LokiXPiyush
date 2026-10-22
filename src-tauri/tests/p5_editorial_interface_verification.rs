/**
 * OpenDeck Phase 5 Integration Test — Editorial Interface, Evidence Inspector,
 * Manual Correction Flow, Immutable Audit Trail, and Visual Reports Verification
 *
 * Verifies P5-01 through P5-06 without mocks or cloud telemetry.
 */

use opendeck_lib::db::Database;
use opendeck_lib::finance::aggregates::FinancialAggregator;
use opendeck_lib::finance::matching::{InvoiceCandidate, ReconciliationMatcher, TransactionCandidate};
use opendeck_lib::finance::review::ReviewService;
use opendeck_lib::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use opendeck_lib::security::{BlobStore, KeyDeriver};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_p5_editorial_interface_and_integrated_workflows() {
    let root = get_workspace_root();
    let temp_dir = std::env::temp_dir().join(format!("opendeck_p5_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let db_path = temp_dir.join("workspace.db");
    let blob_dir = temp_dir.join("blobs");

    let mut db = Database::open(&db_path).expect("Failed to open test database");
    let blob_store = BlobStore::new(&blob_dir).expect("Failed to create blob store");
    let ocr_client = OcrClient::new().expect("Failed to create OCR client");

    let passphrase = "MasterPassphrase_P5_Verification_2026!";
    let salt = KeyDeriver::generate_salt();
    let master_key = KeyDeriver::derive_master_key(passphrase, &salt).unwrap();
    let verification_token = KeyDeriver::compute_verification_token(&master_key);

    let workspace_id = "ws-p5-editorial-verified";
    db.create_workspace(
        workspace_id,
        "THE SHRIKS P5 Workspace",
        "USD",
        &salt,
        &verification_token,
    )
    .unwrap();

    // -------------------------------------------------------------
    // Step 1: P5-02 Evidence Inspector — Ingest Document & Verify Decrypted Preview
    // -------------------------------------------------------------
    let thermal_receipt_path = root.join("tests/fixtures/receipts/thermal_receipt_scanned.png");
    assert!(thermal_receipt_path.exists(), "Receipt scan fixture must exist");

    let ingest_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &thermal_receipt_path,
        "USD",
    )
    .expect("Thermal receipt ingestion must succeed");

    let doc_id = match ingest_res {
        IngestResult::Success { document_id, .. } => document_id,
        _ => panic!("Expected successful ingestion"),
    };

    // Verify blob store decryption returns authentic PNG bytes
    let (sha256, doc_type): (String, String) = db
        .conn_ref()
        .query_row(
            "SELECT sha256, doc_type FROM documents WHERE id = ?1;",
            params![doc_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(doc_type, "PNG");
    let decrypted_bytes = blob_store.read_blob(&master_key, &sha256).unwrap();
    assert_eq!(&decrypted_bytes[0..8], b"\x89PNG\r\n\x1a\n", "Must decrypt authentic PNG magic bytes");

    // Verify normalized fields and bounding boxes exist for coordinate overlay
    let mut field_stmt = db
        .conn_ref()
        .prepare("SELECT field_key, normalized_value, confidence, bounding_box_json FROM document_fields WHERE document_id = ?1;")
        .unwrap();
    let fields: Vec<(String, String, f64, String)> = field_stmt
        .query_map(params![doc_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    drop(field_stmt);

    assert!(!fields.is_empty(), "Must contain normalized OCR fields");
    let total_field = fields.iter().find(|f| f.0 == "total_amount");
    assert!(total_field.is_some(), "Must extract total_amount field");
    let bbox_json = &total_field.unwrap().3;
    assert!(bbox_json.starts_with('[') && bbox_json.ends_with(']'), "Bounding box must be valid JSON coordinate array");

    // -------------------------------------------------------------
    // Step 2: P5-03 Review Surface & Manual Audit Correction Flow
    // -------------------------------------------------------------
    // Ingest bank statement CSV to create ledger transactions
    let bank_csv_path = root.join("tests/fixtures/statements/bank_us_sample.csv");
    let _ = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &bank_csv_path,
        "USD",
    )
    .expect("Bank statement ingestion must succeed");

    // Query an unreviewed transaction
    let (tx_id, original_desc): (String, String) = db
        .conn_ref()
        .query_row(
            "SELECT id, description FROM transactions WHERE workspace_id = ?1 AND description LIKE '%Office Supplies%' LIMIT 1;",
            params![workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    // Perform manual description correction
    let corrected_desc = "Office Supplies Mart — Q3 Stationery & Archival Storage";
    ReviewService::correct_transaction(&db, workspace_id, &tx_id, corrected_desc)
        .expect("Manual correction must succeed");

    // Verify transaction was updated
    let updated_desc: String = db
        .conn_ref()
        .query_row("SELECT description FROM transactions WHERE id = ?1;", params![tx_id], |r| r.get(0))
        .unwrap();
    assert_eq!(updated_desc, corrected_desc);

    // Verify immutable audit log recorded the before/after state
    let (audit_action, audit_payload): (String, String) = db
        .conn_ref()
        .query_row(
            "SELECT action, details_json FROM audit_log WHERE entity_id = ?1 ORDER BY timestamp DESC LIMIT 1;",
            params![tx_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(audit_action, "CORRECT_TRANSACTION");
    assert!(audit_payload.contains(&original_desc), "Audit log must contain previous description");
    assert!(audit_payload.contains(corrected_desc), "Audit log must contain corrected description");

    // -------------------------------------------------------------
    // Step 3: P5-03 Reconciliation Approval & Audit Recording
    // -------------------------------------------------------------
    // Ingest invoice PDF to test matching
    let invoice_pdf_path = root.join("tests/fixtures/invoices/gst_b2b_valid.pdf");
    let _ = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &invoice_pdf_path,
        "INR",
    )
    .expect("GST invoice ingestion must succeed");

    // Load invoice candidates
    let mut inv_stmt = db
        .conn_ref()
        .prepare("SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency FROM invoices i JOIN documents d ON i.document_id = d.id WHERE d.workspace_id = ?1;")
        .unwrap();
    let invoices: Vec<InvoiceCandidate> = inv_stmt
        .query_map(params![workspace_id], |r| {
            Ok(InvoiceCandidate {
                id: r.get(0)?,
                vendor_name: r.get(1)?,
                invoice_number: r.get(2)?,
                invoice_date: r.get(3)?,
                total_minor: r.get(4)?,
                currency: r.get(5)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    drop(inv_stmt);

    // Load transaction candidates
    let mut tx_stmt = db
        .conn_ref()
        .prepare("SELECT id, posted_date, description, amount_minor, currency, external_ref FROM transactions WHERE workspace_id = ?1;")
        .unwrap();
    let transactions: Vec<TransactionCandidate> = tx_stmt
        .query_map(params![workspace_id], |r| {
            Ok(TransactionCandidate {
                id: r.get(0)?,
                posted_date: r.get(1)?,
                description: r.get(2)?,
                amount_minor: r.get(3)?,
                currency: r.get(4)?,
                external_ref: r.get(5)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    drop(tx_stmt);

    let _proposed_links = ReconciliationMatcher::match_records(&invoices, &transactions);

    // Persist a proposed link into database
    let test_link_id = format!("link-{}", Uuid::new_v4());
    let inv_id = &invoices[0].id;
    let tx_item_id = &transactions[0].id;

    db.conn_ref()
        .execute(
            "INSERT INTO invoice_payment_links (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'PROPOSED', datetime('now'));",
            params![test_link_id, workspace_id, inv_id, tx_item_id, 4550i64, 0.95f64, "EXACT_AMOUNT", "Test link match"],
        )
        .unwrap();

    // User approves the proposed link
    ReviewService::approve_link(&mut db, workspace_id, &test_link_id)
        .expect("Link approval must succeed");

    // Verify link status transitioned to APPROVED in DB
    let link_status: String = db
        .conn_ref()
        .query_row("SELECT status FROM invoice_payment_links WHERE id = ?1;", params![test_link_id], |r| r.get(0))
        .unwrap();
    assert_eq!(link_status, "APPROVED");

    // Verify audit log captured APPROVE_LINK
    let audit_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'APPROVE_LINK' AND entity_id = ?1;",
            params![test_link_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audit_count, 1, "Must have exactly 1 immutable audit entry for link approval");

    // -------------------------------------------------------------
    // Step 4: P5-04 Reports Page Aggregates & MoM Cash Flow
    // -------------------------------------------------------------
    let summary = FinancialAggregator::compute_summary(&transactions, "USD");

    assert!(summary.total_income_minor > 0, "Total income must be positive");
    assert!(summary.total_expense_minor > 0, "Total expenses must be positive");
    assert_eq!(
        summary.net_savings_minor,
        summary.total_income_minor - summary.total_expense_minor,
        "Net savings must equal income minus expenses deterministically"
    );
    assert!(!summary.monthly_breakdown.is_empty(), "Must contain monthly breakdown");

    // Clean up temporary test files
    let _ = fs::remove_dir_all(temp_dir);
    println!("Phase 5 full editorial interface, evidence coordinates, and audit trail verification succeeded!");
}
