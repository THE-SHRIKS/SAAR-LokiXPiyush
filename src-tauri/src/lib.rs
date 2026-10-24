pub mod ai;
pub mod commands;
pub mod db;
pub mod export;
pub mod finance;
pub mod ingestion;
pub mod security;
pub mod state;
pub mod supervisor;
pub mod workspace_migration;
pub mod workspace_store;
#[cfg(test)]
pub mod integration_tests;

use ai::{LlmConfig, LlmEngine};
use ingestion::OcrClient;
use state::AppState;

pub fn run() {
    // Application data root. No database is opened here: each workspace has its own encrypted
    // database and is opened only after its passphrase is verified (see `unlock_workspace`).
    // SAAR_DATA_DIR relocates all workspace data (portable installs, and isolated test runs that
    // must not touch the user's real data folder).
    let data_dir = std::env::var_os("SAAR_DATA_DIR")
        .map(std::path::PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| {
            directories::ProjectDirs::from("com", "theshriks", "saar")
                .map(|dirs| dirs.data_local_dir().to_path_buf())
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let _ = std::fs::create_dir_all(&data_dir);

    let app_state = AppState::new(data_dir);

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
            commands::export_reconciliation_report,
            commands::list_report_exports,
            commands::create_encrypted_backup,
            commands::restore_encrypted_backup,
            commands::list_workspaces,
            commands::list_chat_threads,
            commands::create_chat_thread,
            commands::list_chat_messages,
            commands::rename_chat_thread,
            commands::set_chat_thread_pinned,
            commands::delete_chat_thread,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SAAR application");
}
