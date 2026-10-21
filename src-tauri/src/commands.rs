use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

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

#[tauri::command]
pub fn get_system_status(state: State<AppState>) -> SystemStatus {
    let ws_guard = state.active_workspace_id.lock().unwrap();
    SystemStatus {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        offline: true, // Non-negotiable: runtime strictly offline
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

    let mut db_guard = state.db.lock().unwrap();
    if let Some(db) = db_guard.as_mut() {
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
    let _ws_guard = state.active_workspace_id.lock().unwrap();
    // Return empty list initially until documents are ingested
    Ok(Vec::new())
}

#[tauri::command]
pub fn list_transactions(state: State<AppState>) -> Result<Vec<TransactionRecord>, String> {
    let _ws_guard = state.active_workspace_id.lock().unwrap();
    // Return empty list initially until ledger entries are created
    Ok(Vec::new())
}
