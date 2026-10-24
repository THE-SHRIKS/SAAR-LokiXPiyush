use crate::ingestion::csv_mapper::CsvTransactionMapper;
use crate::ingestion::ocr_client::{OcrLine, OcrResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Invoice number after a label such as "Invoice No:", "Reference", "Rechnung / Invoice:" or
/// "RECEIPTID:". The value must contain a digit, so "Invoice Date: ..." is never read as a number.
fn invoice_number_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:invoice\s*(?:no\.?|number|#)|rechnung(?:\s*/\s*invoice)?|invoice|reference|ref\.?|receipt\s*id|receiptid)\s*[:#]?\s*([A-Za-z0-9_/.\-]*[0-9][A-Za-z0-9_/.\-]*)",
        )
        .unwrap()
    })
}

/// Issue date label at the start of a line: "Date:", "Date", "Invoice Date:", "Issued", "Datum:".
fn date_label_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:invoice\s*date|issue\s*date|date|issued(?:\s*on)?|datum)\s*[:\s]\s*(\S+)")
            .unwrap()
    })
}

/// Due date label at the start of a line: "Due Date:", "Due:", "Payment due", "Faellig:".
fn due_label_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:due\s*date|payment\s*due|due|faellig|fällig)\s*[:\s]\s*(\S+)").unwrap()
    })
}

/// "Due Date: <value>" appearing after other text on the same line.
fn inline_due_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)due\s*date\s*:\s*(\S+)").unwrap())
}

/// "Date: <value>" appearing after other text on the same line.
fn inline_date_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)date\s*:\s*(\S+)").unwrap())
}

/// A tax label as a whole word: "Tax 19%", "GST", "VAT", "IGST". A substring test would read
/// "Taxi fare 31.53" as a tax line.
fn tax_label_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:tax|vat|gst|igst|cgst|sgst)\b").unwrap())
}

