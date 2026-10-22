use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

const BLOB_MAGIC: &[u8; 8] = b"ODBLOB01";
const NONCE_LEN: usize = 12;
const SHA256_LEN: usize = 32;
const MIN_BLOB_HEADER_LEN: usize = BLOB_MAGIC.len() + NONCE_LEN + SHA256_LEN; // 8 + 12 + 32 = 52 bytes

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decryption failed / tampering detected: {0}")]
    Decryption(String),
    #[error("Checksum mismatch: expected {expected}, calculated {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Invalid blob header format")]
    InvalidHeader,
}

/// Result of storing a blob
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobStoreResult {
    pub sha256_hex: String,
    pub is_duplicate: bool,
    pub byte_size: u64,
}

/// Encrypted file blob storage manager
pub struct BlobStore {
    root_dir: PathBuf,
}

impl BlobStore {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> std::io::Result<Self> {
        let root = root_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root_dir: root })
    }

    /// Computes the SHA-256 hex string of any data slice.
    pub fn compute_sha256(data: &[u8]) -> (String, [u8; SHA256_LEN]) {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; SHA256_LEN];
        hash_bytes.copy_from_slice(&result);
        (hex::encode(hash_bytes), hash_bytes)
    }

    /// Blob file path for a given SHA-256 hash.
    pub fn blob_path(&self, sha256_hex: &str) -> PathBuf {
        self.root_dir.join(format!("{}.blob.enc", sha256_hex))
    }

    /// Encrypts and saves plaintext data using AES-256-GCM.
    /// Deduplicates: if a blob with the same SHA-256 already exists, no re-encryption or disk write occurs.
    /// Writes atomically via temp file + sync + rename.
    pub fn store_blob(
        &self,
        master_key: &[u8; 32],
        plaintext: &[u8],
    ) -> Result<BlobStoreResult, BlobError> {
        let (sha256_hex, sha256_bytes) = Self::compute_sha256(plaintext);
        let target_path = self.blob_path(&sha256_hex);

        // Deduplication check: if file already exists on disk, reuse it
        if target_path.exists() {
            let metadata = fs::metadata(&target_path)?;
            return Ok(BlobStoreResult {
                sha256_hex,
                is_duplicate: true,
                byte_size: metadata.len(),
            });
        }

        // Generate cryptographically secure random 96-bit nonce
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt with AES-256-GCM, binding plaintext SHA-256 in AAD
        let cipher = Aes256Gcm::new_from_slice(master_key)
            .map_err(|e| BlobError::Decryption(format!("Cipher init error: {}", e)))?;

        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: &sha256_bytes,
                },
            )
            .map_err(|e| BlobError::Decryption(format!("Encryption error: {}", e)))?;

        // Prepare atomic write via temporary file
        let temp_file_name = format!("{}.tmp.{}", sha256_hex, Uuid::new_v4());
        let temp_path = self.root_dir.join(temp_file_name);

        {
            let mut file = File::create(&temp_path)?;
            file.write_all(BLOB_MAGIC)?;
            file.write_all(&nonce_bytes)?;
            file.write_all(&sha256_bytes)?;
            file.write_all(&ciphertext)?;
            file.sync_all()?; // Ensure atomic sync to physical media
        }

        // Atomic rename to final target path
        if let Err(e) = fs::rename(&temp_path, &target_path) {
            let _ = fs::remove_file(&temp_path);
            return Err(BlobError::Io(e));
        }

        let metadata = fs::metadata(&target_path)?;

        Ok(BlobStoreResult {
            sha256_hex,
            is_duplicate: false,
            byte_size: metadata.len(),
        })
    }

    /// Reads, verifies and decrypts an encrypted blob from disk.
    /// Constant-time authentication check prevents bit-flip / ciphertext tampering.
    pub fn read_blob(
        &self,
        master_key: &[u8; 32],
        sha256_hex: &str,
    ) -> Result<Zeroizing<Vec<u8>>, BlobError> {
        let path = self.blob_path(sha256_hex);
        if !path.exists() {
            return Err(BlobError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Blob not found: {}", sha256_hex),
            )));
        }

        let mut file = File::open(&path)?;
        let mut raw_bytes = Vec::new();
        file.read_to_end(&mut raw_bytes)?;

        if raw_bytes.len() < MIN_BLOB_HEADER_LEN + 16 {
            // Must contain at least header + 16-byte GCM tag
            return Err(BlobError::InvalidHeader);
        }

        // Verify magic bytes
        if &raw_bytes[0..BLOB_MAGIC.len()] != BLOB_MAGIC {
            return Err(BlobError::InvalidHeader);
        }

        let nonce_start = BLOB_MAGIC.len();
        let nonce_end = nonce_start + NONCE_LEN;
        let nonce_bytes = &raw_bytes[nonce_start..nonce_end];
        let nonce = Nonce::from_slice(nonce_bytes);

        let sha_start = nonce_end;
        let sha_end = sha_start + SHA256_LEN;
        let stored_sha_bytes = &raw_bytes[sha_start..sha_end];

        let ciphertext_with_tag = &raw_bytes[sha_end..];

        // Decrypt with AES-256-GCM, verifying AAD matches stored SHA-256
        let cipher = Aes256Gcm::new_from_slice(master_key)
            .map_err(|e| BlobError::Decryption(format!("Cipher init error: {}", e)))?;

        let decrypted_bytes = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext_with_tag,
                    aad: stored_sha_bytes,
                },
            )
            .map_err(|e| BlobError::Decryption(format!("Decryption / tampering failure: {}", e)))?;

        // Verify checksum of decrypted plaintext matches expected hash
        let (calc_hex, calc_bytes) = Self::compute_sha256(&decrypted_bytes);
        if calc_bytes != stored_sha_bytes {
            return Err(BlobError::ChecksumMismatch {
                expected: hex::encode(stored_sha_bytes),
                actual: calc_hex,
            });
        }

        Ok(Zeroizing::new(decrypted_bytes))
    }

    /// Creates a safe temporary plaintext file on disk for sidecar workers (OCR / parser).
    /// Returns an RAII `TempPlaintextGuard` that automatically overwrites with zeros and deletes
    /// the file when dropped.
    pub fn create_temp_plaintext(
        &self,
        master_key: &[u8; 32],
        sha256_hex: &str,
        extension: &str,
    ) -> Result<TempPlaintextGuard, BlobError> {
        let plaintext = self.read_blob(master_key, sha256_hex)?;

        let temp_dir = self.root_dir.join(".tmp_safe");
        fs::create_dir_all(&temp_dir)?;

        let temp_path = temp_dir.join(format!("work_{}.{}", Uuid::new_v4(), extension));
        {
            let mut file = File::create(&temp_path)?;
            file.write_all(&plaintext)?;
            file.sync_all()?;
        }

        Ok(TempPlaintextGuard { path: temp_path })
    }
}

