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
    /// Parses a currency amount into integer minor units with exact integer arithmetic.
    ///
    /// The decimal separator is inferred from the shape of the number, so US (`1,234.56`),
    /// European (`1.234,56`), Indian (`1,23,456.00`) and plain (`1234.56`) formats all parse to the
    /// same value. Rules:
    /// - Both `.` and `,` present: the rightmost one is the decimal separator, the other groups thousands.
    /// - One separator kind, repeated: thousands grouping, no decimals.
    /// - One separator, once: a decimal separator when 1-2 digits follow (0 digits for a trailing
    ///   separator); thousands grouping when exactly 3 digits follow.
    /// Anything else is rejected instead of being truncated, so no digit is silently lost.
    pub fn parse_amount_minor(raw: &str, currency: &str) -> Result<i64, String> {
        // Keep digits, separators and sign markers; drop symbols, currency codes, spaces.
        let trimmed = raw.trim();
        let is_negative = trimmed.starts_with('-')
            || trimmed.starts_with('\u{2212}')
            || (trimmed.starts_with('(') && trimmed.ends_with(')'))
            || trimmed.ends_with('-');
        let body: String = trimmed
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '\'')
            .filter(|c| *c != '\'')
            .collect();
        if body.is_empty() {
            // Only a genuinely blank cell is zero. Text with no digits is a malformed amount.
            return if trimmed.is_empty() {
                Ok(0)
            } else {
                Err(format!("Invalid amount '{}'", raw))
            };
        }

        let exponent: usize = if currency.eq_ignore_ascii_case("JPY") { 0 } else { 2 };
        let last_dot = body.rfind('.');
        let last_comma = body.rfind(',');
        let dots = body.matches('.').count();
        let commas = body.matches(',').count();

        // Decide which character (if any) is the decimal separator.
        let decimal_sep: Option<char> = match (last_dot, last_comma) {
            (None, None) => None,
            (Some(d), Some(c)) => Some(if d > c { '.' } else { ',' }),
            (Some(d), None) => Self::single_sep_role(&body, '.', dots, d, exponent)?,
            (None, Some(c)) => Self::single_sep_role(&body, ',', commas, c, exponent)?,
        };

        // The decimal separator may appear once; the other kind (if any) must be valid thousands grouping.
        if let Some(sep) = decimal_sep {
            if body.matches(sep).count() > 1 {
                return Err(format!("Invalid amount '{}'", raw));
            }
        }
        let grouping_char = match decimal_sep {
            Some('.') => ',',
            Some(_) => '.',
            None => if dots > 0 { '.' } else { ',' },
        };
        let whole_for_grouping = match decimal_sep {
            Some(sep) => &body[..body.rfind(sep).unwrap()],
            None => body.as_str(),
        };
        if !Self::valid_grouping(whole_for_grouping, grouping_char) {
            return Err(format!("Invalid digit grouping in amount '{}'", raw));
        }

        let (whole_part, frac_part) = match decimal_sep {
            Some(sep) => {
                let idx = body.rfind(sep).unwrap();
                (body[..idx].to_string(), body[idx + 1..].to_string())
            }
            None => (body.clone(), String::new()),
        };
        let whole_digits: String = whole_part.chars().filter(|c| c.is_ascii_digit()).collect();
        let whole = if whole_digits.is_empty() {
            0i64
        } else {
            i64::from_str(&whole_digits).map_err(|e| format!("Invalid whole amount '{}': {}", raw, e))?
        };

        if frac_part.len() > exponent && !frac_part.chars().all(|c| c == '0') {
            return Err(format!(
                "Amount '{}' has more than {} decimal places for {}",
                raw, exponent, currency
            ));
        }
        let frac_digits: String = frac_part.chars().take(exponent).collect();
        let frac = if exponent == 0 || frac_digits.is_empty() {
            0i64
        } else {
            let padded = format!("{:0<width$}", frac_digits, width = exponent);
            i64::from_str(&padded).map_err(|e| format!("Invalid fractional digits '{}': {}", raw, e))?
        };

        let scale: i64 = if exponent == 0 { 1 } else { 100 };
        let minor = whole
            .checked_mul(scale)
            .and_then(|w| w.checked_add(frac))
            .ok_or_else(|| "Amount overflow".to_string())?;
        Ok(if is_negative { -minor } else { minor })
    }

    /// Thousands grouping must be regular: a first group of 1-3 digits, then either all groups of
    /// three (Western/European) or middle groups of two with a final group of three (Indian).
    fn valid_grouping(whole: &str, sep: char) -> bool {
        let segs: Vec<&str> = whole.split(sep).collect();
        if segs.len() == 1 {
            return true;
        }
        let (first, rest) = segs.split_first().unwrap();
        let (last, middle) = rest.split_last().unwrap();
        let first_ok = (1..=3).contains(&first.len()) && first.chars().all(|c| c.is_ascii_digit());
        let last_ok = last.len() == 3 && last.chars().all(|c| c.is_ascii_digit());
        let middle_ok = middle.iter().all(|m| m.len() == 3 && m.chars().all(|c| c.is_ascii_digit()))
            || middle.iter().all(|m| m.len() == 2 && m.chars().all(|c| c.is_ascii_digit()));
        first_ok && last_ok && middle_ok
    }

    /// Role of a lone separator kind: `Some(sep)` if it is the decimal separator, `None` if it
    /// groups thousands. Errors when the digits around it fit neither role.
    fn single_sep_role(
        body: &str,
        sep: char,
        count: usize,
        last_idx: usize,
        exponent: usize,
    ) -> Result<Option<char>, String> {
        if count > 1 {
            return Ok(None); // repeated: grouping only
        }
        let after = body.len() - last_idx - 1;
        match after {
            3 => Ok(None), // exactly three digits: thousands grouping, e.g. 1,234 or 1.234
            0 => Ok(Some(sep)),
            1 | 2 if exponent > 0 => Ok(Some(sep)),
            _ if exponent == 0 && body[last_idx + 1..].chars().all(|c| c == '0') => Ok(Some(sep)),
            _ => Err(format!("Ambiguous or invalid decimal format: '{}'", body)),
        }
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

        // 5. European dotted format: DD.MM.YYYY, and dashed DD-MM-YYYY
        for fmt in ["%d.%m.%Y", "%d-%m-%Y"] {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, fmt) {
                return Ok(d.format("%Y-%m-%d").to_string());
            }
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
            return Err(CsvError::RowLimitExceeded {
                limit: MAX_CSV_ROWS,
            });
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
                && (h.contains("description")
                    || h.contains("narration")
                    || h.contains("memo")
                    || h.contains("details"))
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

            // Blank lines are skipped. A line that carries data but cannot be read as a transaction
            // is reported with its row number, never dropped silently. Summary lines (totals,
            // balances) are the one legitimate skip.
            if fields.iter().all(|f| f.trim().is_empty()) {
                continue;
            }
            let looks_like_summary = fields.iter().any(|f| {
                let l = f.to_lowercase();
                ["total", "balance", "summary", "statement", "opening", "closing"]
                    .iter()
                    .any(|k| l.contains(k))
            });

            let raw_date = fields.get(date_idx).map(|s| s.as_str()).unwrap_or("");
            if raw_date.trim().is_empty() {
                if looks_like_summary {
                    continue;
                }
                return Err(CsvError::RowParseError {
                    row_index: row_idx + 1,
                    reason: "missing date".to_string(),
                });
            }

            let posted_date = match Self::normalize_date(raw_date) {
                Ok(d) => d,
                Err(reason) => {
                    if looks_like_summary {
                        continue;
                    }
                    return Err(CsvError::RowParseError { row_index: row_idx + 1, reason });
                }
            };

            let description = fields.get(desc_idx).cloned().unwrap_or_default();

            // Skip opening/closing balance rows
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("opening balance") || desc_lower.contains("closing balance") {
                continue;
            }

            let currency = if let Some(c_idx) = currency_col {
                let c = fields
                    .get(c_idx)
                    .map(|s| s.trim().to_uppercase())
                    .unwrap_or_default();
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
                let w_str = withdrawal_col
                    .and_then(|idx| fields.get(idx))
                    .map(|s| s.as_str())
                    .unwrap_or("0");
                let d_str = deposit_col
                    .and_then(|idx| fields.get(idx))
                    .map(|s| s.as_str())
                    .unwrap_or("0");

                let row_err = |reason: String| CsvError::RowParseError {
                    row_index: row_idx + 1,
                    reason,
                };
                let w_minor = Self::parse_amount_minor(w_str, &currency).map_err(&row_err)?;
                let d_minor = Self::parse_amount_minor(d_str, &currency).map_err(&row_err)?;

                if w_minor != 0 && d_minor != 0 {
                    return Err(row_err(
                        "both a debit and a credit amount are present".to_string(),
                    ));
                }
                if w_minor != 0 {
                    -w_minor.abs() // Withdrawal is negative
                } else {
                    d_minor // Deposit keeps its parsed sign (a negative deposit is a reversal)
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
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("120.00", "USD").unwrap(),
            12000
        );
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("-45.50", "USD").unwrap(),
            -4550
        );
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("$5,000.00", "USD").unwrap(),
            500000
        );
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("(90.00)", "USD").unwrap(),
            -9000
        );
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("1,00,000.00", "INR").unwrap(),
            10000000
        );
        assert_eq!(
            CsvTransactionMapper::parse_amount_minor("50000", "JPY").unwrap(),
            50000
        );
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(
            CsvTransactionMapper::normalize_date("2026-09-01").unwrap(),
            "2026-09-01"
        );
        assert_eq!(
            CsvTransactionMapper::normalize_date("01/09/2026").unwrap(),
            "2026-09-01"
        );
        assert_eq!(
            CsvTransactionMapper::normalize_date("15-Sep-2026").unwrap(),
            "2026-09-15"
        );
    }

    #[test]
    fn test_parse_us_bank_csv_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/statements/bank_us_sample.csv"
        );
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
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/statements/bank_hdfc_sample.csv"
        );
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
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/adversarial/bomb_50k_rows.csv"
        );
        let content = std::fs::read_to_string(path).unwrap();
        let res = CsvTransactionMapper::parse_csv_content(&content, "USD");
        assert!(res.is_err(), "Must reject CSV bomb exceeding row limit");
    }
}

