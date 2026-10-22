use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::{rngs::OsRng, RngCore};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

const BACKUP_MAGIC: &[u8; 8] = b"OPENDECK";
const BACKUP_VERSION: u16 = 1;
const AAD_BYTES: &[u8] = b"OPENDECK_V1_BACKUP_CONTAINER";

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Authentication failed: invalid passphrase or tampered backup archive")]
    AuthenticationFailed,
    #[error("Invalid backup file header or unsupported version")]
    InvalidHeader,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    #[error("Path traversal security violation in backup blob path: {0}")]
    SecurityViolation(String),
    #[error("Backup error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub version: u32,
    pub app_version: String,
    pub created_at: String,
    pub workspace_id: String,
    pub workspace_name: String,
    pub db_sha256: String,
    pub blob_count: usize,
    pub total_raw_bytes: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BlobEntry {
    pub relative_path: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct BackupArchivePayload {
    pub manifest: BackupManifest,
    pub sqlite_db: Vec<u8>,
    pub blobs: Vec<BlobEntry>,
}

pub struct BackupService;

impl BackupService {
    /// Derive a 256-bit encryption key from user passphrase and salt using Argon2id
    fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], BackupError> {
        // Argon2id with 32MB memory, 3 iterations, 1 lane
        let params = Params::new(32 * 1024, 3, 1, Some(32))
            .map_err(|e| BackupError::Other(format!("Argon2 params error: {e}")))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| BackupError::Other(format!("KDF derivation failed: {e}")))?;

        Ok(key)
    }

    /// Recursively collect all blob files in the blob storage directory
    fn collect_blobs(dir: &Path, base_dir: &Path) -> Result<Vec<BlobEntry>, BackupError> {
        let mut entries = Vec::new();
        if !dir.exists() {
            return Ok(entries);
        }

        for item in fs::read_dir(dir)? {
            let item = item?;
            let path = item.path();
            if path.is_dir() {
                entries.extend(Self::collect_blobs(&path, base_dir)?);
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(base_dir)
                    .map_err(|e| BackupError::Other(e.to_string()))?
                    .to_string_lossy()
                    .replace('\\', "/");
                let data = fs::read(&path)?;
                entries.push(BlobEntry {
                    relative_path: rel,
                    data,
                });
            }
        }
        Ok(entries)
    }

    /// Create an encrypted authenticated .opendeck backup archive
    pub fn create_backup(
        db_path: &Path,
        blob_dir: &Path,
        workspace_id: &str,
        workspace_name: &str,
        dest_archive_path: &Path,
        passphrase: &str,
    ) -> Result<BackupManifest, BackupError> {
        if passphrase.trim().is_empty() {
            return Err(BackupError::Other(
                "Backup passphrase cannot be empty".to_string(),
            ));
        }

        // Flush and checkpoint WAL if SQLite database exists
        if db_path.exists() {
            if let Ok(conn) = Connection::open(db_path) {
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            }
        }

        let db_bytes = if db_path.exists() {
            fs::read(db_path)?
        } else {
            return Err(BackupError::Other(format!(
                "Database file not found at {}",
                db_path.display()
            )));
        };

        let mut hasher = Sha256::new();
        hasher.update(&db_bytes);
        let db_sha256 = hex::encode(hasher.finalize());

        // Collect blobs
        let blobs = Self::collect_blobs(blob_dir, blob_dir)?;
        let blob_count = blobs.len();

        let total_raw_bytes = db_bytes.len() + blobs.iter().map(|b| b.data.len()).sum::<usize>();

        let manifest = BackupManifest {
            version: 1,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            workspace_id: workspace_id.to_string(),
            workspace_name: workspace_name.to_string(),
            db_sha256,
            blob_count,
            total_raw_bytes,
        };

        let payload = BackupArchivePayload {
            manifest: manifest.clone(),
            sqlite_db: db_bytes,
            blobs,
        };

        let serialized_bytes = serde_json::to_vec(&payload)?;

        // Key derivation
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let key = Self::derive_key(passphrase, &salt)?;

        // AES-256-GCM encryption
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| BackupError::Other(format!("Cipher init error: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &serialized_bytes,
                    aad: AAD_BYTES,
                },
            )
            .map_err(|e| BackupError::Other(format!("Encryption error: {e}")))?;

        // Format output: MAGIC(8) + VER(2) + SALT(16) + NONCE(12) + CIPHERTEXT
        let mut out = Vec::with_capacity(38 + ciphertext.len());
        out.extend_from_slice(BACKUP_MAGIC);
        out.extend_from_slice(&BACKUP_VERSION.to_be_bytes());
        out.extend_from_slice(&salt);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);

        if let Some(parent) = dest_archive_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest_archive_path, out)?;

        Ok(manifest)
    }

    /// Restore an encrypted authenticated .opendeck backup archive into target directories
    pub fn restore_backup(
        archive_path: &Path,
        target_db_path: &Path,
        target_blob_dir: &Path,
        passphrase: &str,
    ) -> Result<BackupManifest, BackupError> {
        let archive_bytes = fs::read(archive_path)?;

        if archive_bytes.len() < 38 {
            return Err(BackupError::InvalidHeader);
        }

        // Verify Magic
        if &archive_bytes[0..8] != BACKUP_MAGIC {
            return Err(BackupError::InvalidHeader);
        }

        let version = u16::from_be_bytes([archive_bytes[8], archive_bytes[9]]);
        if version != BACKUP_VERSION {
            return Err(BackupError::InvalidHeader);
        }

        let salt = &archive_bytes[10..26];
        let nonce_bytes = &archive_bytes[26..38];
        let ciphertext = &archive_bytes[38..];

        // Key derivation
        let key = Self::derive_key(passphrase, salt)?;

        // AES-256-GCM authenticated decryption
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| BackupError::Other(format!("Cipher init error: {e}")))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let decrypted_bytes = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: AAD_BYTES,
                },
            )
            .map_err(|_| BackupError::AuthenticationFailed)?;

        // Deserialize archive payload
        let payload: BackupArchivePayload = serde_json::from_slice(&decrypted_bytes)?;

        // Verify SQLite database SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&payload.sqlite_db);
        let actual_db_hash = hex::encode(hasher.finalize());

        if actual_db_hash != payload.manifest.db_sha256 {
            return Err(BackupError::IntegrityCheckFailed(format!(
                "Database SHA-256 mismatch: expected {}, got {}",
                payload.manifest.db_sha256, actual_db_hash
            )));
        }

        // Unpack SQLite database
        if let Some(parent) = target_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target_db_path, &payload.sqlite_db)?;

        // Verify SQLite database integrity using rusqlite
        let conn = Connection::open(target_db_path)?;
        let integrity_result: String =
            conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
        if integrity_result != "ok" {
            return Err(BackupError::IntegrityCheckFailed(format!(
                "SQLite PRAGMA integrity_check reported: {integrity_result}"
            )));
        }

        // Unpack blobs with path traversal protection
        fs::create_dir_all(target_blob_dir)?;
        let canonical_base = target_blob_dir
            .canonicalize()
            .unwrap_or_else(|_| target_blob_dir.to_path_buf());

        for blob in &payload.blobs {
            let rel_path_clean = blob
                .relative_path
                .trim_start_matches('/')
                .trim_start_matches('\\');
            if rel_path_clean.contains("..") || rel_path_clean.contains(':') {
                return Err(BackupError::SecurityViolation(format!(
                    "Illegal relative path in blob entry: {}",
                    blob.relative_path
                )));
            }

            let full_path = canonical_base.join(rel_path_clean);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full_path, &blob.data)?;
        }

        Ok(payload.manifest)
    }
}