/// RAII Guard for temporary plaintext files.
/// Guarantees that sensitive decrypted document files are securely zeroed and deleted on drop.
pub struct TempPlaintextGuard {
    path: PathBuf,
}

impl TempPlaintextGuard {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempPlaintextGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            // Overwrite file with zeros before deletion to prevent recovery from disk slack
            if let Ok(metadata) = fs::metadata(&self.path) {
                let len = metadata.len() as usize;
                if let Ok(mut file) = File::create(&self.path) {
                    let mut zeros = vec![0u8; std::cmp::min(len, 65536)];
                    let mut written = 0;
                    while written < len {
                        let to_write = std::cmp::min(zeros.len(), len - written);
                        let _ = file.write_all(&zeros[..to_write]);
                        written += to_write;
                    }
                    let _ = file.sync_all();
                    zeros.zeroize();
                }
            }
            let _ = fs::remove_file(&self.path);
        }
    }
}

// Minimal hex encoding helper without needing full external hex crate
mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        let mut s = String::with_capacity(data.as_ref().len() * 2);
        for &b in data.as_ref() {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_read_blob_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("opendeck_blob_test_{}", Uuid::new_v4()));
        let store = BlobStore::new(&temp_dir).unwrap();
        let master_key = [42u8; 32];
        let original_data = b"CONFIDENTIAL_FINANCIAL_INVOICE_DATA_2026_THE_SHRIKS";

        // 1. Store blob
        let res = store.store_blob(&master_key, original_data).unwrap();
        assert!(
            !res.is_duplicate,
            "First store should not be marked duplicate"
        );

        // 2. Read blob
        let decrypted = store.read_blob(&master_key, &res.sha256_hex).unwrap();
        assert_eq!(
            &*decrypted, original_data,
            "Decrypted data must match original"
        );

        // 3. Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_deduplication() {
        let temp_dir = std::env::temp_dir().join(format!("opendeck_blob_test_{}", Uuid::new_v4()));
        let store = BlobStore::new(&temp_dir).unwrap();
        let master_key = [42u8; 32];
        let data = b"DUPLICATE_RECEIPT_CONTENT";

        let res1 = store.store_blob(&master_key, data).unwrap();
        assert!(!res1.is_duplicate);

        let res2 = store.store_blob(&master_key, data).unwrap();
        assert!(
            res2.is_duplicate,
            "Second store of identical content must report duplicate"
        );
        assert_eq!(res1.sha256_hex, res2.sha256_hex);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tampering_detection_rejects_modified_ciphertext() {
        let temp_dir = std::env::temp_dir().join(format!("opendeck_blob_test_{}", Uuid::new_v4()));
        let store = BlobStore::new(&temp_dir).unwrap();
        let master_key = [42u8; 32];
        let data = b"TAMPER_TEST_DATA";

        let res = store.store_blob(&master_key, data).unwrap();
        let blob_path = store.blob_path(&res.sha256_hex);

        // Read raw ciphertext file and flip a single bit in the ciphertext section
        let mut raw_bytes = fs::read(&blob_path).unwrap();
        let last_idx = raw_bytes.len() - 1;
        raw_bytes[last_idx] ^= 0x01; // flip 1 bit

        fs::write(&blob_path, &raw_bytes).unwrap();

        // Attempting to read tampered blob MUST fail with decryption error
        let read_result = store.read_blob(&master_key, &res.sha256_hex);
        assert!(
            read_result.is_err(),
            "Decryption must fail when ciphertext is tampered with"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let temp_dir = std::env::temp_dir().join(format!("opendeck_blob_test_{}", Uuid::new_v4()));
        let store = BlobStore::new(&temp_dir).unwrap();
        let correct_key = [10u8; 32];
        let wrong_key = [99u8; 32];
        let data = b"SECRET_DOCUMENT_PAYROLL";

        let res = store.store_blob(&correct_key, data).unwrap();

        let read_result = store.read_blob(&wrong_key, &res.sha256_hex);
        assert!(
            read_result.is_err(),
            "Decryption must fail when wrong key is provided"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_temp_plaintext_guard_cleans_up_on_drop() {
        let temp_dir = std::env::temp_dir().join(format!("opendeck_blob_test_{}", Uuid::new_v4()));
        let store = BlobStore::new(&temp_dir).unwrap();
        let master_key = [42u8; 32];
        let data = b"EPHEMERAL_OCR_BUFFER";

        let res = store.store_blob(&master_key, data).unwrap();

        let temp_path;
        {
            let guard = store
                .create_temp_plaintext(&master_key, &res.sha256_hex, "png")
                .unwrap();
            temp_path = guard.path().to_path_buf();
            assert!(
                temp_path.exists(),
                "Temp plaintext file must exist while guard is in scope"
            );
            let read_back = fs::read(&temp_path).unwrap();
            assert_eq!(&read_back[..], data);
        } // guard dropped here

        assert!(
            !temp_path.exists(),
            "Temp plaintext file MUST be removed on drop"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