#[cfg(test)]
mod amount_parser_tests {
    use super::CsvTransactionMapper as M;

    fn usd(s: &str) -> i64 {
        M::parse_amount_minor(s, "USD").unwrap()
    }

    #[test]
    fn test_us_european_and_indian_formats_agree() {
        assert_eq!(usd("1,234.56"), 123_456);
        assert_eq!(usd("1.234,56"), 123_456);
        assert_eq!(usd("1,23,456.00"), 12_345_600);
        assert_eq!(usd("2.306,22"), 230_622);
        assert_eq!(usd("86,708.78"), 8_670_878);
        assert_eq!(usd("1234.56"), 123_456);
    }

    #[test]
    fn test_single_separator_roles() {
        assert_eq!(usd("856,80"), 85_680, "comma with two digits is a decimal separator");
        assert_eq!(usd("12.5"), 1_250);
        assert_eq!(usd("12"), 1_200);
        assert_eq!(usd("1,234"), 123_400, "comma with three digits groups thousands");
        assert_eq!(usd("1.234"), 123_400);
        assert_eq!(usd("1,000,000"), 100_000_000);
        assert_eq!(usd("12.345,6"), 1_234_560);
    }

    #[test]
    fn test_symbols_codes_signs_and_blanks() {
        assert_eq!(usd("$45.50"), 4_550);
        assert_eq!(usd("USD 128.45"), 12_845);
        assert_eq!(usd("EUR 1.234,56"), 123_456);
        assert_eq!(usd("(45.00)"), -4_500);
        assert_eq!(usd("-0.05"), -5);
        assert_eq!(usd("45.00-"), -4_500);
        assert_eq!(usd(""), 0);
        assert_eq!(usd("   "), 0);
    }

