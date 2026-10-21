use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTransaction {
    pub posted_date: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub external_ref: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    #[error("File exceeds maximum allowable CSV size / row limit ({limit} rows max)")]
    RowLimitExceeded { limit: usize },
    #[error("CSV has no readable content or missing headers")]
    EmptyOrMissingHeader,
    #[error("Failed to detect Date or Amount columns")]
    MissingRequiredColumns,
    #[error("Row {row_index} parse error: {reason}")]
    RowParseError { row_index: usize, reason: String },
}

/// Max allowable rows to prevent memory exhaustion / CSV bomb attacks
pub const MAX_CSV_ROWS: usize = 10000;

pub struct CsvTransactionMapper;

impl CsvTransactionMapper {
    /// Parses exact currency amount into integer minor units without binary floating-point representation.
    pub fn parse_amount_minor(raw: &str, currency: &str) -> Result<i64, String> {
        let cleaned = raw.trim().replace(['$', '€', '£', '₹', '¥', ' '], "");
        if cleaned.is_empty() {
            return Ok(0);
        }

        let is_negative = cleaned.starts_with('-') || (cleaned.starts_with('(') && cleaned.ends_with(')'));
        let unsigned_str = cleaned
            .trim_start_matches('-')
            .trim_start_matches('+')
            .trim_start_matches('(')
            .trim_end_matches(')')
            .replace(',', ""); // Remove thousands separators (supports both 1,000.00 and Indian 1,00,000.00)

        let is_zero_decimal = currency.to_uppercase() == "JPY";

        let minor_units = if is_zero_decimal {
            i64::from_str(&unsigned_str)
                .map_err(|e| format!("Invalid integer amount '{}': {}", unsigned_str, e))?
        } else {
            let parts: Vec<&str> = unsigned_str.split('.').collect();
            match parts.len() {
                1 => {
                    let whole = i64::from_str(parts[0])
                        .map_err(|e| format!("Invalid whole amount '{}': {}", parts[0], e))?;
                    whole.checked_mul(100).ok_or("Amount overflow")?
                }
                2 => {
                    let whole = if parts[0].is_empty() {
                        0i64
                    } else {
                        i64::from_str(parts[0])
                            .map_err(|e| format!("Invalid whole amount '{}': {}", parts[0], e))?
                    };
                    let frac_str = parts[1];
                    let frac = match frac_str.len() {
                        0 => 0i64,
                        1 => {
                            let digit = i64::from_str(frac_str)
                                .map_err(|e| format!("Invalid fractional digit: {}", e))?;
                            digit * 10
                        }
                        2 => i64::from_str(frac_str)
                            .map_err(|e| format!("Invalid fractional digits: {}", e))?,
                        _ => {
                            // Truncate to 2 decimal places
                            i64::from_str(&frac_str[..2])
                                .map_err(|e| format!("Invalid fractional digits: {}", e))?
                        }
                    };
                    whole
                        .checked_mul(100)
                        .and_then(|w| w.checked_add(frac))
                        .ok_or("Amount overflow")?
                }
                _ => return Err(format!("Invalid decimal format: '{}'", unsigned_str)),
            }
        };

        Ok(if is_negative { -minor_units } else { minor_units })
    }

