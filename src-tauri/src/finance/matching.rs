use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProposedLink {
    pub id: String,
    pub invoice_id: String,
    pub transaction_id: String,
    pub matched_amount_minor: i64,
    pub confidence_score: f64,
    pub match_type: String, // "EXACT", "PARTIAL", "SPLIT", "FUZZY_REF"
    pub explanation: String,
    pub status: String, // "PROPOSED", "APPROVED", "REJECTED"
}

#[derive(Debug, Clone)]
pub struct InvoiceCandidate {
    pub id: String,
    pub vendor_name: String,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub total_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct TransactionCandidate {
    pub id: String,
    pub posted_date: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub external_ref: Option<String>,
}

pub struct ReconciliationMatcher;

impl ReconciliationMatcher {
    /// Reconciles invoices against transactions using deterministic matching rules.
    pub fn match_records(
        invoices: &[InvoiceCandidate],
        transactions: &[TransactionCandidate],
    ) -> Vec<ProposedLink> {
        let mut links = Vec::new();
        let mut matched_invoices = std::collections::HashSet::new();
        let mut matched_transactions = std::collections::HashSet::new();

        // Pass 1: Exact matches (Amount + Currency + Reference Token/Vendor in Description)
        for inv in invoices {
            if matched_invoices.contains(&inv.id) {
                continue;
            }

            for tx in transactions {
                if matched_transactions.contains(&tx.id) {
                    continue;
                }

                // Check currency match
                if inv.currency != tx.currency {
                    continue;
                }

                // Check amount match (Invoice total == abs(transaction debit))
                let tx_abs_minor = tx.amount_minor.abs();
                if inv.total_minor == tx_abs_minor {
                    let desc_lower = tx.description.to_lowercase();
                    let vendor_lower = inv.vendor_name.to_lowercase();
                    let inv_num_lower = inv
                        .invoice_number
                        .as_ref()
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();

                    let has_vendor_ref = !vendor_lower.is_empty()
                        && (desc_lower.contains(&vendor_lower)
                            || vendor_lower
                                .split_whitespace()
                                .any(|word| word.len() > 3 && desc_lower.contains(word)));

                    let has_inv_num_ref = !inv_num_lower.is_empty()
                        && (desc_lower.contains(&inv_num_lower)
                            || tx
                                .external_ref
                                .as_ref()
                                .map(|r| r.to_lowercase().contains(&inv_num_lower))
                                .unwrap_or(false));

                    let days_diff =
                        calculate_date_difference(inv.invoice_date.as_deref(), &tx.posted_date);

                    if (has_vendor_ref || has_inv_num_ref) && days_diff <= 45 {
                        links.push(ProposedLink {
                            id: Uuid::new_v4().to_string(),
                            invoice_id: inv.id.clone(),
                            transaction_id: tx.id.clone(),
                            matched_amount_minor: inv.total_minor,
                            confidence_score: 1.0,
                            match_type: "EXACT".to_string(),
                            explanation: format!(
                                "Exact match on amount ({} {}) and reference ({}) within {} days.",
                                inv.total_minor,
                                inv.currency,
                                if has_inv_num_ref {
                                    "invoice number"
                                } else {
                                    "vendor name"
                                },
                                days_diff
                            ),
                            status: "PROPOSED".to_string(),
                        });

                        matched_invoices.insert(inv.id.clone());
                        matched_transactions.insert(tx.id.clone());
                        break;
                    }
                }
            }
        }

        // Pass 2: Exact Amount match within 15 days (even if text is slightly differing)
        for inv in invoices {
            if matched_invoices.contains(&inv.id) {
                continue;
            }

            for tx in transactions {
                if matched_transactions.contains(&tx.id) {
                    continue;
                }

                if inv.currency == tx.currency && inv.total_minor == tx.amount_minor.abs() {
                    let days_diff =
                        calculate_date_difference(inv.invoice_date.as_deref(), &tx.posted_date);
                    if days_diff <= 15 {
                        links.push(ProposedLink {
                            id: Uuid::new_v4().to_string(),
                            invoice_id: inv.id.clone(),
                            transaction_id: tx.id.clone(),
                            matched_amount_minor: inv.total_minor,
                            confidence_score: 0.85,
                            match_type: "EXACT_AMOUNT_DATE".to_string(),
                            explanation: format!(
                                "Exact amount ({} {}) match within {} days; verify vendor description manually.",
                                inv.total_minor, inv.currency, days_diff
                            ),
                            status: "PROPOSED".to_string(),
                        });

                        matched_invoices.insert(inv.id.clone());
                        matched_transactions.insert(tx.id.clone());
                        break;
                    }
                }
            }
        }

        // Pass 3: Amount Mismatch explanations (Vendor matches, but amount differs due to fee/discount)
        for inv in invoices {
            if matched_invoices.contains(&inv.id) {
                continue;
            }

            for tx in transactions {
                if matched_transactions.contains(&tx.id) {
                    continue;
                }

                if inv.currency == tx.currency {
                    let desc_lower = tx.description.to_lowercase();
                    let vendor_lower = inv.vendor_name.to_lowercase();
                    let inv_num_lower = inv
                        .invoice_number
                        .as_ref()
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();

                    let has_ref = (!inv_num_lower.is_empty()
                        && desc_lower.contains(&inv_num_lower))
                        || (!vendor_lower.is_empty() && desc_lower.contains(&vendor_lower));

                    if has_ref {
                        let days_diff =
                            calculate_date_difference(inv.invoice_date.as_deref(), &tx.posted_date);
                        if days_diff <= 30 {
                            let diff_minor = tx.amount_minor.abs() - inv.total_minor;
                            links.push(ProposedLink {
                                id: Uuid::new_v4().to_string(),
                                invoice_id: inv.id.clone(),
                                transaction_id: tx.id.clone(),
                                matched_amount_minor: tx.amount_minor.abs(),
                                confidence_score: 0.70,
                                match_type: "AMOUNT_MISMATCH".to_string(),
                                explanation: format!(
                                    "Reference matched, but amounts differ: Invoice {} vs Transaction {} (Discrepancy: {} {}).",
                                    inv.total_minor,
                                    tx.amount_minor.abs(),
                                    diff_minor,
                                    inv.currency
                                ),
                                status: "PROPOSED".to_string(),
                            });

                            matched_invoices.insert(inv.id.clone());
                            matched_transactions.insert(tx.id.clone());
                            break;
                        }
                    }
                }
            }
        }

        links
    }
}

/// Helper calculating absolute day difference between two date strings
pub fn calculate_date_difference(d1_opt: Option<&str>, d2_str: &str) -> i64 {
    let d2 = match NaiveDate::parse_from_str(d2_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return 9999,
    };

    if let Some(d1_str) = d1_opt {
        if let Ok(d1) = NaiveDate::parse_from_str(d1_str, "%Y-%m-%d") {
            return (d1 - d2).num_days().abs();
        }
    }

    9999
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_rule() {
        let invoices = vec![InvoiceCandidate {
            id: "inv-1".to_string(),
            vendor_name: "CloudHosting Corp".to_string(),
            invoice_number: Some("INV-88912".to_string()),
            invoice_date: Some("2026-09-01".to_string()),
            total_minor: 12000, // $120.00
            currency: "USD".to_string(),
        }];

        let transactions = vec![TransactionCandidate {
            id: "tx-1".to_string(),
            posted_date: "2026-09-02".to_string(),
            description: "CloudHosting Corp INV-88912".to_string(),
            amount_minor: -12000, // -$120.00
            currency: "USD".to_string(),
            external_ref: None,
        }];

        let links = ReconciliationMatcher::match_records(&invoices, &transactions);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].match_type, "EXACT");
        assert_eq!(links[0].confidence_score, 1.0);
        assert_eq!(links[0].matched_amount_minor, 12000);
    }

    #[test]
    fn test_mismatch_explanation_rule() {
        let invoices = vec![InvoiceCandidate {
            id: "inv-gst".to_string(),
            vendor_name: "ACME Tech Services".to_string(),
            invoice_number: Some("ACME-2026-0915".to_string()),
            invoice_date: Some("2026-09-15".to_string()),
            total_minor: 11800000, // 118,000.00 INR
            currency: "INR".to_string(),
        }];

        // Bank debited with a TDS withholding deduction: 115,000.00 INR
        let transactions = vec![TransactionCandidate {
            id: "tx-gst".to_string(),
            posted_date: "2026-09-16".to_string(),
            description: "NEFT-ACME TECH SERVICES-ACME-2026-0915".to_string(),
            amount_minor: 11500000, // 115,000.00 INR
            currency: "INR".to_string(),
            external_ref: Some("N246019283".to_string()),
        }];

        let links = ReconciliationMatcher::match_records(&invoices, &transactions);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].match_type, "AMOUNT_MISMATCH");
        assert!(links[0].explanation.contains("Discrepancy: -300000 INR"));
    }
}