    #[test]
    fn test_zero_decimal_currency() {
        assert_eq!(M::parse_amount_minor("5000", "JPY").unwrap(), 5_000);
        assert_eq!(M::parse_amount_minor("5,000", "JPY").unwrap(), 5_000);
        assert_eq!(M::parse_amount_minor("5.00", "JPY").unwrap(), 5);
        assert!(M::parse_amount_minor("5.50", "JPY").is_err(), "JPY has no fractional units");
    }

    #[test]
    fn test_excess_precision_overflow_and_garbage_are_rejected_not_truncated() {
        assert!(M::parse_amount_minor("1,234.567", "USD").is_err());
        assert!(M::parse_amount_minor("99999999999999999999.99", "USD").is_err());
        assert!(M::parse_amount_minor("92233720368547758.99", "USD").is_err());
        assert!(M::parse_amount_minor("twelve", "USD").is_err(), "text without digits is malformed, not zero");
        assert!(M::parse_amount_minor("1.2.3.4,5", "USD").is_err(), "irregular grouping is rejected");
        assert!(M::parse_amount_minor("1.234.56,78", "USD").is_err());
        assert!(M::parse_amount_minor("12,34,567.00", "USD").is_err() == false, "Indian grouping 12,34,567 is valid");
        assert!(M::parse_amount_minor("1,2345.00", "USD").is_err(), "a four-digit group is invalid");
    }
}

