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

/// True when `reference` appears in `text` as a whole token. `INV-2025-0002` is found in
/// "ACME INV-2025-0002 PART 1" but not inside "ACME INV-2025-0002-R", which is a different document.
fn contains_reference_token(text: &str, reference: &str) -> bool {
    let hay = text.to_lowercase();
    let needle = reference.to_lowercase();
    let is_word = |c: char| c.is_alphanumeric() || c == '-' || c == '_';
    let mut start = 0;
    while let Some(pos) = hay[start..].find(&needle) {
        let begin = start + pos;
        let end = begin + needle.len();
        let before_ok = hay[..begin].chars().next_back().map_or(true, |c| !is_word(c));
        let after_ok = hay[end..].chars().next().map_or(true, |c| !is_word(c));
        if before_ok && after_ok {
            return true;
        }
        start = begin + needle.len().max(1);
        if start >= hay.len() {
            break;
        }
    }
    false
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

        // Pass 0: invoice number as a whole token in the payment reference, with exact amount and
        // currency. This is the strongest signal, so it runs first and keeps near-identical
        // invoices (for example an invoice and its re-sent copy) from taking each other's payments.
        for inv in invoices {
            let Some(number) = inv.invoice_number.as_deref().filter(|n| !n.is_empty()) else {
                continue;
            };
            for tx in transactions {
                if matched_transactions.contains(&tx.id)
                    || inv.currency != tx.currency
                    || inv.total_minor != tx.amount_minor.abs()
                {
                    continue;
                }
                let referenced = contains_reference_token(&tx.description, number)
                    || tx
                        .external_ref
                        .as_deref()
                        .map(|r| contains_reference_token(r, number))
                        .unwrap_or(false);
                if !referenced {
                    continue;
                }
                let days_diff = calculate_date_difference(inv.invoice_date.as_deref(), &tx.posted_date);
                let date_known = inv.invoice_date.is_some();
                if date_known && days_diff > 45 {
                    continue;
                }
                links.push(ProposedLink {
                    id: Uuid::new_v4().to_string(),
                    invoice_id: inv.id.clone(),
                    transaction_id: tx.id.clone(),
                    matched_amount_minor: inv.total_minor,
                    confidence_score: if date_known { 1.0 } else { 0.9 },
                    match_type: if date_known { "EXACT" } else { "EXACT_NO_DATE" }.to_string(),
                    explanation: if date_known {
                        format!(
                            "Exact match on amount ({} {}) and invoice number {} within {} days.",
                            inv.total_minor, inv.currency, number, days_diff
                        )
                    } else {
                        format!(
                            "Matched on amount ({} {}) and invoice number {}, but the invoice has no date; verify the timing.",
                            inv.total_minor, inv.currency, number
                        )
                    },
                    status: "PROPOSED".to_string(),
                });
                matched_invoices.insert(inv.id.clone());
                matched_transactions.insert(tx.id.clone());
                break;
            }
        }

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

#[cfg(test)]
mod reference_matching_tests {
    use super::*;

    fn inv(id: &str, number: &str, date: Option<&str>, total: i64) -> InvoiceCandidate {
        InvoiceCandidate {
            id: id.to_string(),
            vendor_name: "OAKWOOD SUPPLY LTD".to_string(),
            invoice_number: Some(number.to_string()),
            invoice_date: date.map(|d| d.to_string()),
            total_minor: total,
            currency: "USD".to_string(),
        }
    }

    fn tx(id: &str, date: &str, desc: &str, minor: i64) -> TransactionCandidate {
        TransactionCandidate {
            id: id.to_string(),
            posted_date: date.to_string(),
            description: desc.to_string(),
            amount_minor: minor,
            currency: "USD".to_string(),
            external_ref: None,
        }
    }

    #[test]
    fn test_reference_token_respects_boundaries() {
        assert!(contains_reference_token("OAKWOOD INV-2025-0002 PART 1", "INV-2025-0002"));
        assert!(contains_reference_token("oakwood inv-2025-0002", "INV-2025-0002"));
        assert!(!contains_reference_token("OAKWOOD INV-2025-0002-R", "INV-2025-0002"));
        assert!(!contains_reference_token("OAKWOOD XINV-2025-0002", "INV-2025-0002"));
    }

    #[test]
    fn test_invoice_without_a_date_still_matches_on_amount_and_number() {
        let links = ReconciliationMatcher::match_records(
            &[inv("i1", "INV-7", None, 50_000)],
            &[tx("t1", "2026-03-05", "OAKWOOD INV-7", -50_000)],
        );
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].match_type, "EXACT_NO_DATE");
        assert!(links[0].confidence_score < 1.0, "a missing date must lower confidence");
        assert!(links[0].explanation.contains("no date"));
    }

    #[test]
    fn test_identical_invoices_take_their_own_payments() {
        // Same vendor, same amount: an invoice and its re-sent copy. The earlier payment mentions
        // the copy, which must not be taken by the original.
        let invoices = [
            inv("orig", "INV-100", Some("2026-03-01"), 80_000),
            inv("copy", "INV-100-R", Some("2026-03-03"), 80_000),
        ];
        let txs = [
            tx("pay_copy", "2026-03-08", "OAKWOOD INV-100-R", -80_000),
            tx("pay_orig", "2026-03-12", "OAKWOOD INV-100", -80_000),
        ];
        let links = ReconciliationMatcher::match_records(&invoices, &txs);
        let for_invoice = |id: &str| links.iter().find(|l| l.invoice_id == id).unwrap().transaction_id.clone();
        assert_eq!(for_invoice("orig"), "pay_orig");
        assert_eq!(for_invoice("copy"), "pay_copy");
    }

    #[test]
    fn test_wrong_currency_never_links_even_with_a_matching_reference() {
        let mut t = tx("t1", "2026-03-05", "OAKWOOD INV-7", -50_000);
        t.currency = "EUR".to_string();
        let links = ReconciliationMatcher::match_records(&[inv("i1", "INV-7", Some("2026-03-01"), 50_000)], &[t]);
        assert!(links.is_empty());
    }
}
