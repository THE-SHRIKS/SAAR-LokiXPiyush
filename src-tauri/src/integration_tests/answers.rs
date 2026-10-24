//! The deterministic answer layer, checked against the independent ground-truth ledger.
//!
//! Every expected figure here is computed from `ledger.json` (written by the corpus generator in
//! integer minor units), never from SAAR. Questions are phrased in plain language.
//!
//! Placement note: kept in the library test binary; see corpus.rs for the reason.

use super::dynamic::{corpus_dir, Harness};
use crate::ai::financial_ops::{try_answer, VerifiedAnswer};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;

const WS: &str = "ws-answers";

fn ask(h: &Harness, q: &str) -> Option<VerifiedAnswer> {
    try_answer(h.db.conn_ref(), WS, "USD", q).expect("operation must not error")
}

fn figure(a: &VerifiedAnswer, label: &str) -> i64 {
    a.figures
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no '{}' figure in {:?}", label, a.figures))
        .minor
}

/// Independent expectation straight from the ledger rows.
fn expected(ledger: &Value, currency: &str, prefix: &str, merchant: Option<&str>) -> (i64, i64) {
    let (mut income, mut expense) = (0i64, 0i64);
    for r in ledger["bank_rows"].as_array().unwrap() {
        if r["currency"] != currency || !r["date"].as_str().unwrap().starts_with(prefix) {
            continue;
        }
        if let Some(m) = merchant {
            if !r["description"].as_str().unwrap().to_lowercase().contains(m) {
                continue;
            }
        }
        let minor = r["minor"].as_i64().unwrap();
        if minor > 0 {
            income += minor;
        } else {
            expense += -minor;
        }
    }
    (income, expense)
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
    "November", "December",
];

