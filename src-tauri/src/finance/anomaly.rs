use crate::finance::matching::{calculate_date_difference, InvoiceCandidate, TransactionCandidate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinancialAnomaly {
    pub id: String,
    pub entity_type: String, // "INVOICE", "TRANSACTION"
    pub entity_id: String,
    pub anomaly_type: String, // "DUPLICATE_INVOICE", "DUPLICATE_TRANSACTION", "OVERDUE_UNPAID", "LARGE_UNMATCHED"
    pub description: String,
    pub severity: String, // "LOW", "MEDIUM", "HIGH"
}

pub struct AnomalyDetector;

/// Formats a minor-unit amount for human-readable anomaly text, e.g. "742.81 USD".
fn format_amount(minor: i64, currency: &str) -> String {
    crate::finance::money::Money::new(minor, currency)
        .map(|m| m.format_display())
        .unwrap_or_else(|_| format!("{} {}", minor, currency))
}

/// Largest gap, in days, between two debits that is still reviewed as a possible repeat payment.
const POTENTIAL_DUPLICATE_WINDOW_DAYS: i64 = 14;
/// Gaps in this range look like a monthly billing cycle and are not flagged.
const MONTHLY_CYCLE_DAYS: std::ops::RangeInclusive<i64> = 25..=35;

/// Normalized merchant key: the first alphanumeric word of the description, lowercased.
/// "NORTHWIND CLOUD LTD, INV 2026-0912" and "NORTHWIND CLOUD LTD" both map to "northwind".
fn merchant_key(description: &str) -> Option<String> {
    description
        .split(|c: char| !c.is_alphanumeric())
        .find(|w| w.chars().count() >= 3 && !w.chars().all(|c| c.is_ascii_digit()))
        .map(|w| w.to_lowercase())
}

fn days_between(a: &str, b: &str) -> Option<i64> {
    let a = chrono::NaiveDate::parse_from_str(a, "%Y-%m-%d").ok()?;
    let b = chrono::NaiveDate::parse_from_str(b, "%Y-%m-%d").ok()?;
    Some((a - b).num_days().abs())
}

impl AnomalyDetector {
    /// Flags pairs of distinct debits that may be an unintended repeat payment.
    ///
    /// Flags a pair only when all of these hold: both are debits, same currency and exact same
    /// amount, same normalized merchant, and posting dates within the review window. Pairs whose
    /// gap looks like a monthly cycle are left alone. Nothing is deleted or altered; the flag only
    /// asks for review.
    pub fn detect_potential_duplicate_payments(
        transactions: &[TransactionCandidate],
    ) -> Vec<FinancialAnomaly> {
        let debits: Vec<(&TransactionCandidate, Option<String>)> = transactions
            .iter()
            .filter(|t| t.amount_minor < 0)
            .map(|t| (t, merchant_key(&t.description)))
            .collect();

        let mut anomalies = Vec::new();
        for (i, (tx1, key1)) in debits.iter().enumerate() {
            for (tx2, key2) in debits.iter().skip(i + 1) {
                let (Some(k1), Some(k2)) = (key1, key2) else { continue };
                if k1 != k2
                    || tx1.currency != tx2.currency
                    || tx1.amount_minor != tx2.amount_minor
                {
                    continue;
                }
                let Some(gap) = days_between(&tx1.posted_date, &tx2.posted_date) else { continue };
                // Same day and same description is an imported duplicate, handled by the rule below.
                if gap == 0 && tx1.description.eq_ignore_ascii_case(&tx2.description) {
                    continue;
                }
                if gap > POTENTIAL_DUPLICATE_WINDOW_DAYS || MONTHLY_CYCLE_DAYS.contains(&gap) {
                    continue;
                }
                anomalies.push(FinancialAnomaly {
                    id: Uuid::new_v4().to_string(),
                    entity_type: "TRANSACTION".to_string(),
                    entity_id: tx2.id.clone(),
                    anomaly_type: "POTENTIAL_DUPLICATE_PAYMENT".to_string(),
                    description: format!(
                        "Two debits of {} to '{}' posted {} days apart ({} and {}). Review whether the second payment was intended.",
                        format_amount(tx2.amount_minor, &tx2.currency),
                        tx2.description,
                        gap,
                        tx1.posted_date,
                        tx2.posted_date
                    ),
                    severity: "MEDIUM".to_string(),
                });
            }
        }
        anomalies
    }

    pub fn detect_anomalies(
        invoices: &[InvoiceCandidate],
        transactions: &[TransactionCandidate],
    ) -> Vec<FinancialAnomaly> {
        let mut anomalies = Vec::new();

        // 1. Detect duplicate invoice candidates
        for (i, inv1) in invoices.iter().enumerate() {
            for inv2 in invoices.iter().skip(i + 1) {
                // If both have invoice numbers and they match, definite duplicate
                let has_same_inv_num = inv1.invoice_number.is_some()
                    && inv1.invoice_number == inv2.invoice_number
                    && inv1.vendor_name.to_lowercase() == inv2.vendor_name.to_lowercase();

                // If same vendor and exact same amount within 3 days: suspected duplicate billing
                let days_diff = calculate_date_difference(
                    inv1.invoice_date.as_deref(),
                    inv2.invoice_date.as_deref().unwrap_or(""),
                );
                let is_same_vendor =
                    inv1.vendor_name.to_lowercase() == inv2.vendor_name.to_lowercase();
                let is_same_amount =
                    inv1.total_minor == inv2.total_minor && inv1.currency == inv2.currency;

                if has_same_inv_num {
                    anomalies.push(FinancialAnomaly {
                        id: Uuid::new_v4().to_string(),
                        entity_type: "INVOICE".to_string(),
                        entity_id: inv2.id.clone(),
                        anomaly_type: "DUPLICATE_INVOICE".to_string(),
                        description: format!(
                            "Duplicate invoice number '{}' detected for vendor '{}' (matches invoice {}).",
                            inv1.invoice_number.as_deref().unwrap_or(""),
                            inv1.vendor_name,
                            inv1.id
                        ),
                        severity: "HIGH".to_string(),
                    });
                } else if is_same_vendor && is_same_amount && days_diff <= 3 {
                    anomalies.push(FinancialAnomaly {
                        id: Uuid::new_v4().to_string(),
                        entity_type: "INVOICE".to_string(),
                        entity_id: inv2.id.clone(),
                        anomaly_type: "SUSPECTED_DUPLICATE_BILLING".to_string(),
                        description: format!(
                            "Suspected duplicate billing: identical amount ({}) from '{}' within {} days.",
                            format_amount(inv1.total_minor, &inv1.currency), inv1.vendor_name, days_diff
                        ),
                        severity: "MEDIUM".to_string(),
                    });
                }
                // False-positive suppression: identical amounts across > 25 days (e.g. monthly SaaS) are NOT flagged as duplicates
            }
        }

        // 2a. Potential duplicate payments: two distinct debits that may be an unintended repeat.
        anomalies.extend(Self::detect_potential_duplicate_payments(transactions));

        // 2. Detect duplicate transaction records (e.g. statement double-import)
        for (i, tx1) in transactions.iter().enumerate() {
            for tx2 in transactions.iter().skip(i + 1) {
                if tx1.posted_date == tx2.posted_date
                    && tx1.amount_minor == tx2.amount_minor
                    && tx1.currency == tx2.currency
                    && tx1.description.to_lowercase() == tx2.description.to_lowercase()
                {
                    anomalies.push(FinancialAnomaly {
                        id: Uuid::new_v4().to_string(),
                        entity_type: "TRANSACTION".to_string(),
                        entity_id: tx2.id.clone(),
                        anomaly_type: "DUPLICATE_TRANSACTION".to_string(),
                        description: format!(
                            "Duplicate bank transaction detected on {}: '{}' for {}.",
                            tx1.posted_date, tx1.description, format_amount(tx1.amount_minor, &tx1.currency)
                        ),
                        severity: "HIGH".to_string(),
                    });
                }
            }
        }

        // 3. Large unmatched transactions (> $1,000 or equivalent without invoice)
        for tx in transactions {
            let abs_amount = tx.amount_minor.abs();
            if abs_amount >= 100000 {
                // $1,000.00
                anomalies.push(FinancialAnomaly {
                    id: Uuid::new_v4().to_string(),
                    entity_type: "TRANSACTION".to_string(),
                    entity_id: tx.id.clone(),
                    anomaly_type: "LARGE_UNMATCHED".to_string(),
                    description: format!(
                        "High-value posting ({}) requires documentation / receipt matching.",
                        format_amount(tx.amount_minor, &tx.currency)
                    ),
                    severity: "LOW".to_string(),
                });
            }
        }

        anomalies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_invoice_number_detected() {
        let invoices = vec![
            InvoiceCandidate {
                id: "inv-original".to_string(),
                vendor_name: "CloudHosting Corp".to_string(),
                invoice_number: Some("INV-88912".to_string()),
                invoice_date: Some("2026-09-01".to_string()),
                total_minor: 12000,
                currency: "USD".to_string(),
            },
            InvoiceCandidate {
                id: "inv-dup".to_string(),
                vendor_name: "CloudHosting Corp".to_string(),
                invoice_number: Some("INV-88912".to_string()),
                invoice_date: Some("2026-09-01".to_string()),
                total_minor: 12000,
                currency: "USD".to_string(),
            },
        ];

        let anomalies = AnomalyDetector::detect_anomalies(&invoices, &[]);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "DUPLICATE_INVOICE");
        assert_eq!(anomalies[0].severity, "HIGH");
    }

    #[test]
    fn test_recurring_monthly_subscription_suppresses_false_positive() {
        let invoices = vec![
            InvoiceCandidate {
                id: "inv-sept".to_string(),
                vendor_name: "GitHub Enterprise".to_string(),
                invoice_number: Some("GH-SEP".to_string()),
                invoice_date: Some("2026-09-01".to_string()),
                total_minor: 4200, // $42.00
                currency: "USD".to_string(),
            },
            InvoiceCandidate {
                id: "inv-oct".to_string(),
                vendor_name: "GitHub Enterprise".to_string(),
                invoice_number: Some("GH-OCT".to_string()),
                invoice_date: Some("2026-10-01".to_string()),
                total_minor: 4200, // $42.00 (same amount, 30 days later)
                currency: "USD".to_string(),
            },
        ];

        let anomalies = AnomalyDetector::detect_anomalies(&invoices, &[]);
        assert!(
            anomalies.is_empty(),
            "Monthly recurring charges must not be falsely flagged as duplicates"
        );
    }
}

#[cfg(test)]
mod potential_duplicate_tests {
    use super::*;

    fn debit(id: &str, date: &str, desc: &str, minor: i64) -> TransactionCandidate {
        TransactionCandidate {
            id: id.to_string(),
            posted_date: date.to_string(),
            description: desc.to_string(),
            amount_minor: -minor,
            currency: "USD".to_string(),
            external_ref: None,
        }
    }

    fn flagged(txs: &[TransactionCandidate]) -> Vec<String> {
        AnomalyDetector::detect_potential_duplicate_payments(txs)
            .into_iter()
            .map(|a| a.entity_id)
            .collect()
    }

    #[test]
    fn test_regression_two_742_81_debits_are_flagged_for_review() {
        // Regression: same amount, same merchant, 12 days apart, different descriptions.
        let txs = vec![
            debit("t1", "2026-09-13", "NORTHWIND CLOUD LTD, INV 2026-0912", 74281),
            debit("t2", "2026-09-25", "NORTHWIND CLOUD LTD", 74281),
        ];
        assert_eq!(flagged(&txs), vec!["t2".to_string()]);
    }

    #[test]
    fn test_monthly_subscription_is_not_flagged() {
        // Same merchant and amount about one billing cycle apart is a legitimate subscription.
        let txs = vec![
            debit("t1", "2026-08-01", "CLOUDHOST SUBSCRIPTION", 12000),
            debit("t2", "2026-09-01", "CLOUDHOST SUBSCRIPTION", 12000),
        ];
        assert!(flagged(&txs).is_empty());
    }

    #[test]
    fn test_different_merchant_is_not_flagged() {
        let txs = vec![
            debit("t1", "2026-09-13", "NORTHWIND CLOUD LTD", 74281),
            debit("t2", "2026-09-20", "BLUE HARBOR STATIONERY", 74281),
        ];
        assert!(flagged(&txs).is_empty());
    }

    #[test]
    fn test_different_amount_is_not_flagged() {
        let txs = vec![
            debit("t1", "2026-09-13", "NORTHWIND CLOUD LTD", 74281),
            debit("t2", "2026-09-20", "NORTHWIND CLOUD LTD", 74282),
        ];
        assert!(flagged(&txs).is_empty());
    }

    #[test]
    fn test_refund_credit_is_not_treated_as_a_repeat_debit() {
        let mut refund = debit("t2", "2026-09-18", "NORTHWIND CLOUD LTD REFUND", 5000);
        refund.amount_minor = 5000; // credit
        let txs = vec![debit("t1", "2026-09-13", "NORTHWIND CLOUD LTD", 5000), refund];
        assert!(flagged(&txs).is_empty());
    }

    #[test]
    fn test_outside_review_window_is_not_flagged() {
        let txs = vec![
            debit("t1", "2026-09-01", "NORTHWIND CLOUD LTD", 74281),
            debit("t2", "2026-09-20", "NORTHWIND CLOUD LTD", 74281),
        ];
        assert!(flagged(&txs).is_empty());
    }
}