    /// Normalizes dates into standard ISO 8601 (YYYY-MM-DD).
    /// Supports:
    /// - YYYY-MM-DD
    /// - DD/MM/YYYY (Indian/UK formats like 01/09/2026)
    /// - MM/DD/YYYY
    /// - DD-Mon-YYYY (e.g. 15-Sep-2026)
    pub fn normalize_date(raw: &str) -> Result<String, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("Date string is empty".to_string());
        }

        // 1. ISO format: YYYY-MM-DD
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return Ok(d.format("%Y-%m-%d").to_string());
        }

        // 2. Slash format: DD/MM/YYYY
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
            return Ok(d.format("%Y-%m-%d").to_string());
        }

        // 3. Dash format with month name: DD-b-YYYY (e.g., 15-Sep-2026)
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d-%b-%Y") {
            return Ok(d.format("%Y-%m-%d").to_string());
        }

        // 4. US slash format: MM/DD/YYYY
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
            return Ok(d.format("%Y-%m-%d").to_string());
        }

        Err(format!("Unrecognized date format: '{}'", trimmed))
    }

    /// Detects delimiter automatically (comma, semicolon, or tab)
    fn detect_delimiter(first_line: &str) -> char {
        let comma_count = first_line.matches(',').count();
        let semi_count = first_line.matches(';').count();
        let tab_count = first_line.matches('\t').count();

        if tab_count > comma_count && tab_count > semi_count {
            '\t'
        } else if semi_count > comma_count {
            ';'
        } else {
            ','
        }
    }

    /// Pure Rust RFC 4180 CSV line parser
    pub fn parse_csv_line(line: &str, delimiter: char) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if in_quotes {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        // Escaped double quote ("")
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    current_field.push(c);
                }
            } else if c == '"' {
                in_quotes = true;
            } else if c == delimiter {
                fields.push(current_field.trim().to_string());
                current_field = String::new();
            } else {
                current_field.push(c);
            }
        }
        fields.push(current_field.trim().to_string());
        fields
    }

    /// Parses CSV content into normalized transactions.
    /// Checks row limits against CSV expansion bombs.
    pub fn parse_csv_content(
        content: &str,
        default_currency: &str,
    ) -> Result<Vec<ParsedTransaction>, CsvError> {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(CsvError::EmptyOrMissingHeader);
        }

        // Guard against giant CSV row bombs
        if lines.len() - 1 > MAX_CSV_ROWS {
            return Err(CsvError::RowLimitExceeded { limit: MAX_CSV_ROWS });
        }

        let delimiter = Self::detect_delimiter(lines[0]);
        let headers = Self::parse_csv_line(lines[0], delimiter);

        // Identify column indices
        let mut date_col = None;
        let mut desc_col = None;
        let mut amount_col = None;
        let mut withdrawal_col = None;
        let mut deposit_col = None;
        let mut currency_col = None;
        let mut ref_col = None;

        for (idx, header) in headers.iter().enumerate() {
            let h = header.to_lowercase().replace(['_', '-', ' '], "");
            if date_col.is_none() && (h.contains("date") || h == "valuedt" || h == "txdate") {
                date_col = Some(idx);
            } else if desc_col.is_none()
                && (h.contains("description") || h.contains("narration") || h.contains("memo") || h.contains("details"))
            {
                desc_col = Some(idx);
            } else if h == "amount" || h == "txamount" {
                amount_col = Some(idx);
            } else if h.contains("withdrawal") || h == "debit" {
                withdrawal_col = Some(idx);
            } else if h.contains("deposit") || h == "credit" {
                deposit_col = Some(idx);
            } else if h == "currency" || h == "ccy" {
                currency_col = Some(idx);
            } else if h.contains("ref") || h.contains("chq") || h.contains("check") {
                ref_col = Some(idx);
            }
        }

        let date_idx = date_col.ok_or(CsvError::MissingRequiredColumns)?;
        let desc_idx = desc_col.unwrap_or(if date_idx == 0 { 1 } else { 0 });

        let has_split_amounts = withdrawal_col.is_some() || deposit_col.is_some();
        if amount_col.is_none() && !has_split_amounts {
            return Err(CsvError::MissingRequiredColumns);
        }

        let mut transactions = Vec::new();

        for (row_idx, line) in lines[1..].iter().enumerate() {
            let fields = Self::parse_csv_line(line, delimiter);

            let raw_date = fields.get(date_idx).map(|s| s.as_str()).unwrap_or("");
            if raw_date.is_empty() {
                continue; // Skip summary or empty rows
            }

            let posted_date = match Self::normalize_date(raw_date) {
                Ok(d) => d,
                Err(_) => continue, // Skip unparseable headers or footer totals
            };

            let description = fields.get(desc_idx).cloned().unwrap_or_default();

            // Skip opening/closing balance rows
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("opening balance") || desc_lower.contains("closing balance") {
                continue;
            }

            let currency = if let Some(c_idx) = currency_col {
                let c = fields.get(c_idx).map(|s| s.trim().to_uppercase()).unwrap_or_default();
                if c.is_empty() {
                    default_currency.to_string()
                } else {
                    c
                }
            } else {
                default_currency.to_string()
            };

            let amount_minor = if let Some(a_idx) = amount_col {
                let raw_amt = fields.get(a_idx).map(|s| s.as_str()).unwrap_or("0");
                Self::parse_amount_minor(raw_amt, &currency).map_err(|r| {
                    CsvError::RowParseError {
                        row_index: row_idx + 1,
                        reason: r,
                    }
                })?
            } else {
                // Split withdrawal / deposit columns
                let w_str = withdrawal_col.and_then(|idx| fields.get(idx)).map(|s| s.as_str()).unwrap_or("0");
                let d_str = deposit_col.and_then(|idx| fields.get(idx)).map(|s| s.as_str()).unwrap_or("0");

                let w_minor = Self::parse_amount_minor(w_str, &currency).unwrap_or(0);
                let d_minor = Self::parse_amount_minor(d_str, &currency).unwrap_or(0);

                if w_minor > 0 {
                    -w_minor // Withdrawal is negative
                } else {
                    d_minor // Deposit is positive
                }
            };

            let external_ref = ref_col
                .and_then(|idx| fields.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            transactions.push(ParsedTransaction {
                posted_date,
                description,
                amount_minor,
                currency,
                external_ref,
            });
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount_minor_exact() {
        assert_eq!(CsvTransactionMapper::parse_amount_minor("120.00", "USD").unwrap(), 12000);
        assert_eq!(CsvTransactionMapper::parse_amount_minor("-45.50", "USD").unwrap(), -4550);
        assert_eq!(CsvTransactionMapper::parse_amount_minor("$5,000.00", "USD").unwrap(), 500000);
        assert_eq!(CsvTransactionMapper::parse_amount_minor("(90.00)", "USD").unwrap(), -9000);
        assert_eq!(CsvTransactionMapper::parse_amount_minor("1,00,000.00", "INR").unwrap(), 10000000);
        assert_eq!(CsvTransactionMapper::parse_amount_minor("50000", "JPY").unwrap(), 50000);
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(CsvTransactionMapper::normalize_date("2026-09-01").unwrap(), "2026-09-01");
        assert_eq!(CsvTransactionMapper::normalize_date("01/09/2026").unwrap(), "2026-09-01");
        assert_eq!(CsvTransactionMapper::normalize_date("15-Sep-2026").unwrap(), "2026-09-15");
    }

    #[test]
    fn test_parse_us_bank_csv_fixture() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/statements/bank_us_sample.csv");
        let content = std::fs::read_to_string(path).unwrap();
        let txs = CsvTransactionMapper::parse_csv_content(&content, "USD").unwrap();

        assert_eq!(txs.len(), 7);
        assert_eq!(txs[0].description, "CloudHosting Corp INV-88912");
        assert_eq!(txs[0].amount_minor, -12000); // -$120.00
        assert_eq!(txs[3].description, "Office Supplies Mart");
        assert_eq!(txs[3].amount_minor, -4550); // -$45.50
        assert_eq!(txs[4].description, "Client Payment ACME Advisory");
        assert_eq!(txs[4].amount_minor, 500000); // +$5,000.00
    }

    #[test]
    fn test_parse_hdfc_bank_csv_fixture() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/statements/bank_hdfc_sample.csv");
        let content = std::fs::read_to_string(path).unwrap();
        let txs = CsvTransactionMapper::parse_csv_content(&content, "INR").unwrap();

        // 8 rows minus opening and closing balances = 6 transactions
        assert_eq!(txs.len(), 6);
        assert_eq!(txs[0].description, "UPI-SWIGGY-BANGALORE");
        assert_eq!(txs[0].amount_minor, -45000); // 450.00 INR withdrawal = -45000 minor
        assert_eq!(txs[0].external_ref, Some("UPI-624601928".to_string()));

        assert_eq!(txs[1].description, "NEFT-ACME TECH SERVICES-ACME-2026-0915");
        assert_eq!(txs[1].amount_minor, 11800000); // 118,000.00 INR deposit = +11800000 minor
        assert_eq!(txs[1].external_ref, Some("N246019283".to_string()));
    }

    #[test]
    fn test_csv_bomb_rejection() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/adversarial/bomb_50k_rows.csv");
        let content = std::fs::read_to_string(path).unwrap();
        let res = CsvTransactionMapper::parse_csv_content(&content, "USD");
        assert!(res.is_err(), "Must reject CSV bomb exceeding row limit");
    }
}
