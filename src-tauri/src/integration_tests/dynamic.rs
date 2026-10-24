//! Metamorphic tests: results must follow the data, not the example.
//!
//! - Adding a transaction changes the totals by exactly that amount.
//! - Row order never changes totals.
//! - Different workspaces give different, independently correct results.
//! - Correcting an approved record is reflected afterwards.
//!
//! Placement note: kept in the library test binary; see corpus.rs for the reason.

use crate::ai::GroundingEngine;
use crate::db::Database;
use crate::finance::review::ReviewService;
use crate::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use crate::security::{BlobStore, KeyDeriver};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub(crate) fn corpus_dir(workspace: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/corpus")
        .join(workspace)
}

pub(crate) struct Harness {
    pub(crate) db: Database,
    blobs: BlobStore,
    ocr: OcrClient,
    master: zeroize::Zeroizing<[u8; 32]>,
    temp: PathBuf,
}

impl Harness {
    pub(crate) fn new(workspace: &str) -> Self {
        let temp = std::env::temp_dir().join(format!("saar_dynamic_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        let blobs = BlobStore::new(temp.join("blobs")).unwrap();
        let db = Database::open_in_memory().unwrap();
        let salt = KeyDeriver::generate_salt();
        let master = KeyDeriver::derive_master_key("Dynamic-Results-Passphrase-2026", &salt).unwrap();
        let token = KeyDeriver::compute_verification_token(&master);
        db.create_workspace(workspace, workspace, "USD", &salt, &token).unwrap();
        Harness { db, blobs, ocr: OcrClient::new().expect("OCR client"), master, temp }
    }

    pub(crate) fn ingest_csv(&mut self, workspace: &str, csv_text: &str) {
        let path = self.temp.join(format!("{}.csv", Uuid::new_v4()));
        fs::write(&path, csv_text).unwrap();
        match DocumentOrchestrator::ingest_file(&mut self.db, &self.blobs, &self.ocr, &self.master, workspace, &path, "USD") {
            Ok(IngestResult::Success { .. }) => {}
            other => panic!("csv must ingest as new: {:?}", other.map(|_| "non-success")),
        }
    }

    pub(crate) fn ingest_file(&mut self, workspace: &str, path: &std::path::Path) {
        match DocumentOrchestrator::ingest_file(&mut self.db, &self.blobs, &self.ocr, &self.master, workspace, path, "USD") {
            Ok(IngestResult::Success { .. }) => {}
            other => panic!("{} must ingest as new: {:?}", path.display(), other.map(|_| "non-success")),
        }
    }

    pub(crate) fn usd_totals(&self, workspace: &str) -> (i64, i64) {
        GroundingEngine::scoped_totals(self.db.conn_ref(), workspace, "USD").unwrap()
    }

    pub(crate) fn tx_count(&self, workspace: &str) -> i64 {
        self.db
            .conn_ref()
            .query_row("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1", [workspace], |r| r.get(0))
            .unwrap()
    }
}

#[test]
#[ignore = "runs real ingestion over the corpus bank statements"]
fn results_follow_the_data() {
    let alpha_csv = fs::read_to_string(corpus_dir("workspace_alpha").join("bank_statement.csv")).unwrap();
    let beta_csv = fs::read_to_string(corpus_dir("workspace_beta").join("bank_statement.csv")).unwrap();
    let alpha_ledger: Value =
        serde_json::from_str(&fs::read_to_string(corpus_dir("workspace_alpha").join("ledger.json")).unwrap()).unwrap();
    let beta_ledger: Value =
        serde_json::from_str(&fs::read_to_string(corpus_dir("workspace_beta").join("ledger.json")).unwrap()).unwrap();

    // 1. Different workspaces: each matches its own independent ground truth, and they differ.
    let mut a = Harness::new("ws-a");
    a.ingest_csv("ws-a", &alpha_csv);
    let (a_inc, a_exp) = a.usd_totals("ws-a");
    assert_eq!(a_inc, alpha_ledger["bank_by_currency"]["USD"]["income_minor"].as_i64().unwrap());
    assert_eq!(a_exp, alpha_ledger["bank_by_currency"]["USD"]["expense_minor"].as_i64().unwrap());

    let mut b = Harness::new("ws-b");
    b.ingest_csv("ws-b", &beta_csv);
    let (b_inc, b_exp) = b.usd_totals("ws-b");
    assert_eq!(b_inc, beta_ledger["bank_by_currency"]["USD"]["income_minor"].as_i64().unwrap());
    assert_eq!(b_exp, beta_ledger["bank_by_currency"]["USD"]["expense_minor"].as_i64().unwrap());
    assert_ne!((a_inc, a_exp), (b_inc, b_exp), "different data must give different results");

    // 2. Row order does not change totals: the same rows reversed give identical results.
    let mut lines = alpha_csv.lines();
    let header = lines.next().unwrap();
    let mut rows: Vec<&str> = lines.collect();
    rows.reverse();
    let reversed = format!("{}\n{}\n", header, rows.join("\n"));
    let mut a2 = Harness::new("ws-a2");
    a2.ingest_csv("ws-a2", &reversed);
    assert_eq!(a2.usd_totals("ws-a2"), (a_inc, a_exp), "row order must not affect totals");
    assert_eq!(a2.tx_count("ws-a2"), a.tx_count("ws-a"));

    // 3. Adding a transaction changes the totals by exactly its amount, in the right direction.
    let before_count = a.tx_count("ws-a");
    a.ingest_csv(
        "ws-a",
        &format!("{}\n2026-01-15,ADDED AFTER THE FACT,1234.56,,USD,\n2026-01-16,LATE CUSTOMER PAYMENT,,200.00,USD,\n", header),
    );
    let (inc2, exp2) = a.usd_totals("ws-a");
    assert_eq!(exp2, a_exp + 123_456, "an added debit must raise expenses by exactly its amount");
    assert_eq!(inc2, a_inc + 20_000, "an added credit must raise income by exactly its amount");
    assert_eq!(a.tx_count("ws-a"), before_count + 2);

    // 4. Pagination consistency: summing the stored rows in pages equals the aggregate.
    let conn = a.db.conn_ref();
    let mut paged_expense = 0i64;
    let mut offset = 0;
    loop {
        let page: Vec<i64> = conn
            .prepare("SELECT amount_minor FROM transactions WHERE workspace_id = 'ws-a' AND currency = 'USD' ORDER BY posted_date, id LIMIT 25 OFFSET ?1")
            .unwrap()
            .query_map([offset], |r| r.get(0))
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        if page.is_empty() {
            break;
        }
        paged_expense += page.iter().filter(|m| **m < 0).map(|m| -m).sum::<i64>();
        offset += 25;
    }
    assert_eq!(paged_expense, exp2, "paged sum must equal the full aggregate");

    // 5. Correcting an approved record is reflected afterwards, and is audited.
    let (tx_id, original): (String, String) = a
        .db
        .conn_ref()
        .query_row(
            "SELECT id, description FROM transactions WHERE workspace_id = 'ws-a' AND description = 'UNKNOWN CARD MERCHANT'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(original, "UNKNOWN CARD MERCHANT");
    ReviewService::correct_transaction(&a.db, "ws-a", &tx_id, "RESOLVED: TRAINING PROVIDER").unwrap();
    let now: String = a
        .db
        .conn_ref()
        .query_row("SELECT description FROM transactions WHERE id = ?1", [&tx_id], |r| r.get(0))
        .unwrap();
    assert_eq!(now, "RESOLVED: TRAINING PROVIDER");
    let audited: i64 = a
        .db
        .conn_ref()
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE workspace_id = 'ws-a' AND entity_id = ?1",
            [&tx_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(audited >= 1, "the correction must leave an audit record");
}

#[test]
#[ignore = "runs the OCR worker on a corrupted file"]
fn a_failed_import_is_marked_failed_immediately_and_leaves_no_partial_records() {
    let corrupted = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/golden/bills/corrupted_invoice.pdf");
    assert!(corrupted.exists(), "fixture missing");

    let mut h = Harness::new("ws-fail");
    let result = DocumentOrchestrator::ingest_file(&mut h.db, &h.blobs, &h.ocr, &h.master, "ws-fail", &corrupted, "USD");
    assert!(result.is_err(), "a corrupted PDF must be rejected");

    let conn = h.db.conn_ref();
    let status: String = conn
        .query_row("SELECT status FROM documents WHERE workspace_id = 'ws-fail'", [], |r| r.get(0))
        .expect("the failed import is recorded");
    assert_eq!(status, "FAILED", "not left PROCESSING until the next startup");

    let invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = 'ws-fail'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(invoices, 0, "no partial invoice record");
    let audited: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_log WHERE workspace_id = 'ws-fail' AND action = 'INGEST_FAILED'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(audited, 1, "the failure is audited");
}
