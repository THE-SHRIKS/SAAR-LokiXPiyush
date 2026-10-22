pub mod ai;
pub mod commands;
pub mod db;
pub mod finance;
pub mod ingestion;
pub mod security;
pub mod state;
pub mod supervisor;

use ai::{LlmConfig, LlmEngine};
use db::Database;
use ingestion::OcrClient;
use security::BlobStore;
use state::AppState;

pub fn run() {
    let app_state = AppState::new();

    // In local development / portable execution, initialize database and blob storage
    let data_dir = directories::ProjectDirs::from("com", "theshriks", "opendeck")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let _ = std::fs::create_dir_all(&data_dir);
    let db_path = data_dir.join("opendeck_workspace.db");
    let blob_path = data_dir.join("blobs");

    if let Ok(db) = Database::open(&db_path) {
        *app_state.db.lock().unwrap() = Some(db);
    }

    if let Ok(store) = BlobStore::new(&blob_path) {
        *app_state.blob_store.lock().unwrap() = Some(store);
    }

    if let Ok(ocr) = OcrClient::new() {
        *app_state.ocr_client.lock().unwrap() = Some(ocr);
    }

    if let Ok(llm) = LlmEngine::new(LlmConfig::default()) {
        *app_state.llm_engine.lock().unwrap() = Some(llm);
    }

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_system_status,
            commands::create_workspace,
            commands::unlock_workspace,
            commands::lock_workspace,
            commands::list_documents,
            commands::list_transactions,
            commands::ingest_file,
            commands::get_document_details,
            commands::get_document_preview,
            commands::run_reconciliation,
            commands::list_reconciliation_links,
            commands::list_anomalies,
            commands::approve_reconciliation_link,
            commands::reject_reconciliation_link,
            commands::correct_transaction,
            commands::list_audit_log,
            commands::get_financial_summary,
            commands::chat_grounded,
            commands::classify_transaction,
            commands::approve_category,
            commands::search_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenDeck application");
}
