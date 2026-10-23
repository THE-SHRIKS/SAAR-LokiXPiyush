//! Runs each synthetic workspace corpus (tests/fixtures/corpus) through the real ingestion
//! pipeline and compares what SAAR extracted against the independent ground-truth ledger.
//!
//! Placement note: this lives in the library test binary rather than tests/ because newly built
//! integration-test executables are currently blocked by the machine's Application Control policy.
//!
//! The test writes a per-file comparison report and then asserts that nothing mismatched. A
//! failure is a real extraction or financial defect; do not weaken the comparison.
//!
//! Run: cargo test --lib corpus -- --ignored --nocapture
//! Report path: env CORPUS_REPORT, default <temp>/saar_corpus_report.md

use crate::db::Database;
use crate::ingestion::{DocumentOrchestrator, IngestResult, OcrClient};
use crate::security::{BlobStore, KeyDeriver};
use serde_json::Value;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn corpus_dir(workspace: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/corpus")
        .join(workspace)
}

struct Outcome {
    report: String,
    mismatches: usize,
}

fn run_workspace(workspace: &str) -> Outcome {
    let dir = corpus_dir(workspace);
    let ledger: Value = serde_json::from_str(&fs::read_to_string(dir.join("ledger.json")).unwrap()).unwrap();

    let temp = std::env::temp_dir().join(format!("saar_corpus_{}", Uuid::new_v4()));
    let blobs = BlobStore::new(temp.join("blobs")).unwrap();
    let ocr = OcrClient::new().expect("OCR client");
    let mut db = Database::open_in_memory().unwrap();
    let salt = KeyDeriver::generate_salt();
    let master = KeyDeriver::derive_master_key("Corpus-Workspace-Passphrase-2026", &salt).unwrap();
    let token = KeyDeriver::compute_verification_token(&master);
    db.create_workspace(workspace, workspace, "USD", &salt, &token).unwrap();

    let mut out = Outcome { report: String::new(), mismatches: 0 };
    let _ = writeln!(out.report, "## {}\n", workspace);

    // ---- invoices (PDF)
    let mut ingest_failures = Vec::new();
    for inv in ledger["invoices"].as_array().unwrap() {
        let file = inv["file"].as_str().unwrap();
        let path = dir.join("invoices").join(file);
        match DocumentOrchestrator::ingest_file(&mut db, &blobs, &ocr, &master, workspace, &path, "USD") {
            Ok(IngestResult::Success { .. }) => {}
            Ok(IngestResult::Duplicate { .. }) => ingest_failures.push(format!("{}: reported as byte-duplicate", file)),
            Err(e) => ingest_failures.push(format!("{}: ingestion error: {}", file, e)),
        }
    }
    for e in &ingest_failures {
        let _ = writeln!(out.report, "- INGEST FAIL {}", e);
    }
    out.mismatches += ingest_failures.len();

    let conn = db.conn_ref();
    let mut compared = 0;
    for inv in ledger["invoices"].as_array().unwrap() {
        let file = inv["file"].as_str().unwrap();
        let row: Option<(String, Option<String>, Option<String>, i64, String)> = conn
            .query_row(
                "SELECT i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
                 FROM invoices i JOIN documents d ON d.id = i.document_id
                 WHERE d.workspace_id = ?1 AND d.file_name = ?2",
                [workspace, file],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .ok();
        let Some((vendor, number, date, total, currency)) = row else {
            if !ingest_failures.iter().any(|f| f.starts_with(file)) {
                let _ = writeln!(out.report, "- NOT STORED {}", file);
                out.mismatches += 1;
            }
            continue;
        };
        compared += 1;
        let defects = inv["defects"].as_array().unwrap();
        let has_missing_date = defects.iter().any(|d| d == "missing_date");
        let has_arith = defects.iter().any(|d| d == "arith_mismatch");
        let mut problems = Vec::new();
        if vendor != inv["vendor"].as_str().unwrap() {
            problems.push(format!("vendor '{}' != '{}'", vendor, inv["vendor"]));
        }
        if number.as_deref() != inv["number"].as_str() {
            problems.push(format!("number {:?} != {}", number, inv["number"]));
        }
        if currency != inv["currency"].as_str().unwrap() {
            problems.push(format!("currency {} != {}", currency, inv["currency"]));
        }
        // The arithmetic-mismatch document prints a total that is deliberately 1 minor unit off.
        let expected_total = inv["total_minor"].as_i64().unwrap() + if has_arith { 1 } else { 0 };
        if total != expected_total {
            problems.push(format!("total {} != {}", total, expected_total));
        }
        if !has_missing_date && date.as_deref() != inv["issued"].as_str() {
            problems.push(format!("date {:?} != {}", date, inv["issued"]));
        }
        if has_missing_date && date.is_some() {
            problems.push(format!("date {:?} present but the source has none", date));
        }
        if !problems.is_empty() {
            out.mismatches += 1;
            let _ = writeln!(out.report, "- MISMATCH {}: {}", file, problems.join("; "));
        }
    }
    let _ = writeln!(out.report, "\ninvoices compared: {} of {}", compared, ledger["invoices"].as_array().unwrap().len());

    // ---- receipts (scanned images, OCR)
    for rec in ledger["receipts"].as_array().unwrap() {
        let file = rec["file"].as_str().unwrap();
        let path = dir.join("receipts").join(file);
        match DocumentOrchestrator::ingest_file(&mut db, &blobs, &ocr, &master, workspace, &path, "USD") {
            Ok(_) => {
                let total: Option<i64> = db
                    .conn_ref()
                    .query_row(
                        "SELECT i.total_minor FROM invoices i JOIN documents d ON d.id = i.document_id
                         WHERE d.workspace_id = ?1 AND d.file_name = ?2",
                        [workspace, file],
                        |r| r.get(0),
                    )
                    .ok();
                let expected = rec["total_minor"].as_i64().unwrap();
                match total {
                    Some(t) if t == expected => {
                        let _ = writeln!(out.report, "- receipt OK {} ({})", file, rec["quality"]);
                    }
                    other => {
                        out.mismatches += 1;
                        let _ = writeln!(out.report, "- RECEIPT MISMATCH {} ({}): extracted {:?}, expected {}", file, rec["quality"], other, expected);
                    }
                }
            }
            Err(e) => {
                out.mismatches += 1;
                let _ = writeln!(out.report, "- RECEIPT INGEST FAIL {}: {}", file, e);
            }
        }
    }

    // ---- bank statement
    match DocumentOrchestrator::ingest_file(&mut db, &blobs, &ocr, &master, workspace, &dir.join("bank_statement.csv"), "USD") {
        Ok(_) => {
            let conn = db.conn_ref();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1", [workspace], |r| r.get(0))
                .unwrap();
            let expected_rows = ledger["bank_row_count"].as_i64().unwrap();
            if count != expected_rows {
                out.mismatches += 1;
                let _ = writeln!(out.report, "- BANK ROW COUNT {} != {}", count, expected_rows);
            }
            for (cur, exp) in ledger["bank_by_currency"].as_object().unwrap() {
                let (n, income, expense): (i64, i64, i64) = conn
                    .query_row(
                        "SELECT COUNT(*),
                                COALESCE(SUM(CASE WHEN amount_minor > 0 THEN amount_minor ELSE 0 END), 0),
                                COALESCE(SUM(CASE WHEN amount_minor < 0 THEN -amount_minor ELSE 0 END), 0)
                         FROM transactions WHERE workspace_id = ?1 AND currency = ?2",
                        [workspace, cur.as_str()],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .unwrap();
                let ok = n == exp["count"].as_i64().unwrap()
                    && income == exp["income_minor"].as_i64().unwrap()
                    && expense == exp["expense_minor"].as_i64().unwrap();
                if !ok {
                    out.mismatches += 1;
                    let _ = writeln!(out.report, "- BANK {} MISMATCH: got count {} income {} expense {}; expected {}", cur, n, income, expense, exp);
                } else {
                    let _ = writeln!(out.report, "- bank {} OK: {} rows, income {}, expense {}", cur, n, income, expense);
                }
            }
        }
        Err(e) => {
            out.mismatches += 1;
            let _ = writeln!(out.report, "- BANK INGEST FAIL: {}", e);
        }
    }

    let _ = fs::remove_dir_all(&temp);
    out
}

#[test]
#[ignore = "heavy: runs OCR over the whole corpus"]
fn corpus_workspaces_match_ground_truth() {
    let mut report = String::from("# Corpus run against ground truth\n\n");
    let mut total_mismatches = 0;
    for ws in ["workspace_alpha", "workspace_beta"] {
        let o = run_workspace(ws);
        total_mismatches += o.mismatches;
        report.push_str(&o.report);
        let _ = writeln!(report, "\nmismatches in {}: {}\n", ws, o.mismatches);
    }
    let path = std::env::var("CORPUS_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("saar_corpus_report.md"));
    fs::write(&path, &report).unwrap();
    println!("report written to {}", path.display());
    assert_eq!(total_mismatches, 0, "corpus mismatches; see {}", path.display());
}