fn currency_code_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b(USD|EUR|INR|GBP|JPY)\b").unwrap())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedField {
    pub field_key: String,
    pub original_text: String,
    pub normalized_value: String,
    pub confidence: f64,
    pub page_num: i32,
    pub bounding_box_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub discount_minor: i64,
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
        // Prefer an explicit ISO code written as a whole word; the most frequent one wins.
        // Substring matching would misread names such as "NEUROTECH" as EUR.
        let mut counts: Vec<(String, usize)> = Vec::new();
        for m in currency_code_pattern().find_iter(full_text) {
            let code = m.as_str().to_uppercase();
            match counts.iter_mut().find(|(c, _)| *c == code) {
                Some((_, n)) => *n += 1,
                None => counts.push((code, 1)),
            }
        }
        if let Some((code, _)) = counts.iter().max_by_key(|(_, n)| *n) {
            return code.clone();
        }
        let upper = full_text.to_uppercase();
        if upper.contains("₹") || upper.contains("GSTIN") {
            "INR".to_string()
        } else if upper.contains('€') {
            "EUR".to_string()
        } else if upper.contains('£') {
            "GBP".to_string()
        } else if upper.contains('¥') {
            "JPY".to_string()
        } else if upper.contains('$') {
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
        let mut discount_minor = 0i64;
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
                        && !lower.starts_with("tax")
                        && !lower.starts_with("tel")
                        && !lower.contains("tel:")
                        && !lower.starts_with("date")
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
                    if let Some(caps) = invoice_number_pattern().captures(text) {
                        let clean_inv = caps[1].trim_end_matches(['.', ',']).to_string();
                        if !clean_inv.is_empty() {
                            invoice_number = Some(clean_inv.clone());
                            fields.push(NormalizedField {
                                field_key: "invoice_number".to_string(),
                                original_text: text.to_string(),
                                normalized_value: clean_inv,
                                confidence: line.confidence,
                                page_num: page.page_num,
                                bounding_box_json: bbox_str.clone(),
                            });
                        }
                    }
                }

                // 3. Invoice date & due date
                let due_capture = due_label_pattern().captures(text);
                let date_capture = date_label_pattern().captures(text);
                if invoice_date.is_none()
                    && due_capture.is_none()
                    && (date_capture.is_some() || lower.contains("date:"))
                {
                    // Label at the start of the line ("Date:", "Issued", "Datum:"), or a line that
                    // carries "... Date: <value>" after other text (e.g. a combined header line).
                    let raw_d = match &date_capture {
                        Some(caps) => caps[1].to_string(),
                        None => inline_date_pattern()
                            .captures_iter(text)
                            .last()
                            .map(|c| c[1].to_string())
                            .unwrap_or_default(),
                    };
                    if let Ok(iso_d) = CsvTransactionMapper::normalize_date(&raw_d) {
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

                let due_raw = due_capture
                    .as_ref()
                    .map(|c| c[1].to_string())
                    .or_else(|| inline_due_pattern().captures(text).map(|c| c[1].to_string()));
                if due_date.is_none() && due_raw.is_some() {
                    let raw_d = due_raw.unwrap_or_default();
                    if let Ok(iso_d) = CsvTransactionMapper::normalize_date(&raw_d) {
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
                if lower.starts_with("discount") {
                    let amt = extract_amount_from_line(text, &page.lines, line_idx, &currency);
                    if amt > 0 {
                        discount_minor += amt;
                        fields.push(NormalizedField {
                            field_key: "discount".to_string(),
                            original_text: text.to_string(),
                            normalized_value: format!("{}", amt),
                            confidence: line.confidence,
                            page_num: page.page_num,
                            bounding_box_json: bbox_str.clone(),
                        });
                    }
                } else if lower.starts_with("subtotal") || lower.contains("subtotal:") {
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
                } else if tax_label_pattern().is_match(&lower) {
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
            if subtotal_minor - discount_minor + tax_minor != total_minor {
                is_arithmetic_valid = false;
                uncertainty_reasons.push(format!(
                    "Arithmetic discrepancy: subtotal ({}) - discount ({}) + tax ({}) != total ({})",
                    subtotal_minor, discount_minor, tax_minor, total_minor
                ));
            }
        }
        if total_minor == 0 {
            is_arithmetic_valid = false;
            uncertainty_reasons.push("Total amount could not be read; review required.".to_string());
        }
        if invoice_date.is_none() {
            uncertainty_reasons.push("Invoice date not found in the document.".to_string());
        }
        if invoice_number.is_none() {
            uncertainty_reasons.push("Invoice number not found in the document.".to_string());
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
            discount_minor,
            tax_minor,
            total_minor,
            fields,
            line_items,
            is_arithmetic_valid,
            uncertainty_reasons,
        }
    }
}

/// The last token on a line that looks like an amount, e.g. `2.306,22`, `-102,00`, `$45.50`,
/// `USD128.45`. Percentages (`19%`) and plain words are not amounts.
fn amount_token(text: &str) -> Option<String> {
    for part in text.split_whitespace().rev() {
        let t = part.trim_matches(|c: char| matches!(c, '$' | '€' | '£' | '₹' | '¥' | ':' | ';'));
        let t = t.trim_start_matches(|c: char| c.is_ascii_alphabetic());
        let core = t.trim_matches(|c: char| matches!(c, '-' | '(' | ')' | '\u{2212}'));
        let starts_with_digit = core.chars().next().map_or(false, |c| c.is_ascii_digit());
        let only_number_chars = core
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '\''));
        if starts_with_digit && only_number_chars {
            return Some(t.to_string());
        }
    }
    None
}

/// True when the line is only an amount, optionally with a currency code or symbol.
fn is_amount_only_line(text: &str) -> bool {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    match amount_token(text) {
        None => false,
        Some(tok) => tokens.iter().all(|t| {
            t.contains(tok.as_str())
                || t.chars().all(|c| c.is_ascii_alphabetic() || matches!(c, '$' | '€' | '£' | '₹' | '¥'))
                    && t.chars().count() <= 3
        }),
    }
}

