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
        let row: Option<(String, Option<String>, Option<String>, i64, String, i64)> = conn
            .query_row(
                "SELECT i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency, i.tax_minor
                 FROM invoices i JOIN documents d ON d.id = i.document_id
                 WHERE d.workspace_id = ?1 AND d.file_name = ?2",
                [workspace, file],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .ok();
        let Some((vendor, number, date, total, currency, tax)) = row else {
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
        // The tax line must be read exactly as printed (the arithmetic-mismatch document only
        // changes the printed total).
        if tax != inv["tax_minor"].as_i64().unwrap() {
            problems.push(format!("tax {} != {}", tax, inv["tax_minor"]));
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
                let stored: Option<(i64, i64)> = db
                    .conn_ref()
                    .query_row(
                        "SELECT i.total_minor, i.tax_minor FROM invoices i JOIN documents d ON d.id = i.document_id
                         WHERE d.workspace_id = ?1 AND d.file_name = ?2",
                        [workspace, file],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .ok();
                let total = stored.map(|(t, _)| t);
                let expected = rec["total_minor"].as_i64().unwrap();
                // Receipts carry no tax line, so any extracted tax is a misread (for example a
                // "Taxi fare" item read as tax).
                if let Some((_, tax)) = stored {
                    if tax != 0 {
                        out.mismatches += 1;
                        let _ = writeln!(out.report, "- RECEIPT TAX MISREAD {}: extracted tax {}", file, tax);
                    }
                }
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


    // ---- reconciliation and anomalies, checked against the ledger's expected outcomes
    {
        use crate::finance::anomaly::AnomalyDetector;
        use crate::finance::matching::{InvoiceCandidate, ReconciliationMatcher, TransactionCandidate};

        let conn = db.conn_ref();
        let invoices: Vec<InvoiceCandidate> = conn
            .prepare(
                "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
                 FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = ?1",
            )
            .unwrap()
            .query_map([workspace], |r| {
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
            .map(|x| x.unwrap())
            .collect();
        let transactions: Vec<TransactionCandidate> = conn
            .prepare(
                "SELECT id, posted_date, description, amount_minor, currency, external_ref
                 FROM transactions WHERE workspace_id = ?1",
            )
            .unwrap()
            .query_map([workspace], |r| {
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
            .collect();

        let links = ReconciliationMatcher::match_records(&invoices, &transactions);
        let anomalies = AnomalyDetector::detect_anomalies(&invoices, &transactions);
        let _ = writeln!(out.report, "\nreconciliation: {} links, {} anomalies", links.len(), anomalies.len());

        let inv_by_id = |id: &str| invoices.iter().find(|i| i.id == id);
        let tx_by_row = |date: &str, desc: &str, minor: i64, cur: &str| {
            transactions
                .iter()
                .find(|t| t.posted_date == date && t.description == desc && t.amount_minor == minor && t.currency == cur)
        };
        let ids_for_number = |n: &str| -> Vec<&str> {
            invoices.iter().filter(|i| i.invoice_number.as_deref() == Some(n)).map(|i| i.id.as_str()).collect()
        };
        let mut problems: Vec<String> = Vec::new();

        let bank_rows = ledger["bank_rows"].as_array().unwrap();
        let expect = &ledger["expect"];

        // A. Paid-in-full invoices link, with an EXACT match, to their own payment.
        let (mut matched_ok, mut matched_total) = (0, 0);
        for row in bank_rows.iter().filter(|r| r["kind"] == "matched") {
            matched_total += 1;
            let tx = tx_by_row(
                row["date"].as_str().unwrap(),
                row["description"].as_str().unwrap(),
                row["minor"].as_i64().unwrap(),
                row["currency"].as_str().unwrap(),
            );
            let want = row["invoice"].as_str().unwrap();
            // An invoice with no readable date can only be matched on amount and reference.
            let undated = ledger["invoices"].as_array().unwrap().iter().any(|i| {
                i["number"] == want && i["defects"].as_array().unwrap().iter().any(|d| d == "missing_date")
            });
            let want_type = if undated { "EXACT_NO_DATE" } else { "EXACT" };
            match tx.and_then(|t| links.iter().find(|l| l.transaction_id == t.id)) {
                Some(l) => {
                    let got = inv_by_id(&l.invoice_id).and_then(|i| i.invoice_number.clone());
                    if got.as_deref() == Some(want) && l.match_type == want_type {
                        matched_ok += 1;
                    } else {
                        problems.push(format!("payment for {} linked to {:?} as {}", want, got, l.match_type));
                    }
                }
                None => problems.push(format!("payment for {} has no link", want)),
            }
        }
        let _ = writeln!(out.report, "- paid-in-full matched correctly: {}/{}", matched_ok, matched_total);

        // B. Every duplicate payment is flagged for review.
        for num in expect["duplicate_payment_invoices"].as_array().unwrap() {
            let num = num.as_str().unwrap();
            let rows: Vec<&Value> = bank_rows
                .iter()
                .filter(|r| r["invoice"] == num && r["kind"] == "duplicate_payment")
                .collect();
            let tx_ids: Vec<&str> = rows
                .iter()
                .filter_map(|r| {
                    tx_by_row(
                        r["date"].as_str().unwrap(),
                        r["description"].as_str().unwrap(),
                        r["minor"].as_i64().unwrap(),
                        r["currency"].as_str().unwrap(),
                    )
                })
                .map(|t| t.id.as_str())
                .collect();
            let flagged = anomalies
                .iter()
                .any(|a| a.anomaly_type == "POTENTIAL_DUPLICATE_PAYMENT" && tx_ids.contains(&a.entity_id.as_str()));
            let _ = writeln!(out.report, "- duplicate payment {} flagged: {}", num, flagged);
            if !flagged {
                problems.push(format!("duplicate payment for {} was not flagged", num));
            }
        }

        // C/D. Short and partial payments are never accepted as an exact match.
        for key in ["short_payment_invoices", "partial_payment_invoices"] {
            for num in expect[key].as_array().unwrap() {
                let num = num.as_str().unwrap();
                let ids = ids_for_number(num);
                let exact = links
                    .iter()
                    .any(|l| ids.contains(&l.invoice_id.as_str()) && l.match_type == "EXACT");
                let _ = writeln!(out.report, "- {} {} treated as exact match: {}", key, num, exact);
                if exact {
                    problems.push(format!("{} ({}) was matched as EXACT", num, key));
                }
            }
        }

        // E. Unpaid invoices have no exact link.
        for num in expect["unpaid_invoices"].as_array().unwrap() {
            let ids = ids_for_number(num.as_str().unwrap());
            if links
                .iter()
                .any(|l| ids.contains(&l.invoice_id.as_str()) && l.match_type == "EXACT")
            {
                problems.push(format!("unpaid invoice {} was matched as EXACT", num));
            }
        }

        // F. The unexplained card payment is not linked to any invoice.
        if let Some(t) = transactions.iter().find(|t| t.description == "UNKNOWN CARD MERCHANT") {
            if links.iter().any(|l| l.transaction_id == t.id) {
                problems.push("UNKNOWN CARD MERCHANT was linked to an invoice".to_string());
            }
        }

        // G. No link may join records in different currencies.
        for l in &links {
            let inv = inv_by_id(&l.invoice_id).unwrap();
            let tx = transactions.iter().find(|t| t.id == l.transaction_id).unwrap();
            if inv.currency != tx.currency {
                problems.push(format!(
                    "cross-currency link: invoice {} {} vs payment {}",
                    inv.id, inv.currency, tx.currency
                ));
            }
        }

        // H. The re-sent invoice and the reused invoice number are flagged.
        let pair = expect["suspected_duplicate_invoice_pair"].as_array().unwrap();
        let pair_ids: Vec<&str> = pair.iter().flat_map(|n| ids_for_number(n.as_str().unwrap())).collect();
        let resend_flagged = anomalies.iter().any(|a| {
            a.entity_type == "INVOICE"
                && pair_ids.contains(&a.entity_id.as_str())
                && (a.anomaly_type == "SUSPECTED_DUPLICATE_BILLING" || a.anomaly_type == "DUPLICATE_INVOICE")
        });
        let _ = writeln!(out.report, "- re-sent invoice flagged: {}", resend_flagged);
        if !resend_flagged {
            problems.push("re-sent invoice was not flagged".to_string());
        }
        let clash_ids = ids_for_number(expect["duplicate_invoice_number"].as_str().unwrap());
        let clash_flagged = anomalies
            .iter()
            .any(|a| a.anomaly_type == "DUPLICATE_INVOICE" && clash_ids.contains(&a.entity_id.as_str()));
        let _ = writeln!(out.report, "- reused invoice number flagged: {}", clash_flagged);
        if !clash_flagged {
            problems.push("reused invoice number was not flagged".to_string());
        }

        // I. Recurring charges are never flagged as duplicates.
        for name in expect["recurring_must_not_flag"].as_array().unwrap() {
            let name = name.as_str().unwrap();
            if let Some(a) = anomalies.iter().find(|a| {
                (a.anomaly_type == "POTENTIAL_DUPLICATE_PAYMENT" || a.anomaly_type == "DUPLICATE_TRANSACTION")
                    && a.description.contains(name)
            }) {
                problems.push(format!("recurring charge flagged as duplicate: {}", a.description));
            }
        }

        for p in &problems {
            let _ = writeln!(out.report, "- RECON PROBLEM: {}", p);
        }
        out.mismatches += problems.len();
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
