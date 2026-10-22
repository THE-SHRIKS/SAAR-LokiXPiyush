use saar_lib::db::Database;
use saar_lib::security::{BlobStore, KeyDeriver};
use rusqlite::params;
use std::fs;
use uuid::Uuid;

#[test]
fn test_p1_02_workspace_key_lifecycle_and_disk_inspection() {
    let test_dir = std::env::temp_dir().join(format!("opendeck_p1_02_{}", Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    let db_path = test_dir.join("workspace.db");

    let passphrase = "MasterPassphrase#2026!TheShriks";
    let workspace_id = "ws-real-disk-01";

    // Step 1: Initialize database on real disk and create workspace
    {
        let db = Database::open(&db_path).expect("Failed to create on-disk database");
        let salt = KeyDeriver::generate_salt();
        let master_key = KeyDeriver::derive_master_key(passphrase, &salt).expect("KDF failed");
        let verification_token = KeyDeriver::compute_verification_token(&master_key);

        db.create_workspace(
            workspace_id,
            "THE SHRIKS Operations",
            "USD",
            &salt,
            &verification_token,
        )
        .expect("Failed to insert workspace");

        // Insert a document record to verify persistence
        let conn = db.conn_ref();
        conn.execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'));",
            params!["doc-disk-01", workspace_id, "q3_tax_report.pdf", 4096, "hash_q3_report"],
        )
        .expect("Failed to insert document");
    } // Database closed and flushed to disk here

    // Step 2: Disk inspection — verify no sensitive secrets leaked to SQLite file
    assert!(db_path.exists(), "Database file must exist on disk");
    let disk_bytes = fs::read(&db_path).expect("Failed to read raw DB file");

    // Passphrase string MUST NOT appear anywhere in the database file
    let passphrase_bytes = passphrase.as_bytes();
    let contains_passphrase = disk_bytes
        .windows(passphrase_bytes.len())
        .any(|window| window == passphrase_bytes);
    assert!(
        !contains_passphrase,
        "SECURITY VIOLATION: Raw master passphrase was found in plaintext on disk!"
    );

    // Step 3: Reopen database and verify wrong passphrase rejection
    {
        let db = Database::open(&db_path).expect("Failed to reopen on-disk database");
        let (salt, expected_token) = db
            .get_workspace_auth_params(workspace_id)
            .expect("Failed to read auth params");

        // Attempt unlock with wrong passphrase
        let wrong_unlock_result =
            KeyDeriver::verify_passphrase("IncorrectPassphraseAttempt!99", &salt, &expected_token);
        assert!(
            wrong_unlock_result.is_err(),
            "Wrong passphrase MUST be rejected"
        );

        // Attempt unlock with correct passphrase
        let correct_key = KeyDeriver::verify_passphrase(passphrase, &salt, &expected_token)
            .expect("Correct passphrase must succeed");

        assert_eq!(correct_key.len(), 32);

        // Verify data persistence
        let file_name: String = db
            .conn_ref()
            .query_row(
                "SELECT file_name FROM documents WHERE id = 'doc-disk-01';",
                [],
                |row| row.get(0),
            )
            .expect("Document record must be recoverable after unlock");
        assert_eq!(file_name, "q3_tax_report.pdf");
    }

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_p1_04_blob_store_ciphertext_tampering_and_dedupe() {
    let test_dir = std::env::temp_dir().join(format!("opendeck_p1_04_{}", Uuid::new_v4()));
    let blob_dir = test_dir.join("encrypted_blobs");
    let store = BlobStore::new(&blob_dir).expect("Failed to initialize BlobStore");

    let master_key = [0x5au8; 32];
    let original_payload = b"%PDF-1.7 Confidential Bank Statement for THE SHRIKS \x00\x01\x02\x03";

    // 1. Store blob on disk
    let store_res = store
        .store_blob(&master_key, original_payload)
        .expect("Store blob failed");
    assert!(!store_res.is_duplicate);

    // 2. Disk inspection of ciphertext file
    let blob_file_path = store.blob_path(&store_res.sha256_hex);
    assert!(blob_file_path.exists());
    let raw_ciphertext_file = fs::read(&blob_file_path).expect("Failed to read blob file");

    // Must start with magic bytes "ODBLOB01"
    assert_eq!(&raw_ciphertext_file[0..8], b"ODBLOB01");

    // Plaintext string MUST NOT appear anywhere in the encrypted file
    let sample_plaintext_slice = b"Confidential Bank Statement";
    let leaks_plaintext = raw_ciphertext_file
        .windows(sample_plaintext_slice.len())
        .any(|w| w == sample_plaintext_slice);
    assert!(
        !leaks_plaintext,
        "SECURITY VIOLATION: Unencrypted plaintext found in blob store file on disk!"
    );

    // 3. Deduplication check
    let dup_res = store
        .store_blob(&master_key, original_payload)
        .expect("Second store failed");
    assert!(
        dup_res.is_duplicate,
        "Re-storing identical payload must be recognized as duplicate"
    );
    assert_eq!(dup_res.sha256_hex, store_res.sha256_hex);

    // 4. Bit-flip tampering check
    let mut tampered_bytes = raw_ciphertext_file.clone();
    let flip_index = tampered_bytes.len() - 5;
    tampered_bytes[flip_index] ^= 0x40; // Flip one bit in ciphertext
    fs::write(&blob_file_path, &tampered_bytes).expect("Failed to write tampered blob");

    let tampered_read = store.read_blob(&master_key, &store_res.sha256_hex);
    assert!(
        tampered_read.is_err(),
        "Tampered ciphertext bit-flip MUST fail authenticated AES-256-GCM decryption"
    );

    // Restore original and verify recovery
    fs::write(&blob_file_path, &raw_ciphertext_file).unwrap();
    let recovered = store
        .read_blob(&master_key, &store_res.sha256_hex)
        .expect("Clean file should decrypt");
    assert_eq!(&recovered[..], original_payload);

    // 5. Temporary plaintext file zeroize & cleanup on drop
    let temp_path;
    {
        let guard = store
            .create_temp_plaintext(&master_key, &store_res.sha256_hex, "pdf")
            .expect("Failed to create safe temp file");
        temp_path = guard.path().to_path_buf();
        assert!(temp_path.exists());
        let temp_contents = fs::read(&temp_path).unwrap();
        assert_eq!(&temp_contents[..], original_payload);
    } // guard dropped here

    assert!(
        !temp_path.exists(),
        "Temporary plaintext file must be securely deleted from disk on drop"
    );

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_p1_05_offline_configuration_and_csp_audit() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tauri_conf_path = manifest_dir.join("tauri.conf.json");
    let capabilities_path = manifest_dir.join("capabilities/default.json");

    assert!(tauri_conf_path.exists(), "tauri.conf.json must exist");
    assert!(
        capabilities_path.exists(),
        "capabilities/default.json must exist"
    );

    let tauri_conf_str =
        fs::read_to_string(&tauri_conf_path).expect("Failed to read tauri.conf.json");
    let tauri_conf: serde_json::Value =
        serde_json::from_str(&tauri_conf_str).expect("Invalid tauri.conf.json JSON");

    // 1. Audit Content Security Policy
    let csp = tauri_conf["app"]["security"]["csp"]
        .as_str()
        .expect("CSP must be set in tauri.conf.json");
    assert!(
        csp.contains("connect-src 'none'"),
        "CSP MUST enforce connect-src 'none' to block all network egress"
    );

    // 2. Audit offline frontend distribution (no remote dev server)
    assert!(
        tauri_conf["build"]["devUrl"].is_null(),
        "devUrl must be omitted or null in production offline build"
    );
    let frontend_dist = tauri_conf["build"]["frontendDist"]
        .as_str()
        .expect("frontendDist must be configured");
    assert_eq!(frontend_dist, "../dist");

    // 3. Audit capabilities permissions
    let cap_str =
        fs::read_to_string(&capabilities_path).expect("Failed to read default.json capability");
    let cap: serde_json::Value = serde_json::from_str(&cap_str).expect("Invalid capability JSON");

    let perms = cap["permissions"]
        .as_array()
        .expect("permissions array required");
    for perm in perms {
        let p_str = perm.as_str().unwrap();
        assert!(
            !p_str.starts_with("http:"),
            "HTTP permissions are strictly prohibited in SAAR: found {}",
            p_str
        );
        assert!(
            !p_str.starts_with("network:"),
            "Network permissions are strictly prohibited in SAAR: found {}",
            p_str
        );
        assert!(
            !p_str.starts_with("updater:"),
            "Updater permissions are strictly prohibited in SAAR: found {}",
            p_str
        );
        assert!(
            !p_str.starts_with("shell:"),
            "Arbitrary shell permissions are strictly prohibited in SAAR: found {}",
            p_str
        );
    }
}