/// Parses an amount from the current line, or from the next line when the label stands alone
/// (OCR often reports a label and its value as separate lines). Returns the magnitude in minor
/// units, or 0 when no amount is found.
fn extract_amount_from_line(line_text: &str, lines: &[OcrLine], idx: usize, currency: &str) -> i64 {
    if let Some(tok) = amount_token(line_text) {
        if let Ok(m) = CsvTransactionMapper::parse_amount_minor(&tok, currency) {
            if m != 0 {
                return m.abs();
            }
        }
    }

    let next_is_amount = idx + 1 < lines.len() && is_amount_only_line(lines[idx + 1].text.trim());
    if next_is_amount {
        if let Some(tok) = amount_token(lines[idx + 1].text.trim()) {
            if let Ok(m) = CsvTransactionMapper::parse_amount_minor(&tok, currency) {
                if m != 0 {
                    return m.abs();
                }
            }
        }
    }

    // Rotated or skewed scans can be read value-first, so the amount precedes its label. Use the
    // previous line only when the next line offers no amount.
    if !next_is_amount && idx > 0 {
        let prev_text = lines[idx - 1].text.trim();
        if is_amount_only_line(prev_text) {
            if let Some(tok) = amount_token(prev_text) {
                if let Ok(m) = CsvTransactionMapper::parse_amount_minor(&tok, currency) {
                    if m != 0 {
                        return m.abs();
                    }
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

#[cfg(test)]
mod label_tests {
    use super::*;
    use crate::ingestion::ocr_client::{OcrLine, OcrPage, OcrResponse};

    fn response(lines: &[&str]) -> OcrResponse {
        OcrResponse {
            status: "success".to_string(),
            tier: "test".to_string(),
            page_count: 1,
            pages: vec![OcrPage {
                page_num: 1,
                width: 500.0,
                height: 700.0,
                lines: lines
                    .iter()
                    .map(|t| OcrLine { text: t.to_string(), confidence: 0.99, bbox: serde_json::json!([]) })
                    .collect(),
            }],
            error: None,
        }
    }

    #[test]
    fn test_taxi_fare_is_not_a_tax_line() {
        // Lines as OCR reported them for a parking and taxi receipt.
        let norm = FieldNormalizer::normalize_ocr(
            &response(&["CITY PARKING", "DATE 2025-04-27", "Notebook", "10.29", "Taxi fare", "31.53", "Coffee", "3.78", "TOTAL", "USD 45.60"]),
            "USD",
        );
        assert_eq!(norm.tax_minor, 0, "'Taxi fare' must not be read as tax");
        assert_eq!(norm.total_minor, 4560);
        assert_eq!(norm.invoice_date.as_deref(), Some("2025-04-27"));
    }

    #[test]
    fn test_real_tax_labels_are_still_read() {
        let norm = FieldNormalizer::normalize_ocr(
            &response(&["ACME LTD", "Invoice No: A-1", "Subtotal USD: 100.00", "Tax 18%: 18.00", "Total Due USD: 118.00"]),
            "USD",
        );
        assert_eq!(norm.tax_minor, 1800);
        assert!(norm.is_arithmetic_valid);
        let gst = FieldNormalizer::normalize_ocr(&response(&["ACME LTD", "Subtotal: 100.00", "GST: 5.00", "Total: 105.00"]), "USD");
        assert_eq!(gst.tax_minor, 500);
    }

    #[test]
    fn test_vendor_name_containing_tel_or_tax_is_still_a_vendor() {
        let norm = FieldNormalizer::normalize_ocr(&response(&["GRAND HOTEL AND SYNTAX CAFE", "Invoice No: H-9", "Total: 20.00"]), "USD");
        assert_eq!(norm.vendor_name, "GRAND HOTEL AND SYNTAX CAFE");
    }
}
