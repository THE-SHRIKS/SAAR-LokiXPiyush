//! Runs the synthetic golden dataset (tests/fixtures/golden) through real ingestion and compares
//! the results against expectations computed independently by generate_golden.py.
//! A failing assertion here is a real defect; do not weaken the expectations.

use saar_lib::db::Database;
use saar_lib::finance::anomaly::AnomalyDetector;
use saar_lib::finance::matching::TransactionCandidate;
use saar_lib::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use saar_lib::ai::GroundingEngine;
use saar_lib::security::{BlobStore, KeyDeriver};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/golden")
}

#[test]
fn test_golden_dataset_matches_independent_expectations() {
    let expected: Value =
        serde_json::from_str(&fs::read_to_string(golden_dir().join("expected.json")).unwrap()).unwrap();

    let temp = std::env::temp_dir().join(format!("saar_golden_{}", Uuid::new_v4()));
    let blobs = BlobStore::new(temp.join("blobs")).unwrap();
    let ocr = OcrClient::new().expect("OCR client");
    let mut db = Database::open_in_memory().unwrap();
    let salt = KeyDeriver::generate_salt();
    let master = KeyDeriver::derive_master_key("Golden-Dataset-Passphrase-2026", &salt).unwrap();
    let token = KeyDeriver::compute_verification_token(&master);
    let ws = "ws-golden";
    db.create_workspace(ws, "Golden", "USD", &salt, &token).unwrap();

    let ingest = DocumentOrchestrator::ingest_file(
        &mut db,
        &blobs,
        &ocr,
        &master,
        ws,
        &golden_dir().join("bank_golden.csv"),
        "USD",
    );
    let stored = match ingest {
        Ok(IngestResult::Success { .. }) => {}
        Ok(_) => panic!("golden CSV must ingest as a new document"),
        Err(e) => panic!("golden CSV ingestion failed: {}", e),
    };
    let _ = stored;

    let conn = db.conn_ref();

    // 1. Row count
    let tx_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1", [ws], |r| r.get(0))
        .unwrap();
    assert_eq!(
        tx_count,
        expected["transaction_count"].as_i64().unwrap(),
        "every golden row must be stored, none dropped"
    );

    // 2. Per-currency counts and exact minor-unit sums, across all currencies
    for (currency, exp) in expected["by_currency"].as_object().unwrap() {
        let (count, income, expense): (i64, i64, i64) = conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN amount_minor > 0 THEN amount_minor ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN amount_minor < 0 THEN -amount_minor ELSE 0 END), 0)
                 FROM transactions WHERE workspace_id = ?1 AND currency = ?2",
                [ws, currency.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(count, exp["count"].as_i64().unwrap(), "{} row count", currency);
        assert_eq!(income, exp["income_minor"].as_i64().unwrap(), "{} income (minor units)", currency);
        assert_eq!(expense, exp["expense_minor"].as_i64().unwrap(), "{} expense (minor units)", currency);
    }

    // 3. Base-currency totals in the fact packet must equal the full aggregate, not a sample.
    let (income, expense) = GroundingEngine::scoped_totals(conn, ws, "USD").unwrap();
    assert_eq!(income, expected["by_currency"]["USD"]["income_minor"].as_i64().unwrap());
    assert_eq!(expense, expected["by_currency"]["USD"]["expense_minor"].as_i64().unwrap());

    // 4. Potential duplicate payments: exactly the expected pairs, and no subscription pairs.
    let candidates: Vec<TransactionCandidate> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, posted_date, description, amount_minor, currency, external_ref
                 FROM transactions WHERE workspace_id = ?1",
            )
            .unwrap();
        stmt.query_map([ws], |r| {
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
        .map(|x| x.unwrap())
        .collect()
    };
    let flagged: Vec<String> = AnomalyDetector::detect_potential_duplicate_payments(&candidates)
        .into_iter()
        .map(|a| a.description)
        .collect();
    let expected_dups = expected["potential_duplicate_payments"].as_array().unwrap();
    assert_eq!(
        flagged.len(),
        expected_dups.len(),
        "expected {} potential duplicate pairs, got {:?}",
        expected_dups.len(),
        flagged
    );
    for dup in expected_dups {
        let first = dup["dates"][0].as_str().unwrap();
        let second = dup["dates"][1].as_str().unwrap();
        assert!(
            flagged.iter().any(|f| f.contains(first) && f.contains(second)),
            "pair {} / {} must be flagged for review",
            first,
            second
        );
    }
    for sub in expected["monthly_subscription_pairs_not_flagged"].as_array().unwrap() {
        let _ = sub; // checked in aggregate: no flag may mention the subscription merchant
    }
    assert!(
        flagged.iter().all(|f| !f.to_lowercase().contains("cloudhost")),
        "monthly subscription repeats must not be flagged as duplicate payments"
    );

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_malformed_row_is_reported_not_silently_dropped() {
    let temp = std::env::temp_dir().join(format!("saar_golden_bad_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp).unwrap();
    let bad = temp.join("bad_rows.csv");
    fs::write(
        &bad,
        "Date,Description,Debit,Credit,Currency,Balance\n\
         2026-09-01,GOOD ROW,10.00,,USD,\n\
         2026-09-02,BAD AMOUNT ROW,twelve,,USD,\n\
         2026-09-03,ANOTHER GOOD ROW,5.00,,USD,\n",
    )
    .unwrap();

    let blobs = BlobStore::new(temp.join("blobs")).unwrap();
    let ocr = OcrClient::new().expect("OCR client");
    let mut db = Database::open_in_memory().unwrap();
    let salt = KeyDeriver::generate_salt();
    let master = KeyDeriver::derive_master_key("Malformed-Row-Passphrase-2026", &salt).unwrap();
    let token = KeyDeriver::compute_verification_token(&master);
    db.create_workspace("ws-bad", "Bad", "USD", &salt, &token).unwrap();

    let result = DocumentOrchestrator::ingest_file(&mut db, &blobs, &ocr, &master, "ws-bad", &bad, "USD");
    match result {
        Err(e) => assert!(e.to_string().contains("parse") || e.to_string().contains("Row"), "error must name the bad row: {}", e),
        Ok(IngestResult::Success { records_extracted, .. }) => {
            panic!("file with a malformed row was accepted with {} records; the bad row must not be dropped silently", records_extracted)
        }
        Ok(_) => panic!("unexpected ingest result"),
    }
    let stored: i64 = db
        .conn_ref()
        .query_row("SELECT COUNT(*) FROM transactions WHERE workspace_id = 'ws-bad'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, 0, "a rejected file must leave no partial transactions behind");
    let _ = fs::remove_dir_all(&temp);
}
