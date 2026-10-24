use crate::finance::matching::TransactionCandidate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MonthAggregate {
    pub month_key: String, // e.g. "2026-09"
    pub total_income_minor: i64,
    pub total_expense_minor: i64,
    pub net_cash_flow_minor: i64,
    pub currency: String,
    pub transaction_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinancialSummary {
    pub total_income_minor: i64,
    pub total_expense_minor: i64,
    pub net_savings_minor: i64,
    pub currency: String,
    pub monthly_breakdown: Vec<MonthAggregate>,
    pub mom_expense_change_percent: Option<f64>,
    /// Transactions left out of these totals because their currency differs from the target.
    /// Reported so that nothing is dropped silently. Amounts are gross, in each currency's minor units.
    pub excluded_currencies: Vec<CurrencyExclusion>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyExclusion {
    pub currency: String,
    pub transaction_count: usize,
    pub gross_minor: i64,
}

pub struct FinancialAggregator;

impl FinancialAggregator {
    /// Computes deterministic aggregates from a list of normalized transactions.
    pub fn compute_summary(
        transactions: &[TransactionCandidate],
        target_currency: &str,
    ) -> FinancialSummary {
        let mut monthly_map: BTreeMap<String, (i64, i64, usize)> = BTreeMap::new();
        let mut total_income = 0i64;
        let mut total_expense = 0i64;
        let mut excluded: BTreeMap<String, (usize, i64)> = BTreeMap::new();

        for tx in transactions {
            // Aggregate only matching currency to avoid mixed currency corruption
            if tx.currency != target_currency {
                let entry = excluded.entry(tx.currency.clone()).or_insert((0usize, 0i64));
                entry.0 += 1;
                entry.1 += tx.amount_minor.abs();
                continue;
            }

            let month_key = if tx.posted_date.len() >= 7 {
                tx.posted_date[..7].to_string()
            } else {
                "UNKNOWN".to_string()
            };

            let entry = monthly_map.entry(month_key).or_insert((0, 0, 0));
            entry.2 += 1; // Count

            if tx.amount_minor >= 0 {
                // Income
                entry.0 += tx.amount_minor;
                total_income += tx.amount_minor;
            } else {
                // Expense (stored as negative, convert to positive magnitude for expense aggregate)
                let abs_exp = tx.amount_minor.abs();
                entry.1 += abs_exp;
                total_expense += abs_exp;
            }
        }

        let mut monthly_breakdown = Vec::new();
        for (m_key, (inc, exp, cnt)) in monthly_map {
            let net = inc - exp;
            monthly_breakdown.push(MonthAggregate {
                month_key: m_key,
                total_income_minor: inc,
                total_expense_minor: exp,
                net_cash_flow_minor: net,
                currency: target_currency.to_string(),
                transaction_count: cnt,
            });
        }

        // Compute Month-on-Month expense change between the last two recorded months
        let mom_change = if monthly_breakdown.len() >= 2 {
            let prev_exp = monthly_breakdown[monthly_breakdown.len() - 2].total_expense_minor;
            let curr_exp = monthly_breakdown[monthly_breakdown.len() - 1].total_expense_minor;
            if prev_exp > 0 {
                let change = ((curr_exp - prev_exp) as f64 / prev_exp as f64) * 100.0;
                Some((change * 10.0).round() / 10.0)
            } else {
                None
            }
        } else {
            None
        };

        let net_savings = total_income - total_expense;

        FinancialSummary {
            total_income_minor: total_income,
            total_expense_minor: total_expense,
            net_savings_minor: net_savings,
            currency: target_currency.to_string(),
            monthly_breakdown,
            mom_expense_change_percent: mom_change,
            excluded_currencies: excluded
                .into_iter()
                .map(|(currency, (transaction_count, gross_minor))| CurrencyExclusion {
                    currency,
                    transaction_count,
                    gross_minor,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregates_independent_hand_calculation() {
        // Hand calculation reference:
        // Transactions:
        // 1. 2026-08-15: Income $3,000.00 (+300000)
        // 2. 2026-08-20: Expense -$500.00 (-50000)
        // Month 2026-08: Income = 300000, Expense = 50000, Net = +250000
        //
        // 3. 2026-09-01: Expense -$120.00 (-12000)
        // 4. 2026-09-10: Expense -$45.50 (-4550)
        // 5. 2026-09-15: Income $5,000.00 (+500000)
        // Month 2026-09: Income = 500000, Expense = 16550, Net = +483450
        //
        // Total Income: 300000 + 500000 = 800000 ($8,000.00)
        // Total Expense: 50000 + 16550 = 66550 ($665.50)
        // Net Savings: 800000 - 66550 = 733450 ($7,334.50)
        // MoM Expense Change: (16550 - 50000) / 50000 = -33450 / 50000 = -66.9%

        let txs = vec![
            TransactionCandidate {
                id: "tx-1".to_string(),
                posted_date: "2026-08-15".to_string(),
                description: "Client Advance".to_string(),
                amount_minor: 300000,
                currency: "USD".to_string(),
                external_ref: None,
            },
            TransactionCandidate {
                id: "tx-2".to_string(),
                posted_date: "2026-08-20".to_string(),
                description: "Equipment Purchase".to_string(),
                amount_minor: -50000,
                currency: "USD".to_string(),
                external_ref: None,
            },
            TransactionCandidate {
                id: "tx-3".to_string(),
                posted_date: "2026-09-01".to_string(),
                description: "Cloud Hosting".to_string(),
                amount_minor: -12000,
                currency: "USD".to_string(),
                external_ref: None,
            },
            TransactionCandidate {
                id: "tx-4".to_string(),
                posted_date: "2026-09-10".to_string(),
                description: "Office Supplies".to_string(),
                amount_minor: -4550,
                currency: "USD".to_string(),
                external_ref: None,
            },
            TransactionCandidate {
                id: "tx-5".to_string(),
                posted_date: "2026-09-15".to_string(),
                description: "Consulting Fee".to_string(),
                amount_minor: 500000,
                currency: "USD".to_string(),
                external_ref: None,
            },
        ];

        let summary = FinancialAggregator::compute_summary(&txs, "USD");

        assert_eq!(summary.total_income_minor, 800000);
        assert_eq!(summary.total_expense_minor, 66550);
        assert_eq!(summary.net_savings_minor, 733450);
        assert_eq!(summary.monthly_breakdown.len(), 2);

        assert_eq!(summary.monthly_breakdown[0].month_key, "2026-08");
        assert_eq!(summary.monthly_breakdown[0].total_income_minor, 300000);
        assert_eq!(summary.monthly_breakdown[0].total_expense_minor, 50000);
        assert_eq!(summary.monthly_breakdown[0].net_cash_flow_minor, 250000);

        assert_eq!(summary.monthly_breakdown[1].month_key, "2026-09");
        assert_eq!(summary.monthly_breakdown[1].total_income_minor, 500000);
        assert_eq!(summary.monthly_breakdown[1].total_expense_minor, 16550);
        assert_eq!(summary.monthly_breakdown[1].net_cash_flow_minor, 483450);

        assert_eq!(summary.mom_expense_change_percent, Some(-66.9));
    }
}

#[cfg(test)]
mod excluded_currency_tests {
    use super::*;

    fn tx(id: &str, amount: i64, currency: &str) -> TransactionCandidate {
        TransactionCandidate {
            id: id.to_string(),
            posted_date: "2026-09-01".to_string(),
            description: "X".to_string(),
            amount_minor: amount,
            currency: currency.to_string(),
            external_ref: None,
        }
    }

    #[test]
    fn test_other_currencies_are_excluded_from_totals_but_reported() {
        let txs = vec![
            tx("u1", -1000, "USD"),
            tx("u2", 500, "USD"),
            tx("e1", -120000, "EUR"),
            tx("e2", 9999, "EUR"),
            tx("i1", -1500050, "INR"),
        ];
        let summary = FinancialAggregator::compute_summary(&txs, "USD");
        // Totals cover USD only.
        assert_eq!(summary.total_expense_minor, 1000);
        assert_eq!(summary.total_income_minor, 500);
        // Excluded currencies are reported with exact counts and gross minor units.
        assert_eq!(
            summary.excluded_currencies,
            vec![
                CurrencyExclusion { currency: "EUR".into(), transaction_count: 2, gross_minor: 129999 },
                CurrencyExclusion { currency: "INR".into(), transaction_count: 1, gross_minor: 1500050 },
            ]
        );
    }

    #[test]
    fn test_no_exclusions_when_all_rows_match_target() {
        let txs = vec![tx("u1", -1000, "USD")];
        let summary = FinancialAggregator::compute_summary(&txs, "USD");
        assert!(summary.excluded_currencies.is_empty());
    }
}

/// Invoices (bills) imported into a workspace, per currency. Bank totals come from transactions
/// only, so this is what shows that imported bills were read even before a bank statement exists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceCurrencySummary {
    pub currency: String,
    pub invoice_count: i64,
    pub total_minor: i64,
    /// Invoices with no proposed or approved bank payment link.
    pub unmatched_count: i64,
    pub unmatched_total_minor: i64,
}

pub fn invoice_summary(
    conn: &rusqlite::Connection,
    workspace_id: &str,
) -> rusqlite::Result<Vec<InvoiceCurrencySummary>> {
    let mut stmt = conn.prepare(
        "SELECT i.currency,
                COUNT(*),
                COALESCE(SUM(i.total_minor), 0),
                COALESCE(SUM(CASE WHEN l.invoice_id IS NULL THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN l.invoice_id IS NULL THEN i.total_minor ELSE 0 END), 0)
         FROM invoices i
         JOIN documents d ON d.id = i.document_id
         LEFT JOIN (SELECT DISTINCT invoice_id FROM invoice_payment_links
                    WHERE workspace_id = ?1 AND status IN ('PROPOSED', 'APPROVED')) l
                ON l.invoice_id = i.id
         WHERE d.workspace_id = ?1
         GROUP BY i.currency
         ORDER BY i.currency",
    )?;
    let rows = stmt.query_map([workspace_id], |r| {
        Ok(InvoiceCurrencySummary {
            currency: r.get(0)?,
            invoice_count: r.get(1)?,
            total_minor: r.get(2)?,
            unmatched_count: r.get(3)?,
            unmatched_total_minor: r.get(4)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod invoice_summary_tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_invoice_summary_counts_by_currency_and_tracks_unmatched() {
        let db = Database::open_in_memory().unwrap();
        db.create_workspace("ws", "WS", "USD", &[1u8; 16], &[2u8; 32]).unwrap();
        let c = db.conn_ref();
        for (doc, inv, cur, total) in [("d1", "i1", "USD", 10_000i64), ("d2", "i2", "USD", 2_500), ("d3", "i3", "INR", 700_000)] {
            c.execute(
                "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, status, imported_at)
                 VALUES (?1, 'ws', ?1, 1, ?1, 'INVOICE', 'READY', '2026-01-01')",
                [doc],
            ).unwrap();
            c.execute(
                "INSERT INTO invoices (id, document_id, vendor_name, currency, total_minor) VALUES (?1, ?2, 'V', ?3, ?4)",
                rusqlite::params![inv, doc, cur, total],
            ).unwrap();
        }
        c.execute(
            "INSERT INTO transactions (id, workspace_id, posted_date, description, amount_minor, currency, review_state)
             VALUES ('t1', 'ws', '2026-01-02', 'PAY', -10000, 'USD', 'UNREVIEWED')",
            [],
        ).unwrap();
        c.execute(
            "INSERT INTO invoice_payment_links (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
             VALUES ('l1', 'ws', 'i1', 't1', 10000, 1.0, 'EXACT', 'x', 'APPROVED', '2026-01-02')",
            [],
        ).unwrap();
        let s = invoice_summary(c, "ws").unwrap();
        assert_eq!(s.len(), 2);
        let usd = &s[1];
        assert_eq!((usd.currency.as_str(), usd.invoice_count, usd.total_minor), ("USD", 2, 12_500));
        assert_eq!((usd.unmatched_count, usd.unmatched_total_minor), (1, 2_500));
        let inr = &s[0];
        assert_eq!((inr.currency.as_str(), inr.invoice_count, inr.unmatched_count), ("INR", 1, 1));
    }
}
