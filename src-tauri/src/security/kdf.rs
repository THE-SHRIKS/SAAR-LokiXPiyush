use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

/// OpenDeck Argon2id Key Derivation Configuration
/// Memory: 64 MiB (65,536 KiB), Iterations: 3, Parallelism: 1
pub struct KeyDeriver;

impl KeyDeriver {
    /// Generates a cryptographically secure random 16-byte salt from the OS CSPRNG.
    pub fn generate_salt() -> [u8; SALT_LEN] {
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Derives a 32-byte master encryption key from the user passphrase and workspace salt.
    pub fn derive_master_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, String> {
        if passphrase.is_empty() {
            return Err("Passphrase cannot be empty.".to_string());
        }
        if salt.len() < SALT_LEN {
            return Err("Invalid salt length.".to_string());
        }

        // Configure memory-hard Argon2id parameters
        let params = Params::new(65536, 3, 1, Some(KEY_LEN))
            .map_err(|e| format!("Argon2 params error: {}", e))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        Ok(key)
    }

    /// Computes a verification token derived from the master key.
    /// Used to securely confirm the passphrase without storing or leaking the master key.
    pub fn compute_verification_token(master_key: &[u8; KEY_LEN]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(b"opendeck_master_key_verification_v1");
        let result = hasher.finalize();
        let mut token = [0u8; 32];
        token.copy_from_slice(&result);
        token
    }

    /// Constant-time verification of a passphrase against salt and stored verification token.
    pub fn verify_passphrase(
        passphrase: &str,
        salt: &[u8],
        expected_token: &[u8; 32],
    ) -> Result<Zeroizing<[u8; KEY_LEN]>, String> {
        let master_key = Self::derive_master_key(passphrase, salt)?;
        let computed_token = Self::compute_verification_token(&master_key);

        // Constant time comparison
        if subtle_slices_equal(&computed_token, expected_token) {
            Ok(master_key)
        } else {
            Err("Incorrect master passphrase provided.".to_string())
        }
    }
}

/// Constant-time 32-byte equality comparison to prevent timing attacks.
fn subtle_slices_equal(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_derivation_deterministic() {
        let salt = KeyDeriver::generate_salt();
        let key1 = KeyDeriver::derive_master_key("CorrectHorseBatteryStaple", &salt).unwrap();
        let key2 = KeyDeriver::derive_master_key("CorrectHorseBatteryStaple", &salt).unwrap();
        assert_eq!(*key1, *key2, "Same passphrase and salt must produce identical keys");
    }

    #[test]
    fn test_wrong_passphrase_rejection() {
        let salt = KeyDeriver::generate_salt();
        let master_key = KeyDeriver::derive_master_key("CorrectPassphrase123!", &salt).unwrap();
        let expected_token = KeyDeriver::compute_verification_token(&master_key);

        // Verify correct passphrase succeeds
        let unlocked = KeyDeriver::verify_passphrase("CorrectPassphrase123!", &salt, &expected_token);
        assert!(unlocked.is_ok(), "Correct passphrase must succeed");

        // Verify incorrect passphrase fails
        let wrong = KeyDeriver::verify_passphrase("WrongPassphrase456!", &salt, &expected_token);
        assert!(wrong.is_err(), "Wrong passphrase must be rejected");
        assert_eq!(wrong.unwrap_err(), "Incorrect master passphrase provided.");
    }

    #[test]
    fn test_empty_passphrase_rejected() {
        let salt = KeyDeriver::generate_salt();
        let res = KeyDeriver::derive_master_key("", &salt);
        assert!(res.is_err());
    }
}
