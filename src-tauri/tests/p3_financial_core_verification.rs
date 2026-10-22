use opendeck_lib::db::Database;
use opendeck_lib::finance::aggregates::FinancialAggregator;
use opendeck_lib::finance::anomaly::AnomalyDetector;
use opendeck_lib::finance::matching::{
    InvoiceCandidate, ReconciliationMatcher, TransactionCandidate,
};
use opendeck_lib::finance::review::ReviewService;
use opendeck_lib::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use opendeck_lib::security::{BlobStore, KeyDeriver};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_p3_full_financial_core_and_reconciliation_verification() {
    let test_dir = std::env::temp_dir().join(format!("opendeck_p3_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    let db_path = test_dir.join("test_p3_workspace.db");
    let blob_dir = test_dir.join("blobs");

    let mut db = Database::open(&db_path).expect("Failed to open test database");
    let blob_store = BlobStore::new(&blob_dir).expect("Failed to create blob store");
    let ocr_client = OcrClient::new().expect("Failed to create OCR client");

    let passphrase = "P3FinancialReconciliationPassphrase#2026";
    let salt = KeyDeriver::generate_salt();
    let master_key = KeyDeriver::derive_master_key(passphrase, &salt).unwrap();
    let verification_token = KeyDeriver::compute_verification_token(&master_key);

    let workspace_id = "ws-p3-test";
    db.create_workspace(
        workspace_id,
        "P3 Test Finance Workspace",
        "USD",
        &salt,
        &verification_token,
    )
    .unwrap();

    let root_fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures");

    // -------------------------------------------------------------
    // Step 1: Ingest Invoices (GST B2B PDF, Thermal Receipt PNG)
    // -------------------------------------------------------------
    let gst_pdf_path = root_fixtures.join("invoices/gst_b2b_valid.pdf");
    let gst_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &gst_pdf_path,
        "INR",
    )
    .expect("Failed to ingest gst_b2b_valid.pdf");

    let gst_doc_id = match gst_res {
        IngestResult::Success { document_id, .. } => document_id,
        other => panic!("Expected IngestResult::Success for GST, got {:?}", other),
    };

    let receipt_path = root_fixtures.join("receipts/thermal_receipt_scanned.png");
    let receipt_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &receipt_path,
        "USD",
    )
    .expect("Failed to ingest thermal_receipt_scanned.png");

    let _receipt_doc_id = match receipt_res {
        IngestResult::Success { document_id, .. } => document_id,
        other => panic!(
            "Expected IngestResult::Success for receipt, got {:?}",
            other
        ),
    };

    // Also manually insert a digital invoice for CloudHosting Corp to test exact invoice number matching
    let cloud_doc_id = Uuid::new_v4().to_string();
    let cloud_inv_id = Uuid::new_v4().to_string();
    let now_str = chrono::Utc::now().to_rfc3339();

    db.conn_ref()
        .execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, page_count, status, imported_at)
             VALUES (?1, ?2, 'cloudhosting_inv_88912.pdf', 1024, 'dummy_sha_cloud', 'PDF', 1, 'STORED', ?3);",
            params![cloud_doc_id, workspace_id, now_str],
        )
        .unwrap();

    db.conn_ref()
        .execute(
            "INSERT INTO invoices (id, document_id, vendor_name, invoice_number, invoice_date, due_date, currency, total_minor, tax_minor, review_state)
             VALUES (?1, ?2, 'CloudHosting Corp', 'INV-88912', '2026-09-01', '2026-09-15', 'USD', 12000, 0, 'UNREVIEWED');",
            params![cloud_inv_id, cloud_doc_id],
        )
        .unwrap();

    // -------------------------------------------------------------
    // Step 2: Ingest Bank Statements (US Bank CSV and HDFC Bank CSV)
    // -------------------------------------------------------------
    let us_bank_path = root_fixtures.join("statements/bank_us_sample.csv");
    let us_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &us_bank_path,
        "USD",
    )
    .expect("Failed to ingest bank_us_sample.csv");

    match us_res {
        IngestResult::Success {
            records_extracted, ..
        } => {
            assert!(
                records_extracted > 0,
                "US bank transactions must be ingested"
            );
        }
        other => panic!(
            "Expected IngestResult::Success for US bank CSV, got {:?}",
            other
        ),
    }

    let hdfc_path = root_fixtures.join("statements/bank_hdfc_sample.csv");
    let hdfc_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &hdfc_path,
        "INR",
    )
    .expect("Failed to ingest bank_hdfc_sample.csv");

    match hdfc_res {
        IngestResult::Success {
            records_extracted, ..
        } => {
            assert!(
                records_extracted > 0,
                "HDFC bank transactions must be ingested"
            );
        }
        other => panic!(
            "Expected IngestResult::Success for HDFC bank CSV, got {:?}",
            other
        ),
    }

    // -------------------------------------------------------------
    // Step 3: Verify Deterministic Reconciliation Matching
    // -------------------------------------------------------------
    // Query invoice candidates
    let mut inv_stmt = db
        .conn_ref()
        .prepare(
            "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
             FROM invoices i
             JOIN documents d ON i.document_id = d.id
             WHERE d.workspace_id = ?1;",
        )
        .unwrap();

    let inv_rows = inv_stmt
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
        .unwrap();

    let invoices: Vec<InvoiceCandidate> = inv_rows.map(|r| r.unwrap()).collect();
    drop(inv_stmt);
    assert!(invoices.len() >= 3, "Must have at least 3 invoice records");

    // Query transaction candidates
    let mut tx_stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, posted_date, description, amount_minor, currency, external_ref
             FROM transactions
             WHERE workspace_id = ?1;",
        )
        .unwrap();

    let tx_rows = tx_stmt
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
        .unwrap();

    let transactions: Vec<TransactionCandidate> = tx_rows.map(|r| r.unwrap()).collect();
    drop(tx_stmt);
    assert!(
        transactions.len() >= 10,
        "Must have transaction records from both banks"
    );

    // Match records
    let proposed_links = ReconciliationMatcher::match_records(&invoices, &transactions);
    assert!(
        !proposed_links.is_empty(),
        "ReconciliationMatcher must produce proposed links"
    );

    // Verify CloudHosting match:
    // Invoice 12000 minor ($120.00) matches transaction "CLOUDHOSTING CORP INV-88912" (-12000)
    let cloud_match = proposed_links
        .iter()
        .find(|l| l.invoice_id == cloud_inv_id)
        .expect("CloudHosting Corp invoice must be matched");

    assert_eq!(cloud_match.matched_amount_minor, 12000);
    assert_eq!(cloud_match.match_type, "EXACT");
    assert_eq!(cloud_match.confidence_score, 1.0);
    assert!(cloud_match.explanation.contains("Exact match on amount"));

    // Verify ACME Tech GST invoice match in INR:
    // Invoice 11800000 minor (118,000.00 INR) matches HDFC transaction -11800000
    let gst_inv_id: String = db
        .conn_ref()
        .query_row(
            "SELECT id FROM invoices WHERE document_id = ?1;",
            params![gst_doc_id],
            |r| r.get(0),
        )
        .unwrap();

    let acme_match = proposed_links
        .iter()
        .find(|l| l.invoice_id == gst_inv_id)
        .expect("ACME Tech Services GST invoice must be matched with HDFC statement");

    assert_eq!(acme_match.matched_amount_minor, 11800000);
    assert!(acme_match.confidence_score >= 0.85);

    // -------------------------------------------------------------
    // Step 4: Verify Anomaly Detection
    // -------------------------------------------------------------
    let anomalies = AnomalyDetector::detect_anomalies(&invoices, &transactions);
    // There should be anomalies detected (e.g. large unmatched debit transactions like client payout or transfers)
    assert!(
        !anomalies.is_empty(),
        "AnomalyDetector must report alerts for unlinked high transactions"
    );

    let high_unmatched = anomalies
        .iter()
        .find(|a| a.anomaly_type == "LARGE_UNMATCHED");
    assert!(
        high_unmatched.is_some(),
        "Must flag large unmatched debit as anomaly"
    );

    // -------------------------------------------------------------
    // Step 5: Persist Link & Test Review Inbox Workflow
    // -------------------------------------------------------------
    // Insert the cloud match link into invoice_payment_links
    db.conn_ref()
        .execute(
            "INSERT INTO invoice_payment_links
             (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'PROPOSED', ?9);",
            params![
                cloud_match.id,
                workspace_id,
                cloud_match.invoice_id,
                cloud_match.transaction_id,
                cloud_match.matched_amount_minor,
                cloud_match.confidence_score,
                cloud_match.match_type,
                cloud_match.explanation,
                now_str
            ],
        )
        .unwrap();

    // Verify initial UNREVIEWED / PROPOSED states
    let initial_tx_state: String = db
        .conn_ref()
        .query_row(
            "SELECT review_state FROM transactions WHERE id = ?1;",
            params![cloud_match.transaction_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(initial_tx_state, "UNREVIEWED");

    // User approves the proposed match
    ReviewService::approve_link(&mut db, workspace_id, &cloud_match.id)
        .expect("Approve link must succeed");

    // Verify states transitioned to APPROVED
    let (tx_matched_inv, updated_tx_state): (Option<String>, String) = db
        .conn_ref()
        .query_row(
            "SELECT matched_invoice_id, review_state FROM transactions WHERE id = ?1;",
            params![cloud_match.transaction_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(tx_matched_inv, Some(cloud_inv_id.clone()));
    assert_eq!(updated_tx_state, "APPROVED");

    let updated_inv_state: String = db
        .conn_ref()
        .query_row(
            "SELECT review_state FROM invoices WHERE id = ?1;",
            params![cloud_inv_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(updated_inv_state, "APPROVED");

    // Verify immutable audit log recorded the APPROVE event
    let audit_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE workspace_id = ?1 AND action = 'APPROVE_LINK' AND entity_id = ?2;",
            params![workspace_id, cloud_match.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audit_count, 1, "Audit log must contain APPROVE_LINK event");

    // Test transaction correction workflow
    let test_tx = &transactions[0];
    ReviewService::correct_transaction(
        &db,
        workspace_id,
        &test_tx.id,
        "Corrected Description for Audit",
    )
    .expect("Transaction correction must succeed");

    let corrected_desc: String = db
        .conn_ref()
        .query_row(
            "SELECT description FROM transactions WHERE id = ?1;",
            params![test_tx.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(corrected_desc, "Corrected Description for Audit");

    // Verify correction audit log
    let correction_audit: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE workspace_id = ?1 AND action = 'CORRECT_TRANSACTION' AND entity_id = ?2;",
            params![workspace_id, test_tx.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        correction_audit, 1,
        "Audit log must contain CORRECT_TRANSACTION event"
    );

    // -------------------------------------------------------------
    // Step 6: Verify Monthly Financial Aggregates
    // -------------------------------------------------------------
    // Re-fetch transactions for USD
    let mut usd_tx_stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, posted_date, description, amount_minor, currency, external_ref
             FROM transactions
             WHERE workspace_id = ?1 AND currency = 'USD';",
        )
        .unwrap();

    let usd_txs: Vec<TransactionCandidate> = usd_tx_stmt
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

    let summary = FinancialAggregator::compute_summary(&usd_txs, "USD");
    assert_eq!(summary.currency, "USD");
    assert!(summary.total_income_minor > 0, "Must have USD income");
    assert!(summary.total_expense_minor > 0, "Must have USD expenses");
    assert_eq!(
        summary.net_savings_minor,
        summary.total_income_minor - summary.total_expense_minor,
        "Net savings must equal total income minus total expense"
    );

    // Verify independent calculation of hand arithmetic:
    // Compute total sum manually over the list
    let mut manual_inc = 0i64;
    let mut manual_exp = 0i64;
    for tx in &usd_txs {
        if tx.amount_minor > 0 {
            manual_inc += tx.amount_minor;
        } else {
            manual_exp += tx.amount_minor.abs();
        }
    }
    assert_eq!(summary.total_income_minor, manual_inc);
    assert_eq!(summary.total_expense_minor, manual_exp);

    // Clean up test directory
    let _ = fs::remove_dir_all(&test_dir);
}
