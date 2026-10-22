use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;
use uuid::Uuid;

use crate::ingestion::{DocumentOrchestrator, IngestResult, NormalizedField, NormalizedLineItem};
use crate::security::KeyDeriver;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub app_version: String,
    pub platform: String,
    pub offline: bool,
    pub ocr_ready: bool,
    pub llm_ready: bool,
    pub llm_model_id: String,
    pub active_workspace_id: Option<String>,
    pub database_encrypted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub id: String,
    pub display_name: String,
    pub base_currency: String,
    pub schema_version: i32,
    pub created_at: String,
    pub is_unlocked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecord {
    pub id: String,
    pub workspace_id: String,
    pub file_name: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub doc_type: String,
    pub page_count: i32,
    pub status: String,
    pub imported_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRecord {
    pub id: String,
    pub workspace_id: String,
    pub source_doc_id: Option<String>,
    pub posted_date: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub external_ref: Option<String>,
    pub matched_invoice_id: Option<String>,
    pub review_state: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetails {
    pub document: DocumentRecord,
    pub fields: Vec<NormalizedField>,
    pub line_items: Vec<NormalizedLineItem>,
    pub invoice_total_minor: Option<i64>,
}

#[tauri::command]
pub fn get_system_status(state: State<AppState>) -> SystemStatus {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    SystemStatus {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        offline: true, // Runtime strictly offline
        ocr_ready: true,
        llm_ready: true,
        llm_model_id: "qwen-2.5-3b-instruct-q3_k_m".to_string(),
        active_workspace_id: ws_guard.clone(),
        database_encrypted: true,
    }
}

#[tauri::command]
pub fn create_workspace(
    state: State<AppState>,
    display_name: String,
    passphrase: String,
    base_currency: String,
) -> Result<WorkspaceInfo, String> {
    if passphrase.trim().is_empty() {
        return Err("Passphrase cannot be empty.".to_string());
    }

    let salt = KeyDeriver::generate_salt();
    let master_key = KeyDeriver::derive_master_key(&passphrase, &salt)?;
    let verification_token = KeyDeriver::compute_verification_token(&master_key);

    let workspace_id = Uuid::new_v4().to_string();

    let db_guard = state.db.lock().unwrap();
    if let Some(db) = db_guard.as_ref() {
        db.create_workspace(
            &workspace_id,
            &display_name,
            &base_currency,
            &salt,
            &verification_token,
        )
        .map_err(|e| format!("Database error: {}", e))?;
    }

    // Update active in-memory state
    *state.active_workspace_id.lock().unwrap() = Some(workspace_id.clone());
    *state.active_master_key.lock().unwrap() = Some(master_key);

    Ok(WorkspaceInfo {
        id: workspace_id,
        display_name,
        base_currency,
        schema_version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        is_unlocked: true,
    })
}

#[tauri::command]
pub fn unlock_workspace(
    state: State<AppState>,
    workspace_id: String,
    passphrase: String,
) -> Result<WorkspaceInfo, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let (salt, expected_token) = db
        .get_workspace_auth_params(&workspace_id)
        .map_err(|_| "Workspace not found.".to_string())?;

    let master_key = KeyDeriver::verify_passphrase(&passphrase, &salt, &expected_token)?;

    // Set active unlocked state
    drop(db_guard);
    *state.active_workspace_id.lock().unwrap() = Some(workspace_id.clone());
    *state.active_master_key.lock().unwrap() = Some(master_key);

    Ok(WorkspaceInfo {
        id: workspace_id,
        display_name: "Active Workspace".to_string(),
        base_currency: "USD".to_string(),
        schema_version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        is_unlocked: true,
    })
}

#[tauri::command]
pub fn lock_workspace(state: State<AppState>) -> Result<(), String> {
    *state.active_workspace_id.lock().unwrap() = None;
    *state.active_master_key.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn list_documents(state: State<AppState>) -> Result<Vec<DocumentRecord>, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = match ws_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(Vec::new()),
    };
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let mut stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, workspace_id, file_name, file_size_bytes, sha256, doc_type, page_count, status, imported_at
             FROM documents WHERE workspace_id = ?1 ORDER BY imported_at DESC;",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![ws_id], |row| {
            Ok(DocumentRecord {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                file_name: row.get(2)?,
                file_size_bytes: row.get(3)?,
                sha256: row.get(4)?,
                doc_type: row.get(5)?,
                page_count: row.get(6)?,
                status: row.get(7)?,
                imported_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn list_transactions(state: State<AppState>) -> Result<Vec<TransactionRecord>, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = match ws_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(Vec::new()),
    };
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let mut stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, workspace_id, source_doc_id, posted_date, description, amount_minor, currency, external_ref, matched_invoice_id, review_state
             FROM transactions WHERE workspace_id = ?1 ORDER BY posted_date DESC;",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![ws_id], |row| {
            Ok(TransactionRecord {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                source_doc_id: row.get(2)?,
                posted_date: row.get(3)?,
                description: row.get(4)?,
                amount_minor: row.get(5)?,
                currency: row.get(6)?,
                external_ref: row.get(7)?,
                matched_invoice_id: row.get(8)?,
                review_state: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn ingest_file(state: State<AppState>, file_path: String) -> Result<IngestResult, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard
        .as_ref()
        .ok_or("No workspace is currently unlocked.")?
        .clone();
    drop(ws_guard);

    let key_guard = state.active_master_key.lock().unwrap();
    let master_key = key_guard
        .as_ref()
        .ok_or("Workspace master key is locked.")?
        .clone();
    drop(key_guard);

    let mut db_guard = state.db.lock().unwrap();
    let db = db_guard.as_mut().ok_or("Database uninitialized.")?;

    let blob_guard = state.blob_store.lock().unwrap();
    let blob_store = blob_guard.as_ref().ok_or("BlobStore uninitialized.")?;

    let ocr_guard = state.ocr_client.lock().unwrap();
    let ocr_client = ocr_guard.as_ref().ok_or("OcrClient uninitialized.")?;

    DocumentOrchestrator::ingest_file(
        db,
        blob_store,
        ocr_client,
        &master_key,
        &ws_id,
        Path::new(&file_path),
        "USD",
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_document_details(
    state: State<AppState>,
    document_id: String,
) -> Result<DocumentDetails, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    // 1. Fetch document record
    let document: DocumentRecord = db
        .conn_ref()
        .query_row(
            "SELECT id, workspace_id, file_name, file_size_bytes, sha256, doc_type, page_count, status, imported_at
             FROM documents WHERE id = ?1 LIMIT 1;",
            params![document_id],
            |row| {
                Ok(DocumentRecord {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    file_name: row.get(2)?,
                    file_size_bytes: row.get(3)?,
                    sha256: row.get(4)?,
                    doc_type: row.get(5)?,
                    page_count: row.get(6)?,
                    status: row.get(7)?,
                    imported_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| format!("Document not found: {}", e))?;

    // 2. Fetch document fields
    let mut field_stmt = db
        .conn_ref()
        .prepare(
            "SELECT field_key, original_text, normalized_value, confidence, page_num, bounding_box_json
             FROM document_fields WHERE document_id = ?1 ORDER BY page_num ASC;",
        )
        .map_err(|e| e.to_string())?;

    let field_rows = field_stmt
        .query_map(params![document_id], |row| {
            Ok(NormalizedField {
                field_key: row.get(0)?,
                original_text: row.get(1)?,
                normalized_value: row.get(2)?,
                confidence: row.get(3)?,
                page_num: row.get(4)?,
                bounding_box_json: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut fields = Vec::new();
    for f in field_rows {
        fields.push(f.map_err(|e| e.to_string())?);
    }

    // 3. Fetch invoice and line items
    let mut invoice_total_minor = None;
    let mut line_items = Vec::new();

    let inv_opt: Result<(String, i64), _> = db.conn_ref().query_row(
        "SELECT id, total_minor FROM invoices WHERE document_id = ?1 LIMIT 1;",
        params![document_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    if let Ok((inv_id, total)) = inv_opt {
        invoice_total_minor = Some(total);

        let mut item_stmt = db
            .conn_ref()
            .prepare("SELECT description, amount_minor FROM line_items WHERE invoice_id = ?1;")
            .map_err(|e| e.to_string())?;

        let item_rows = item_stmt
            .query_map(params![inv_id], |row| {
                Ok(NormalizedLineItem {
                    description: row.get(0)?,
                    amount_minor: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for it in item_rows {
            line_items.push(it.map_err(|e| e.to_string())?);
        }
    }

    Ok(DocumentDetails {
        document,
        fields,
        line_items,
        invoice_total_minor,
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationResult {
    pub links: Vec<crate::finance::matching::ProposedLink>,
    pub anomalies: Vec<crate::finance::anomaly::FinancialAnomaly>,
}

#[tauri::command]
pub fn run_reconciliation(state: State<AppState>) -> Result<ReconciliationResult, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard
        .as_ref()
        .ok_or("No workspace is currently unlocked.")?
        .clone();
    drop(ws_guard);

    let mut db_guard = state.db.lock().unwrap();
    let db = db_guard.as_mut().ok_or("Database uninitialized.")?;

    // 1. Fetch invoices in workspace
    let mut inv_stmt = db
        .conn_ref()
        .prepare(
            "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
             FROM invoices i
             JOIN documents d ON i.document_id = d.id
             WHERE d.workspace_id = ?1;",
        )
        .map_err(|e| e.to_string())?;

    let inv_rows = inv_stmt
        .query_map(params![ws_id], |row| {
            Ok(crate::finance::matching::InvoiceCandidate {
                id: row.get(0)?,
                vendor_name: row.get(1)?,
                invoice_number: row.get(2)?,
                invoice_date: row.get(3)?,
                total_minor: row.get(4)?,
                currency: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut invoices = Vec::new();
    for inv in inv_rows {
        invoices.push(inv.map_err(|e| e.to_string())?);
    }

    // 2. Fetch transactions in workspace
    let mut tx_stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, posted_date, description, amount_minor, currency, external_ref
             FROM transactions
             WHERE workspace_id = ?1;",
        )
        .map_err(|e| e.to_string())?;

    let tx_rows = tx_stmt
        .query_map(params![ws_id], |row| {
            Ok(crate::finance::matching::TransactionCandidate {
                id: row.get(0)?,
                posted_date: row.get(1)?,
                description: row.get(2)?,
                amount_minor: row.get(3)?,
                currency: row.get(4)?,
                external_ref: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut transactions = Vec::new();
    for tx in tx_rows {
        transactions.push(tx.map_err(|e| e.to_string())?);
    }

    // 3. Run deterministic reconciliation matching
    let links = crate::finance::matching::ReconciliationMatcher::match_records(&invoices, &transactions);

    // 4. Run deterministic anomaly detection
    let anomalies = crate::finance::anomaly::AnomalyDetector::detect_anomalies(&invoices, &transactions);

    // 5. Persist proposed links to database
    let now = chrono::Utc::now().to_rfc3339();
    for link in &links {
        let _ = db.conn_ref().execute(
            "INSERT OR IGNORE INTO invoice_payment_links
             (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            params![
                link.id,
                ws_id,
                link.invoice_id,
                link.transaction_id,
                link.matched_amount_minor,
                link.confidence_score,
                link.match_type,
                link.explanation,
                link.status,
                now
            ],
        );
    }

    // Persist anomalies
    for anom in &anomalies {
        let _ = db.conn_ref().execute(
            "INSERT OR IGNORE INTO reconciliation_anomalies
             (id, workspace_id, entity_type, entity_id, anomaly_type, description, severity, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            params![
                anom.id,
                ws_id,
                anom.entity_type,
                anom.entity_id,
                anom.anomaly_type,
                anom.description,
                anom.severity,
                now
            ],
        );
    }

    Ok(ReconciliationResult { links, anomalies })
}

#[tauri::command]
pub fn list_reconciliation_links(state: State<AppState>) -> Result<Vec<crate::finance::matching::ProposedLink>, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = match ws_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(Vec::new()),
    };
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let mut stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status
             FROM invoice_payment_links WHERE workspace_id = ?1 ORDER BY created_at DESC;",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![ws_id], |row| {
            Ok(crate::finance::matching::ProposedLink {
                id: row.get(0)?,
                invoice_id: row.get(1)?,
                transaction_id: row.get(2)?,
                matched_amount_minor: row.get(3)?,
                confidence_score: row.get(4)?,
                match_type: row.get(5)?,
                explanation: row.get(6)?,
                status: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn list_anomalies(state: State<AppState>) -> Result<Vec<crate::finance::anomaly::FinancialAnomaly>, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = match ws_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(Vec::new()),
    };
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let mut stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, entity_type, entity_id, anomaly_type, description, severity
             FROM reconciliation_anomalies WHERE workspace_id = ?1 ORDER BY created_at DESC;",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![ws_id], |row| {
            Ok(crate::finance::anomaly::FinancialAnomaly {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                anomaly_type: row.get(3)?,
                description: row.get(4)?,
                severity: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn approve_reconciliation_link(
    state: State<AppState>,
    link_id: String,
) -> Result<(), String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let mut db_guard = state.db.lock().unwrap();
    let db = db_guard.as_mut().ok_or("Database uninitialized.")?;

    crate::finance::review::ReviewService::approve_link(db, &ws_id, &link_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_reconciliation_link(
    state: State<AppState>,
    link_id: String,
) -> Result<(), String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    crate::finance::review::ReviewService::reject_link(db, &ws_id, &link_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn correct_transaction(
    state: State<AppState>,
    transaction_id: String,
    new_description: String,
) -> Result<(), String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    crate::finance::review::ReviewService::correct_transaction(db, &ws_id, &transaction_id, &new_description)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_financial_summary(
    state: State<AppState>,
    currency: String,
) -> Result<crate::finance::aggregates::FinancialSummary, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let mut stmt = db
        .conn_ref()
        .prepare(
            "SELECT id, posted_date, description, amount_minor, currency, external_ref
             FROM transactions WHERE workspace_id = ?1;",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![ws_id], |row| {
            Ok(crate::finance::matching::TransactionCandidate {
                id: row.get(0)?,
                posted_date: row.get(1)?,
                description: row.get(2)?,
                amount_minor: row.get(3)?,
                currency: row.get(4)?,
                external_ref: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut transactions = Vec::new();
    for r in rows {
        transactions.push(r.map_err(|e| e.to_string())?);
    }

    Ok(crate::finance::aggregates::FinancialAggregator::compute_summary(&transactions, &currency))
}

#[tauri::command]
pub fn chat_grounded(
    state: State<AppState>,
    query: String,
) -> Result<crate::ai::GroundedChatResponse, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    // Defense: Detect adversarial prompt injection
    if crate::ai::AdversarialDefense::contains_prompt_injection(&query) {
        return Ok(crate::ai::GroundedChatResponse {
            answer: "Adversarial prompt injection detected and quarantined. The query was rejected by local safety boundaries.".to_string(),
            intent: crate::ai::QueryIntent::GeneralFinancial,
            citations: Vec::new(),
            grounded: false,
            verification_note: "Quarantined by AdversarialDefense engine.".to_string(),
        });
    }

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    // Build fact packet from active workspace records
    let fact_packet = crate::ai::GroundingEngine::build_fact_packet(db.conn_ref(), &ws_id, &query, "USD")
        .map_err(|e| format!("Failed to build fact packet: {}", e))?;
    drop(db_guard);

    let (system_prompt, user_prompt) = crate::ai::GroundingEngine::build_prompts(&fact_packet);

    let llm_guard = state.llm_engine.lock().unwrap();
    let llm = llm_guard.as_ref().ok_or("Local LLM engine uninitialized.")?;

    let raw_answer = llm
        .generate(&system_prompt, &user_prompt, None)
        .map_err(|e| format!("Local model inference failed: {}", e))?;
    drop(llm_guard);

    Ok(crate::ai::GroundingEngine::validate_and_parse_response(&fact_packet, &raw_answer))
}

#[tauri::command]
pub fn classify_transaction(
    state: State<AppState>,
    transaction_id: String,
) -> Result<crate::ai::CategoryProposal, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    let (desc, amount_minor, currency): (String, i64, String) = db
        .conn_ref()
        .query_row(
            "SELECT description, amount_minor, currency FROM transactions WHERE id = ?1 AND workspace_id = ?2;",
            params![transaction_id, ws_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| format!("Transaction not found: {}", e))?;

    let llm_guard = state.llm_engine.lock().unwrap();
    let llm = llm_guard.as_ref().ok_or("Local LLM engine uninitialized.")?;

    let proposal = crate::ai::TransactionClassifier::classify_transaction(
        llm,
        &transaction_id,
        &desc,
        amount_minor,
        &currency,
    )?;
    drop(llm_guard);

    crate::ai::TransactionClassifier::save_proposal(db.conn_ref(), &ws_id, &proposal)
        .map_err(|e| format!("Failed to save category proposal: {}", e))?;

    Ok(proposal)
}

#[tauri::command]
pub fn approve_category(
    state: State<AppState>,
    proposal_id: String,
) -> Result<(), String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = ws_guard.as_ref().ok_or("No workspace is unlocked.")?.clone();
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    crate::ai::TransactionClassifier::approve_category(db.conn_ref(), &ws_id, &proposal_id)
        .map_err(|e| e.to_string())?;

    let audit_json = serde_json::json!({
        "proposal_id": proposal_id,
        "action": "APPROVE_CATEGORY"
    }).to_string();

    db.record_audit_event(&ws_id, "transaction_category", &proposal_id, "APPROVE_CATEGORY", &audit_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_workspace(
    state: State<AppState>,
    query: String,
) -> Result<Vec<crate::ai::SearchResult>, String> {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    let ws_id = match ws_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(Vec::new()),
    };
    drop(ws_guard);

    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("Database uninitialized.")?;

    crate::ai::SearchEngine::search(db.conn_ref(), &ws_id, &query, 20)
        .map_err(|e| e.to_string())
}
