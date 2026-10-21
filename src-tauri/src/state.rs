use std::sync::{Arc, Mutex};
use crate::db::Database;
use zeroize::Zeroizing;

pub struct AppState {
    pub db: Arc<Mutex<Option<Database>>>,
    pub active_workspace_id: Arc<Mutex<Option<String>>>,
    pub active_master_key: Arc<Mutex<Option<Zeroizing<[u8; 32]>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
            active_workspace_id: Arc::new(Mutex::new(None)),
            active_master_key: Arc::new(Mutex::new(None)),
        }
    }
}