#[test]
#[ignore = "runs real ingestion over the corpus"]
fn answers_equal_the_independent_ledger() {
    let ledger: Value =
        serde_json::from_str(&fs::read_to_string(corpus_dir("workspace_alpha").join("ledger.json")).unwrap()).unwrap();
    let mut h = Harness::new(WS);
    h.ingest_csv(WS, &fs::read_to_string(corpus_dir("workspace_alpha").join("bank_statement.csv")).unwrap());

    // 1. Every month with data: spend, income and net cash flow in USD.
    let months: BTreeSet<String> = ledger["bank_rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["date"].as_str().unwrap()[..7].to_string())
        .collect();
    assert!(months.len() >= 12, "the corpus must span a year or more");
    for ym in &months {
        let (year, month): (i32, usize) = (ym[..4].parse().unwrap(), ym[5..7].parse().unwrap());
        let name = format!("{} {}", MONTHS[month - 1], year);
        let (inc, exp) = expected(&ledger, "USD", ym, None);

        let spend = ask(&h, &format!("How much did I spend in {}?", name)).expect("spend question");
        assert_eq!(figure(&spend, "expenses"), exp, "expenses {}", name);
        let income = ask(&h, &format!("What was my total income in {}?", name)).expect("income question");
        assert_eq!(figure(&income, "income"), inc, "income {}", name);
        let net = ask(&h, &format!("What was the net cash flow for {}?", name)).expect("net question");
        assert_eq!(figure(&net, "net"), inc - exp, "net {}", name);
        assert!(spend.text.contains(&name), "the answer must state its period: {}", spend.text);
    }

    // 2. All time, a whole year, and a quarter.
    let (_, all_exp) = expected(&ledger, "USD", "", None);
    let a = ask(&h, "What are my total expenses?").unwrap();
    assert_eq!(figure(&a, "expenses"), all_exp);
    assert!(a.scope.contains("all dates"));

    let (_, y2025) = expected(&ledger, "USD", "2025", None);
    assert_eq!(figure(&ask(&h, "Total spending in 2025").unwrap(), "expenses"), y2025);

    let q: i64 = ["2025-04", "2025-05", "2025-06"].iter().map(|p| expected(&ledger, "USD", p, None).1).sum();
    assert_eq!(figure(&ask(&h, "What did we spend in Q2 2025?").unwrap(), "expenses"), q);

    // 3. Another currency: the answer follows the currency asked, and USD is reported as not included.
    let (_, eur_exp) = expected(&ledger, "EUR", "2025", None);
    let eur = ask(&h, "What were my EUR expenses in 2025?").unwrap();
    assert_eq!(figure(&eur, "expenses"), eur_exp);
    assert_eq!(eur.figures[0].currency, "EUR");
    assert!(eur.text.contains("different currency"), "other currencies must be reported: {}", eur.text);

    // 4. Base-currency answers disclose the transactions in other currencies instead of dropping them.
    let usd = ask(&h, "What were my expenses in 2025?").unwrap();
    assert!(usd.text.contains("EUR") && usd.text.contains("INR"), "excluded currencies named: {}", usd.text);

    // 5. A merchant named in the question narrows the scope.
    let (_, stream) = expected(&ledger, "USD", "", Some("streamline"));
    let m = ask(&h, "How much did I pay to Streamline in total?").unwrap();
    assert_eq!(figure(&m, "expenses"), stream);
    assert!(m.scope.contains("streamline"));

    // 6. A period with no data says so and states what the records cover.
    let none = ask(&h, "How much did I spend in January 2019?").unwrap();
    assert_eq!(figure(&none, "expenses"), 0);
    assert!(none.text.contains("No expenses") && none.text.contains("Your records cover"), "{}", none.text);

    // 7. Counts.
    let n = ask(&h, "How many transactions are there?").unwrap();
    assert!(n.text.contains(&format!("{} transactions", ledger["bank_row_count"])), "{}", n.text);

    // 8. Largest expenses are returned in descending order.
    let top = ask(&h, "What are my largest expenses?").unwrap();
    let minors: Vec<i64> = top.figures.iter().map(|f| f.minor).collect();
    assert!(minors.windows(2).all(|w| w[0] >= w[1]) && !minors.is_empty());

    // 9. Questions that cannot be computed from records get no verified answer.
    assert!(ask(&h, "Will I be profitable next year?").is_none());
    assert!(ask(&h, "Tell me a joke about accountants").is_none());
    assert!(ask(&h, "What colour is the sky?").is_none());
}

#[test]
#[ignore = "runs real ingestion over the corpus"]
fn invoice_and_duplicate_answers_equal_the_ledger() {
    let ledger: Value =
        serde_json::from_str(&fs::read_to_string(corpus_dir("workspace_alpha").join("ledger.json")).unwrap()).unwrap();
    let mut h = Harness::new(WS);

    // Ingest the invoices needed: the first invoice and its number clash, the second and its re-send.
    let invoices = ledger["invoices"].as_array().unwrap();
    let first = invoices[0]["number"].as_str().unwrap().to_string();
    let second = invoices[1]["number"].as_str().unwrap().to_string();
    let resend = format!("{}-R", second);
    for inv in invoices {
        let n = inv["number"].as_str().unwrap();
        if n == first || n == second || n == resend {
            h.ingest_file(WS, &corpus_dir("workspace_alpha").join("invoices").join(inv["file"].as_str().unwrap()));
        }
    }
    h.ingest_csv(WS, &fs::read_to_string(corpus_dir("workspace_alpha").join("bank_statement.csv")).unwrap());

    // Exact invoice total, vendor and currency from the record.
    let second_rec = &invoices[1];
    let a = ask(&h, &format!("What is the total of invoice {}?", second)).expect("invoice question");
    assert_eq!(a.figures[0].minor, second_rec["total_minor"].as_i64().unwrap());
    assert_eq!(a.figures[0].currency, second_rec["currency"].as_str().unwrap());
    assert!(a.text.contains(second_rec["vendor"].as_str().unwrap()));
    assert_eq!(a.citations.len(), 1);

    // A reused invoice number is reported as such.
    let reused = ask(&h, &format!("Show invoice {}", first)).unwrap();
    assert!(reused.text.contains("used by 2 invoices"), "{}", reused.text);

    // An invoice and its re-sent copy are described as the same vendor, currency and total.
    let pair = ask(&h, &format!("Are {} and {} the same bill?", second, resend)).unwrap();
    assert!(pair.text.contains("same vendor, currency and total"), "{}", pair.text);
    assert_eq!(pair.figures.len(), 2);

    // An unknown invoice is reported as absent, not invented.
    let missing = ask(&h, "What is the total of invoice ZZZ-9999-0001?").unwrap();
    assert!(missing.text.contains("No invoice numbered"), "{}", missing.text);
    assert!(missing.figures.is_empty());

    // Duplicates: the ledger's re-sent invoice and reused number are among the findings.
    let dup = ask(&h, "Are there any duplicate payments or invoices?").unwrap();
    assert!(dup.text.contains("suspected duplicate"), "{}", dup.text);
    assert!(!dup.citations.is_empty());

    // Unmatched debits: only debits, none that the matcher linked to an invoice.
    let um = ask(&h, "Which payments have no matching invoice?").unwrap();
    assert!(um.text.contains("no matching invoice"), "{}", um.text);
    assert!(um.figures[0].minor > 0);
}
