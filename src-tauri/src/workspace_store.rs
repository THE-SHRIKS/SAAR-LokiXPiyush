//! Per-workspace encrypted storage layout.
//!
//! Each workspace lives in its own SQLCipher database and its own blob directory, both under
//! `<data_dir>/workspaces/`. The catalog file lists only what is needed to unlock a workspace:
//! an opaque id, the Argon2id salt (not secret), the creation time, and the file names. Display
//! names, records, and chat history exist only inside the encrypted per-workspace database, so
//! the catalog reveals neither financial content nor workspace names.

use crate::security::kdf::KEY_LEN;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const CATALOG_FILE: &str = "catalog.json";
pub const WORKSPACE_DIR: &str = "workspaces";
pub const DB_EXTENSION: &str = "saardb";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub salt_hex: String,
    pub created_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub version: u32,
    pub workspaces: Vec<CatalogEntry>,
}

impl Catalog {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join(CATALOG_FILE);
        if !path.exists() {
            return Ok(Catalog { version: 1, workspaces: Vec::new() });
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("Catalog unreadable: {}", e))?;
        serde_json::from_str(&raw).map_err(|e| format!("Catalog is corrupt: {}", e))
    }

    /// Writes the catalog atomically: write a temporary file, then rename it into place.
    pub fn save(&self, data_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        let target = data_dir.join(CATALOG_FILE);
        let tmp = data_dir.join(format!("{}.tmp", CATALOG_FILE));
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&tmp, raw).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &target).map_err(|e| e.to_string())
    }

    pub fn find(&self, id: &str) -> Option<&CatalogEntry> {
        self.workspaces.iter().find(|w| w.id == id)
    }
}

/// Path of the encrypted database for a workspace.
pub fn db_path(data_dir: &Path, workspace_id: &str) -> PathBuf {
    data_dir
        .join(WORKSPACE_DIR)
        .join(format!("{}.{}", workspace_id, DB_EXTENSION))
}

/// Directory holding the encrypted document blobs for a workspace.
pub fn blob_dir(data_dir: &Path, workspace_id: &str) -> PathBuf {
    data_dir.join(WORKSPACE_DIR).join(format!("{}.blobs", workspace_id))
}

/// Derives the SQLCipher database key from the workspace master key.
///
/// The master key is the Argon2id output for this passphrase and salt. The database key is
/// domain-separated from it, so the database key and the blob key are never the same value,
/// and the master key itself is never handed to SQLCipher.
pub fn derive_db_key(master_key: &[u8; KEY_LEN]) -> [u8; KEY_LEN] {
    let mut hasher = Sha256::new();
    hasher.update(b"saar.workspace.database-key.v1");
    hasher.update(master_key);
    let digest = hasher.finalize();
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}

pub fn salt_to_hex(salt: &[u8]) -> String {
    hex::encode(salt)
}

pub fn salt_from_hex(s: &str) -> Result<Vec<u8>, String> {
    hex::decode(s).map_err(|_| "Catalog salt is malformed.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_key_is_domain_separated_from_master_key() {
        let master = [7u8; KEY_LEN];
        let db_key = derive_db_key(&master);
        assert_ne!(db_key, master);
        assert_eq!(db_key, derive_db_key(&master), "derivation must be deterministic");
        assert_ne!(db_key, derive_db_key(&[8u8; KEY_LEN]), "different master, different key");
    }

    #[test]
    fn test_catalog_round_trip_contains_no_display_names() {
        let dir = std::env::temp_dir().join(format!("saar_catalog_{}", uuid::Uuid::new_v4()));
        let catalog = Catalog {
            version: 1,
            workspaces: vec![CatalogEntry {
                id: "ws-1".into(),
                salt_hex: salt_to_hex(&[1u8; 16]),
                created_at: "2026-10-24T00:00:00Z".into(),
            }],
        };
        catalog.save(&dir).unwrap();
        let raw = fs::read_to_string(dir.join(CATALOG_FILE)).unwrap();
        assert!(!raw.contains("displayName"), "catalog must not carry workspace names");
        assert_eq!(Catalog::load(&dir).unwrap().workspaces, catalog.workspaces);
        let _ = fs::remove_dir_all(&dir);
    }
}
