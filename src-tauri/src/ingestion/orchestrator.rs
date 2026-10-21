use std::fs;
use std::path::Path;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Database;
use crate::ingestion::csv_mapper::CsvTransactionMapper;
use crate::ingestion::normalizer::FieldNormalizer;
use crate::ingestion::ocr_client::OcrClient;
use crate::security::BlobStore;

const MAX_FILE_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50 MiB limit

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IngestResult {
    Success {
        document_id: String,
        file_name: String,
        doc_type: String,
        page_count: i32,
        records_extracted: usize,
    },
    Duplicate {
        existing_document_id: String,
        file_name: String,
        sha256: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("File exceeds maximum allowable size of 50 MB ({size} bytes)")]
    FileTooLarge { size: u64 },
    #[error("Unsupported or malicious file type signature")]
    UnsupportedFileType,
    #[error("OCR extraction error: {0}")]
    Ocr(String),
    #[error("CSV parsing error: {0}")]
    Csv(String),
    #[error("Storage error: {0}")]
    Blob(String),
}

pub struct DocumentOrchestrator;

impl DocumentOrchestrator {
    /// Detects file type from magic bytes header
    pub fn detect_doc_type(bytes: &[u8], file_name: &str) -> Result<&'static str, IngestionError> {
        let ext = Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if bytes.starts_with(b"%PDF-") {
            Ok("PDF")
        } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Ok("PNG")
        } else if bytes.starts_with(b"\xff\xd8\xff") {
            Ok("JPEG")
        } else if ext == "csv" {
            // Validate that CSV bytes are valid text
            if std::str::from_utf8(bytes).is_ok() {
                Ok("CSV")
            } else {
                Err(IngestionError::UnsupportedFileType)
            }
        } else {
            Err(IngestionError::UnsupportedFileType)
        }
    }

    /// Primary end-to-end ingestion pipeline
    pub fn ingest_file(
        db: &mut Database,
        blob_store: &BlobStore,
        ocr_client: &OcrClient,
        master_key: &[u8; 32],
        workspace_id: &str,
        file_path: &Path,
        base_currency: &str,
    ) -> Result<IngestResult, IngestionError> {
        let metadata = fs::metadata(file_path)?;
        if metadata.len() > MAX_FILE_SIZE_BYTES {
            return Err(IngestionError::FileTooLarge { size: metadata.len() });
        }

        let file_bytes = fs::read(file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown_file")
            .to_string();

        let doc_type = Self::detect_doc_type(&file_bytes, &file_name)?;

        // 1. Compute SHA-256 for deduplication
        let (sha256_hex, _) = BlobStore::compute_sha256(&file_bytes);

        // 2. Check for duplicate in database for this workspace
        let existing_doc: Option<String> = db
            .conn_ref()
            .query_row(
                "SELECT id FROM documents WHERE workspace_id = ?1 AND sha256 = ?2 LIMIT 1;",
                params![workspace_id, sha256_hex],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing_doc {
            return Ok(IngestResult::Duplicate {
                existing_document_id: existing_id,
                file_name,
                sha256: sha256_hex,
            });
        }

        // 3. Store encrypted blob on disk
        let blob_res = blob_store
            .store_blob(master_key, &file_bytes)
            .map_err(|e| IngestionError::Blob(e.to_string()))?;

        let doc_id = Uuid::new_v4().to_string();

        // 4. Initial database registration with PROCESSING status
        db.conn_ref().execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, page_count, status, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'PROCESSING', datetime('now'));",
            params![
                doc_id,
                workspace_id,
                file_name,
                metadata.len() as i64,
                blob_res.sha256_hex,
                doc_type,
                1
            ],
        )?;

        // 5. Ingestion processing based on document type
        let records_extracted = if doc_type == "CSV" {
            let csv_text = std::str::from_utf8(&file_bytes)
                .map_err(|_| IngestionError::UnsupportedFileType)?;

            let transactions = CsvTransactionMapper::parse_csv_content(csv_text, base_currency)
                .map_err(|e| IngestionError::Csv(e.to_string()))?;

            let tx_count = transactions.len();

            let conn = db.connection_mut();
            let tx = conn.transaction()?;

            for tx_item in &transactions {
                let tx_id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO transactions (id, workspace_id, source_doc_id, posted_date, description, amount_minor, currency, external_ref, review_state)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'UNREVIEWED');",
                    params![
                        tx_id,
                        workspace_id,
                        doc_id,
                        tx_item.posted_date,
                        tx_item.description,
                        tx_item.amount_minor,
                        tx_item.currency,
                        tx_item.external_ref,
                    ],
                )?;
            }

            tx.execute(
                "UPDATE documents SET status = 'READY', page_count = 1 WHERE id = ?1;",
                params![doc_id],
            )?;

            tx.commit()?;
            tx_count
        } else {
            // PDF or Image document
            let file_ext = match doc_type {
                "PDF" => "pdf",
                "PNG" => "png",
                _ => "jpg",
            };

            // Create temporary decrypted plaintext for supervised sidecar
            let temp_guard = blob_store
                .create_temp_plaintext(master_key, &blob_res.sha256_hex, file_ext)
                .map_err(|e| IngestionError::Blob(e.to_string()))?;

            let ocr_res = ocr_client
                .extract_document(temp_guard.path())
                .map_err(|e| IngestionError::Ocr(e.to_string()))?;

            // Drop temporary file guard immediately to zeroize and delete plaintext
            drop(temp_guard);

            let normalized = FieldNormalizer::normalize_ocr(&ocr_res, base_currency);

            let conn = db.connection_mut();
            let tx = conn.transaction()?;

            // Save document fields
            for f in &normalized.fields {
                let f_id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO document_fields (id, document_id, field_key, original_text, normalized_value, confidence, page_num, bounding_box_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                    params![
                        f_id,
                        doc_id,
                        f.field_key,
                        f.original_text,
                        f.normalized_value,
                        f.confidence,
                        f.page_num,
                        f.bounding_box_json,
                    ],
                )?;
            }

            // Save invoice record
            let invoice_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO invoices (id, document_id, vendor_name, invoice_number, invoice_date, due_date, currency, total_minor, tax_minor, review_state)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'UNREVIEWED');",
                params![
                    invoice_id,
                    doc_id,
                    normalized.vendor_name,
                    normalized.invoice_number,
                    normalized.invoice_date,
                    normalized.due_date,
                    normalized.currency,
                    normalized.total_minor,
                    normalized.tax_minor,
                ],
            )?;

            // Save line items
            for item in &normalized.line_items {
                let item_id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO line_items (id, invoice_id, description, amount_minor)
                     VALUES (?1, ?2, ?3, ?4);",
                    params![item_id, invoice_id, item.description, item.amount_minor],
                )?;
            }

            tx.execute(
                "UPDATE documents SET status = 'READY', page_count = ?2 WHERE id = ?1;",
                params![doc_id, ocr_res.page_count],
            )?;

            tx.commit()?;
            normalized.fields.len()
        };

        // Record immutable audit trail
        let audit_json = serde_json::json!({
            "file_name": file_name,
            "doc_type": doc_type,
            "sha256": blob_res.sha256_hex,
            "records_extracted": records_extracted,
        })
        .to_string();

        db.record_audit_event(workspace_id, "document", &doc_id, "INGEST", &audit_json)?;

        Ok(IngestResult::Success {
            document_id: doc_id,
            file_name,
            doc_type: doc_type.to_string(),
            page_count: 1,
            records_extracted,
        })
    }
}
