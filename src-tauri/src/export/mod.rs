pub mod pdf_writer;
pub mod csv_exporter;

use std::fs;
use std::path::Path;
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::export::csv_exporter::CsvExporter;
use crate::export::pdf_writer::{ReconciliationReportRow, ReportGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub export_id: String,
    pub file_path: String,
    pub export_format: String,
    pub record_count: usize,
    pub record_snapshot_hash: String,
    pub file_size_bytes: usize,
    pub created_at: String,
}

pub struct ExportService;

impl ExportService {
    /// Query reconciliation rows from SQLite
    pub fn fetch_reconciliation_rows(
        conn: &Connection,
        workspace_id: &str,
    ) -> Result<(String, String, i64, i64, Vec<ReconciliationReportRow>)> {
        // Workspace name
        let workspace_name: String = conn
            .query_row(
                "SELECT display_name FROM workspaces WHERE id = ?1",
                [workspace_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "OpenDeck Default Workspace".to_string());

        // Inflow / Outflow totals
        let (total_income, total_expense): (i64, i64) = conn.query_row(
            "SELECT 
                COALESCE(SUM(CASE WHEN amount_minor > 0 THEN amount_minor ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN amount_minor < 0 THEN ABS(amount_minor) ELSE 0 END), 0)
             FROM transactions
             WHERE workspace_id = ?1",
            [workspace_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 0));

        let mut stmt = conn.prepare(
            "SELECT 
                t.id,
                t.posted_date,
                t.description,
                t.amount_minor,
                t.currency,
                COALESCE(t.source_doc_id, '') as source_doc_id,
                COALESCE(l.status, 'UNLINKED') as match_status,
                COALESCE(i.invoice_number, '') as invoice_number
             FROM transactions t
             LEFT JOIN invoice_payment_links l ON t.id = l.transaction_id
             LEFT JOIN invoices i ON l.invoice_id = i.id
             WHERE t.workspace_id = ?1
             ORDER BY t.posted_date DESC, t.id ASC",
        )?;

        let mut rows: Vec<ReconciliationReportRow> = Vec::new();
        let mut primary_currency = "USD".to_string();

        let mut query_rows = stmt.query([workspace_id])?;
        while let Some(r) = query_rows.next()? {
            let currency: String = r.get(4)?;
            if !currency.is_empty() {
                primary_currency = currency.clone();
            }

            rows.push(ReconciliationReportRow {
                date: r.get(1)?,
                description: r.get(2)?,
                amount_minor: r.get(3)?,
                currency,
                match_status: r.get(6)?,
                invoice_number: r.get(7)?,
                provenance_id: r.get(5)?,
            });
        }

        Ok((workspace_name, primary_currency, total_income, total_expense, rows))
    }

    /// Generate and save export file to disk, recording audit log and report_exports entry
    pub fn export_reconciliation(
        conn: &mut Connection,
        workspace_id: &str,
        format: &str,
        dest_path: &Path,
    ) -> Result<ExportResult> {
        let (workspace_name, currency, total_income, total_expense, rows) =
            Self::fetch_reconciliation_rows(conn, workspace_id)?;

        let record_count = rows.len();
        let net_savings = total_income - total_expense;

        // Determine period
        let period = if rows.is_empty() {
            "All Time (Empty)".to_string()
        } else {
            let min_date = rows.iter().map(|r| &r.date).min().cloned().unwrap_or_default();
            let max_date = rows.iter().map(|r| &r.date).max().cloned().unwrap_or_default();
            format!("{min_date} to {max_date}")
        };

        let export_id = Uuid::new_v4().to_string();
        let (bytes, hash) = match format.to_lowercase().as_str() {
            "pdf" => {
                let (pdf_bytes, pdf_hash) = ReportGenerator::generate_reconciliation_pdf(
                    &workspace_name,
                    &period,
                    &currency,
                    total_income,
                    total_expense,
                    net_savings,
                    &rows,
                );
                (pdf_bytes, pdf_hash)
            }
            "csv" => {
                let (csv_str, csv_hash) = CsvExporter::export_reconciliation(&rows);
                (csv_str.into_bytes(), csv_hash)
            }
            other => return Err(anyhow!("Unsupported export format '{other}'. Must be 'pdf' or 'csv'.")),
        };

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to destination
        fs::write(dest_path, &bytes)?;
        let file_size_bytes = bytes.len();
        let dest_str = dest_path.to_string_lossy().to_string();

        let format_upper = format.to_uppercase();

        let tx = conn.transaction()?;

        // Record in report_exports
        tx.execute(
            "INSERT INTO report_exports (
                id, workspace_id, period, currencies, export_format, file_path, record_snapshot_hash, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
            rusqlite::params![
                &export_id,
                &workspace_id,
                &period,
                &currency,
                &format_upper,
                &dest_str,
                &hash,
            ],
        )?;

        // Record immutable audit log
        let audit_id = Uuid::new_v4().to_string();
        let details = serde_json::json!({
            "export_id": export_id,
            "format": format_upper,
            "destination": dest_str,
            "record_count": record_count,
            "snapshot_hash": hash,
            "file_size_bytes": file_size_bytes
        });

        tx.execute(
            "INSERT INTO audit_log (
                id, workspace_id, entity_type, entity_id, action, details_json, timestamp
             ) VALUES (?1, ?2, 'REPORT_EXPORT', ?3, 'EXPORT_GENERATED', ?4, datetime('now'))",
            rusqlite::params![
                &audit_id,
                &workspace_id,
                &export_id,
                &details.to_string(),
            ],
        )?;

        tx.commit()?;

        let created_at: String = conn
            .query_row(
                "SELECT created_at FROM report_exports WHERE id = ?1",
                [&export_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "2026-10-22 00:00:00".to_string());

        Ok(ExportResult {
            export_id,
            file_path: dest_str,
            export_format: format.to_uppercase(),
            record_count,
            record_snapshot_hash: hash,
            file_size_bytes,
            created_at,
        })
    }
}
