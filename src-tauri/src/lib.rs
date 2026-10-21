pub mod commands;
pub mod db;
pub mod security;
pub mod state;
pub mod supervisor;

use db::Database;
use state::AppState;

pub fn run() {
    let app_state = AppState::new();

    // In local development / portable execution, initialize database
    let db_dir = directories::ProjectDirs::from("com", "theshriks", "opendeck")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let _ = std::fs::create_dir_all(&db_dir);
    let db_path = db_dir.join("opendeck_workspace.db");

    if let Ok(db) = Database::open(&db_path) {
        *app_state.db.lock().unwrap() = Some(db);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenDeck application");
}
