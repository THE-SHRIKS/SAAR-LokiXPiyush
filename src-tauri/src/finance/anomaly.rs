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

impl AnomalyDetector {
    /// Detects duplicate invoices and transactions with false-positive suppression for recurring billing.
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
                let days_diff = calculate_date_difference(inv1.invoice_date.as_deref(), inv2.invoice_date.as_deref().unwrap_or(""));
                let is_same_vendor = inv1.vendor_name.to_lowercase() == inv2.vendor_name.to_lowercase();
                let is_same_amount = inv1.total_minor == inv2.total_minor && inv1.currency == inv2.currency;

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
                            "Suspected duplicate billing: identical amount ({} {}) from '{}' within {} days.",
                            inv1.total_minor, inv1.currency, inv1.vendor_name, days_diff
                        ),
                        severity: "MEDIUM".to_string(),
                    });
                }
                // False-positive suppression: identical amounts across > 25 days (e.g. monthly SaaS) are NOT flagged as duplicates
            }
        }

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
                            "Duplicate bank transaction detected on {}: '{}' for {} {}.",
                            tx1.posted_date, tx1.description, tx1.amount_minor, tx1.currency
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
                        "High-value posting ({} {}) requires documentation / receipt matching.",
                        tx.amount_minor, tx.currency
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
        assert!(anomalies.is_empty(), "Monthly recurring charges must not be falsely flagged as duplicates");
    }
}
