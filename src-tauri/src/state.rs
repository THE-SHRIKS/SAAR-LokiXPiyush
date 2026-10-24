use crate::db::Database;
use crate::ingestion::OcrClient;
use crate::security::BlobStore;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub struct AppState {
    pub db: Arc<Mutex<Option<Database>>>,
    pub blob_store: Arc<Mutex<Option<BlobStore>>>,
    pub ocr_client: Arc<Mutex<Option<OcrClient>>>,
    pub llm_engine: Arc<Mutex<Option<crate::ai::LlmEngine>>>,
    pub active_workspace_id: Arc<Mutex<Option<String>>>,
    pub active_master_key: Arc<Mutex<Option<Zeroizing<[u8; 32]>>>>,
    /// Application data root; per-workspace files live under `<data_dir>/workspaces`.
    pub data_dir: PathBuf,
    /// True only while the open database was opened with an encryption key.
    pub database_encrypted: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            database_encrypted: Arc::new(Mutex::new(false)),
            db: Arc::new(Mutex::new(None)),
            blob_store: Arc::new(Mutex::new(None)),
            ocr_client: Arc::new(Mutex::new(None)),
            llm_engine: Arc::new(Mutex::new(None)),
            active_workspace_id: Arc::new(Mutex::new(None)),
            active_master_key: Arc::new(Mutex::new(None)),
        }
    }
}
