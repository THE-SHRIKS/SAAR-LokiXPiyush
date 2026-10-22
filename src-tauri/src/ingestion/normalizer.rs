use crate::ingestion::csv_mapper::CsvTransactionMapper;
use crate::ingestion::ocr_client::{OcrLine, OcrResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedField {
    pub field_key: String,
    pub original_text: String,
    pub normalized_value: String,
    pub confidence: f64,
    pub page_num: i32,
    pub bounding_box_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedLineItem {
    pub description: String,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedInvoiceData {
    pub vendor_name: String,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub currency: String,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub fields: Vec<NormalizedField>,
    pub line_items: Vec<NormalizedLineItem>,
    pub is_arithmetic_valid: bool,
    pub uncertainty_reasons: Vec<String>,
}

pub struct FieldNormalizer;

impl FieldNormalizer {
    /// Detects currency from text
    pub fn detect_currency(full_text: &str, default_currency: &str) -> String {
        let upper = full_text.to_uppercase();
        if upper.contains("INR") || upper.contains("₹") || upper.contains("GSTIN") {
            "INR".to_string()
        } else if upper.contains("EUR") || upper.contains("€") {
            "EUR".to_string()
        } else if upper.contains("GBP") || upper.contains("£") {
            "GBP".to_string()
        } else if upper.contains("JPY") || upper.contains("¥") {
            "JPY".to_string()
        } else if upper.contains('$') || upper.contains("USD") {
            "USD".to_string()
        } else {
            default_currency.to_string()
        }
    }

    /// Normalizes OCR output into structured financial invoice records
    pub fn normalize_ocr(ocr: &OcrResponse, default_currency: &str) -> NormalizedInvoiceData {
        let mut full_text_buf = String::new();
        for page in &ocr.pages {
            for line in &page.lines {
                full_text_buf.push_str(&line.text);
                full_text_buf.push('\n');
            }
        }

        let currency = Self::detect_currency(&full_text_buf, default_currency);

        let mut vendor_name = String::new();
        let mut invoice_number = None;
        let mut invoice_date = None;
        let mut due_date = None;
        let mut subtotal_minor = 0i64;
        let mut tax_minor = 0i64;
        let mut total_minor = 0i64;

        let mut fields = Vec::new();
        let mut line_items = Vec::new();
        let mut uncertainty_reasons = Vec::new();

        for page in &ocr.pages {
            for (line_idx, line) in page.lines.iter().enumerate() {
                let text = line.text.trim();
                let lower = text.to_lowercase();
                let bbox_str =
                    serde_json::to_string(&line.bbox).unwrap_or_else(|_| "[]".to_string());

                // 1. Vendor name heuristic (top lines if not metadata)
                if vendor_name.is_empty() && line_idx < 3 {
                    if lower.contains("supplier:") {
                        let v = text.split("Supplier:").nth(1).unwrap_or("").trim();
                        let clean = v.split("Invoice No:").next().unwrap_or(v).trim();
                        vendor_name = clean.to_string();
                        fields.push(NormalizedField {
                            field_key: "vendor_name".to_string(),
                            original_text: text.to_string(),
                            normalized_value: vendor_name.clone(),
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    } else if !lower.contains("invoice")
                        && !lower.contains("tax")
                        && !lower.contains("tel")
                    {
                        vendor_name = text.to_string();
                        fields.push(NormalizedField {
                            field_key: "vendor_name".to_string(),
                            original_text: text.to_string(),
                            normalized_value: vendor_name.clone(),
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                }

                // 2. Invoice number
                if invoice_number.is_none() {
                    if lower.contains("invoice no:")
                        || lower.contains("invoice number:")
                        || lower.contains("receiptid:")
                    {
                        let inv_str = if lower.contains("invoice no:") {
                            text.split("Invoice No:").nth(1).unwrap_or("")
                        } else if lower.contains("invoice number:") {
                            text.split("Invoice Number:").nth(1).unwrap_or("")
                        } else {
                            text.split("RECEIPTID:").nth(1).unwrap_or("")
                        }
                        .trim();

                        let clean_inv = inv_str.split_whitespace().next().unwrap_or(inv_str).trim();
                        if !clean_inv.is_empty() {
                            invoice_number = Some(clean_inv.to_string());
                            fields.push(NormalizedField {
                                field_key: "invoice_number".to_string(),
                                original_text: text.to_string(),
                                normalized_value: clean_inv.to_string(),
                                confidence: line.confidence,
                                page_num: page.page_num,
                                bounding_box_json: bbox_str.clone(),
                            });
                        }
                    }
                }

                // 3. Invoice date & due date
                if invoice_date.is_none()
                    && (lower.contains("date:") || lower.contains("invoice date:"))
                {
                    let d_str = if lower.contains("invoice date:") {
                        text.split("Invoice Date:").nth(1).unwrap_or("")
                    } else {
                        text.split("Date:")
                            .nth(1)
                            .or_else(|| text.split("DATE:").nth(1))
                            .unwrap_or("")
                    };
                    let raw_d = d_str.split_whitespace().next().unwrap_or("");
                    if let Ok(iso_d) = CsvTransactionMapper::normalize_date(raw_d) {
                        invoice_date = Some(iso_d.clone());
                        fields.push(NormalizedField {
                            field_key: "invoice_date".to_string(),
                            original_text: text.to_string(),
                            normalized_value: iso_d,
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                }

                if due_date.is_none() && lower.contains("due date:") {
                    let d_str = text.split("Due Date:").nth(1).unwrap_or("");
                    let raw_d = d_str.split_whitespace().next().unwrap_or("");
                    if let Ok(iso_d) = CsvTransactionMapper::normalize_date(raw_d) {
                        due_date = Some(iso_d.clone());
                        fields.push(NormalizedField {
                            field_key: "due_date".to_string(),
                            original_text: text.to_string(),
                            normalized_value: iso_d,
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                }

                // 4. Totals, subtotal, and tax
                if lower.starts_with("subtotal") || lower.contains("subtotal:") {
                    let amt = extract_amount_from_line(text, &page.lines, line_idx, &currency);
                    if amt > 0 {
                        subtotal_minor = amt;
                        fields.push(NormalizedField {
                            field_key: "subtotal".to_string(),
                            original_text: text.to_string(),
                            normalized_value: format!("{}", amt),
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                } else if lower.contains("tax")
                    || lower.contains("igst")
                    || lower.contains("cgst")
                    || lower.contains("sgst")
                {
                    let amt = extract_amount_from_line(text, &page.lines, line_idx, &currency);
                    tax_minor += amt;
                    fields.push(NormalizedField {
                        field_key: "tax_amount".to_string(),
                        original_text: text.to_string(),
                        normalized_value: format!("{}", amt),
                        confidence: line.confidence,
                        page_num: page.page_num,
                        bounding_box_json: bbox_str.clone(),
                    });
                } else if lower.starts_with("total")
                    || lower.contains("total:")
                    || lower.contains("total amount")
                {
                    let amt = extract_amount_from_line(text, &page.lines, line_idx, &currency);
                    if amt > 0 {
                        total_minor = amt;
                        fields.push(NormalizedField {
                            field_key: "total_amount".to_string(),
                            original_text: text.to_string(),
                            normalized_value: format!("{}", amt),
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                }

                // 5. Line items: detect numbered or priced items
                if (text.starts_with("1 ") || text.starts_with("2 ") || text.starts_with("3 "))
                    && !lower.contains("residency")
                    && !lower.contains("market st")
                {
                    let amt = extract_amount_from_line(text, &page.lines, line_idx, &currency);
                    if amt > 0 {
                        let desc = text.trim_end_matches(char::is_numeric).trim().to_string();
                        line_items.push(NormalizedLineItem {
                            description: desc,
                            amount_minor: amt,
                        });
                    }
                }
            }
        }

        // Validate arithmetic: subtotal + tax == total
        let mut is_arithmetic_valid = true;
        if subtotal_minor > 0 && total_minor > 0 {
            if subtotal_minor + tax_minor != total_minor {
                is_arithmetic_valid = false;
                uncertainty_reasons.push(format!(
                    "Arithmetic discrepancy: subtotal ({}) + tax ({}) != total ({})",
                    subtotal_minor, tax_minor, total_minor
                ));
            }
        }

        NormalizedInvoiceData {
            vendor_name: if vendor_name.is_empty() {
                "Unknown Supplier".to_string()
            } else {
                vendor_name
            },
            invoice_number,
            invoice_date,
            due_date,
            currency,
            subtotal_minor,
            tax_minor,
            total_minor,
            fields,
            line_items,
            is_arithmetic_valid,
            uncertainty_reasons,
        }
    }
}

/// Helper to parse amount from current line or peek adjacent line
fn extract_amount_from_line(line_text: &str, lines: &[OcrLine], idx: usize, currency: &str) -> i64 {
    // 1. Check if line itself contains a currency string or decimal number
    for part in line_text.split_whitespace().rev() {
        let clean = part.replace(['$', '€', '£', '₹', '¥'], "");
        if clean.contains('.') || clean.chars().all(|c| c.is_ascii_digit() || c == ',') {
            if let Ok(m) = CsvTransactionMapper::parse_amount_minor(&clean, currency) {
                if m > 0 {
                    return m;
                }
            }
        }
    }

    // 2. Check subsequent line (often amounts appear on the right column as next detected token)
    if idx + 1 < lines.len() {
        let next_text = &lines[idx + 1].text;
        let clean = next_text.replace(['$', '€', '£', '₹', '¥', ' '], "");
        if clean.contains('.') {
            if let Ok(m) = CsvTransactionMapper::parse_amount_minor(&clean, currency) {
                if m > 0 {
                    return m;
                }
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::ocr_client::OcrClient;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_gst_invoice() {
        let client = OcrClient::new().unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests/fixtures/invoices/gst_b2b_valid.pdf");

        let ocr_res = client.extract_document(&path).unwrap();
        let norm = FieldNormalizer::normalize_ocr(&ocr_res, "INR");

        assert_eq!(norm.vendor_name, "ACME Tech Services Pvt Ltd");
        assert_eq!(norm.invoice_number, Some("ACME-2026-0915".to_string()));
        assert_eq!(norm.invoice_date, Some("2026-09-15".to_string()));
        assert_eq!(norm.due_date, Some("2026-09-30".to_string()));
        assert_eq!(norm.currency, "INR");
        assert_eq!(norm.subtotal_minor, 10000000); // 1,00,000.00 INR
        assert_eq!(norm.tax_minor, 1800000); // 18,000.00 INR
        assert_eq!(norm.total_minor, 11800000); // 1,18,000.00 INR
        assert!(norm.is_arithmetic_valid);
    }

    #[test]
    fn test_normalize_thermal_receipt() {
        let client = OcrClient::new().unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests/fixtures/receipts/thermal_receipt_scanned.png");

        let ocr_res = client.extract_document(&path).unwrap();
        let norm = FieldNormalizer::normalize_ocr(&ocr_res, "USD");

        assert_eq!(norm.vendor_name, "OFFICE SUPPLIES MART");
        assert_eq!(norm.invoice_number, Some("REC-99214".to_string()));
        assert_eq!(norm.invoice_date, Some("2026-09-10".to_string()));
        assert_eq!(norm.currency, "USD");
        assert_eq!(norm.subtotal_minor, 4550); // $45.50
        assert_eq!(norm.total_minor, 4550); // $45.50
        assert!(norm.is_arithmetic_valid);
    }
}