#[cfg(test)]
mod row_handling_tests {
    use super::{CsvError, CsvTransactionMapper as M};

    const HEADER: &str = "Date,Description,Debit,Credit,Currency\n";

    fn parse(body: &str) -> Result<Vec<super::ParsedTransaction>, CsvError> {
        M::parse_csv_content(&format!("{}{}", HEADER, body), "USD")
    }

    #[test]
    fn test_bad_debit_amount_is_a_row_error_not_zero() {
        let err = parse("2026-09-01,GOOD,10.00,,USD\n2026-09-02,BAD,twelve,,USD\n").unwrap_err();
        match err {
            CsvError::RowParseError { row_index, .. } => assert_eq!(row_index, 2),
            other => panic!("expected a row error, got {other:?}"),
        }
    }

    #[test]
    fn test_row_with_data_but_no_date_is_reported() {
        let err = parse("2026-09-01,GOOD,10.00,,USD\n,NO DATE ROW,5.00,,USD\n").unwrap_err();
        assert!(matches!(err, CsvError::RowParseError { row_index: 2, .. }));
    }

    #[test]
    fn test_unrecognised_date_is_reported() {
        let err = parse("31/31/2026,IMPOSSIBLE DATE,5.00,,USD\n").unwrap_err();
        assert!(matches!(err, CsvError::RowParseError { row_index: 1, .. }));
    }

    #[test]
    fn test_debit_and_credit_on_one_row_is_ambiguous() {
        let err = parse("2026-09-01,BOTH,5.00,5.00,USD\n").unwrap_err();
        assert!(matches!(err, CsvError::RowParseError { .. }));
    }

    #[test]
    fn test_summary_footers_and_blank_lines_are_skipped_not_errors() {
        let rows = parse(
            "2026-09-01,PAYMENT,10.00,,USD\n\nTotal,,10.00,,\nClosing balance,,,,\n2026-09-02,DEPOSIT,,25.00,USD\n",
        )
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].amount_minor, -1000);
        assert_eq!(rows[1].amount_minor, 2500);
    }

    #[test]
    fn test_blank_amount_cells_are_a_zero_row_not_an_error() {
        let rows = parse("2026-09-01,ZERO VALUE,,,USD\n").unwrap();
        assert_eq!(rows[0].amount_minor, 0);
    }
}
