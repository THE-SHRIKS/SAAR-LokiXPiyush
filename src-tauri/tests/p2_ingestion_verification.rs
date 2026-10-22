use saar_lib::db::Database;
use saar_lib::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use saar_lib::security::{BlobStore, KeyDeriver};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_p2_full_document_and_statement_ingestion_pipeline() {
    let test_dir = std::env::temp_dir().join(format!("opendeck_p2_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    let db_path = test_dir.join("test_p2_workspace.db");
    let blob_dir = test_dir.join("blobs");

    let mut db = Database::open(&db_path).expect("Failed to open test database");
    let blob_store = BlobStore::new(&blob_dir).expect("Failed to create blob store");
    let ocr_client = OcrClient::new().expect("Failed to create OCR client");

    let passphrase = "P2IngestionTestPassphrase#2026";
    let salt = KeyDeriver::generate_salt();
    let master_key = KeyDeriver::derive_master_key(passphrase, &salt).unwrap();
    let verification_token = KeyDeriver::compute_verification_token(&master_key);

    let workspace_id = "ws-p2-test";
    db.create_workspace(
        workspace_id,
        "P2 Test Workspace",
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
    // Test 1: Born-Digital PDF Invoice Ingestion & Field Provenance
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
        IngestResult::Success {
            document_id,
            doc_type,
            ..
        } => {
            assert_eq!(doc_type, "PDF");
            document_id
        }
        other => panic!("Expected IngestResult::Success, got {:?}", other),
    };

    // Verify invoice fields and exact minor units in database
    let (vendor, inv_num, total_minor, tax_minor): (String, Option<String>, i64, i64) = db
        .conn_ref()
        .query_row(
            "SELECT vendor_name, invoice_number, total_minor, tax_minor FROM invoices WHERE document_id = ?1;",
            params![gst_doc_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .expect("Invoice must be inserted into invoices table");

    assert_eq!(vendor, "ACME Tech Services Pvt Ltd");
    assert_eq!(inv_num, Some("ACME-2026-0915".to_string()));
    assert_eq!(total_minor, 11800000); // 118,000.00 INR = 11,800,000 minor units
    assert_eq!(tax_minor, 1800000); // 18,000.00 INR = 1,800,000 minor units

    // Verify document fields have bounding box coordinates for UI highlights
    let field_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM document_fields WHERE document_id = ?1 AND bounding_box_json != '[]';",
            params![gst_doc_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        field_count > 0,
        "Document fields must retain bounding box provenance"
    );

    // -------------------------------------------------------------
    // Test 2: Scanned Thermal Receipt OCR Ingestion
    // -------------------------------------------------------------
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

    let receipt_doc_id = match receipt_res {
        IngestResult::Success {
            document_id,
            doc_type,
            ..
        } => {
            assert_eq!(doc_type, "PNG");
            document_id
        }
        other => panic!("Expected IngestResult::Success, got {:?}", other),
    };

    let receipt_vendor: String = db
        .conn_ref()
        .query_row(
            "SELECT vendor_name FROM invoices WHERE document_id = ?1;",
            params![receipt_doc_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(receipt_vendor, "OFFICE SUPPLIES MART");

    // -------------------------------------------------------------
    // Test 3: CSV Bank Statement Ingestion (US Format)
    // -------------------------------------------------------------
    let us_csv_path = root_fixtures.join("statements/bank_us_sample.csv");
    let us_csv_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &us_csv_path,
        "USD",
    )
    .expect("Failed to ingest bank_us_sample.csv");

    match us_csv_res {
        IngestResult::Success {
            doc_type,
            records_extracted,
            ..
        } => {
            assert_eq!(doc_type, "CSV");
            assert_eq!(records_extracted, 7);
        }
        other => panic!("Expected IngestResult::Success, got {:?}", other),
    }

    // -------------------------------------------------------------
    // Test 4: CSV Bank Statement Ingestion (HDFC Indian Format)
    // -------------------------------------------------------------
    let hdfc_csv_path = root_fixtures.join("statements/bank_hdfc_sample.csv");
    let hdfc_csv_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &hdfc_csv_path,
        "INR",
    )
    .expect("Failed to ingest bank_hdfc_sample.csv");

    match hdfc_csv_res {
        IngestResult::Success {
            doc_type,
            records_extracted,
            ..
        } => {
            assert_eq!(doc_type, "CSV");
            assert_eq!(records_extracted, 6);
        }
        other => panic!("Expected IngestResult::Success, got {:?}", other),
    }

    // Verify transactions in database
    let total_tx_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1;",
            params![workspace_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(total_tx_count, 13); // 7 (US) + 6 (HDFC)

    // -------------------------------------------------------------
    // Test 5: Content Deduplication
    // -------------------------------------------------------------
    let dup_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &gst_pdf_path,
        "INR",
    )
    .expect("Duplicate ingestion should succeed returning Duplicate variant");

    match dup_res {
        IngestResult::Duplicate {
            existing_document_id,
            ..
        } => {
            assert_eq!(existing_document_id, gst_doc_id);
        }
        other => panic!("Expected IngestResult::Duplicate, got {:?}", other),
    }

    // Verify document count did not increase
    let doc_count: i64 = db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE workspace_id = ?1;",
            params![workspace_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(doc_count, 4); // GST + Receipt + US CSV + HDFC CSV

    // -------------------------------------------------------------
    // Test 6: Malicious & Corrupted File Boundaries
    // -------------------------------------------------------------
    let corrupted_pdf_path = root_fixtures.join("adversarial/corrupted_truncated.pdf");
    let corrupt_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &corrupted_pdf_path,
        "USD",
    );
    assert!(corrupt_res.is_err(), "Corrupted PDF must fail gracefully");

    let bomb_csv_path = root_fixtures.join("adversarial/bomb_50k_rows.csv");
    let bomb_res = DocumentOrchestrator::ingest_file(
        &mut db,
        &blob_store,
        &ocr_client,
        &master_key,
        workspace_id,
        &bomb_csv_path,
        "USD",
    );
    assert!(bomb_res.is_err(), "CSV row bomb must be rejected");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
