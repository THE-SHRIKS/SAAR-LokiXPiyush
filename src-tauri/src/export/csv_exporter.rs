use crate::export::pdf_writer::ReconciliationReportRow;
use crate::finance::aggregates::FinancialSummary;
use sha2::{Digest, Sha256};

/// RFC 4180 compliant CSV field escaping
pub fn escape_csv_field(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        val.to_string()
    }
}

pub struct CsvExporter;

impl CsvExporter {
    /// Format signed integer minor units to standard decimal string
    pub fn format_minor(cents: i64) -> String {
        let is_negative = cents < 0;
        let abs = cents.abs();
        let whole = abs / 100;
        let frac = abs % 100;
        let sign = if is_negative { "-" } else { "" };
        format!("{sign}{whole}.{frac:02}")
    }

    /// Export reconciled transaction records to RFC 4180 CSV with SHA-256 provenance hash
    pub fn export_reconciliation(records: &[ReconciliationReportRow]) -> (String, String) {
        let mut out = String::new();
        out.push_str("date,description,amount_minor,amount_formatted,currency,match_status,invoice_number,provenance_id\r\n");

        for r in records {
            let row = format!(
                "{},{},{},{},{},{},{},{}\r\n",
                escape_csv_field(&r.date),
                escape_csv_field(&r.description),
                r.amount_minor,
                escape_csv_field(&Self::format_minor(r.amount_minor)),
                escape_csv_field(&r.currency),
                escape_csv_field(&r.match_status),
                escape_csv_field(&r.invoice_number),
                escape_csv_field(&r.provenance_id),
            );
            out.push_str(&row);
        }

        let mut hasher = Sha256::new();
        hasher.update(out.as_bytes());
        let hash = hex::encode(hasher.finalize());

        (out, hash)
    }

    /// Export monthly financial aggregates to RFC 4180 CSV with SHA-256 provenance hash
    pub fn export_summary(summary: &FinancialSummary) -> (String, String) {
        let mut out = String::new();
        out.push_str("month,currency,transaction_count,income_minor,expense_minor,net_cash_flow_minor,income_formatted,expense_formatted,net_cash_flow_formatted\r\n");

        for m in &summary.monthly_breakdown {
            let row = format!(
                "{},{},{},{},{},{},{},{},{}\r\n",
                escape_csv_field(&m.month_key),
                escape_csv_field(&m.currency),
                m.transaction_count,
                m.total_income_minor,
                m.total_expense_minor,
                m.net_cash_flow_minor,
                escape_csv_field(&Self::format_minor(m.total_income_minor)),
                escape_csv_field(&Self::format_minor(m.total_expense_minor)),
                escape_csv_field(&Self::format_minor(m.net_cash_flow_minor)),
            );
            out.push_str(&row);
        }

        // Summary Total Row
        let total_row = format!(
            "TOTAL,{},{},{},{},{},{},{},{}\r\n",
            escape_csv_field(&summary.currency),
            summary
                .monthly_breakdown
                .iter()
                .map(|m| m.transaction_count)
                .sum::<usize>(),
            summary.total_income_minor,
            summary.total_expense_minor,
            summary.net_savings_minor,
            escape_csv_field(&Self::format_minor(summary.total_income_minor)),
            escape_csv_field(&Self::format_minor(summary.total_expense_minor)),
            escape_csv_field(&Self::format_minor(summary.net_savings_minor)),
        );
        out.push_str(&total_row);

        let mut hasher = Sha256::new();
        hasher.update(out.as_bytes());
        let hash = hex::encode(hasher.finalize());

        (out, hash)
    }
}
