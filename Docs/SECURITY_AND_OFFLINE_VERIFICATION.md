# SAAR — Security and Offline Verification (partial, this pass)

## Storage architecture (implemented)

- **One encrypted SQLCipher database per workspace**, at `<data_dir>/workspaces/<id>.saardb`.
- **Blobs per workspace**, at `<data_dir>/workspaces/<id>.blobs/`, encrypted with the workspace master key (AES-256-GCM, unchanged).
- **Key derivation:** Argon2id (64 MiB, 3 passes, 1 lane) from the passphrase and a per-workspace salt gives the master key. The SQLCipher key is `SHA-256("saar.workspace.database-key.v1" || master)`. The master key is never passed to SQLCipher. The database key and the blob key are therefore different values.
- **Catalog:** `<data_dir>/catalog.json` lists only the workspace id, the Argon2id salt (not secret), the creation time, and the file location. It holds no display names, no records, and no chat content.
- **Startup** opens no database. A workspace is opened only after its passphrase is verified by SQLCipher's first read.
- **Lock** drops the connection, the blob store, and the master key.
- **Readiness:** `databaseEncrypted` is true only when the open database was opened with a key. It was previously true for any SQLCipher build, which was an incorrect readiness indicator.

## Evidence

| Check | Method | Result |
|---|---|---|
| Migrated workspace file is not plaintext | `migration_verification::test_migration_produces_verified_encrypted_workspaces` checks the header and searches for record text | **PASS** |
| Wrong key rejected for an encrypted file | Same test opens with a wrong derived key | **PASS** |
| Correct key opens, integrity check is `ok` | Same test | **PASS** |
| Legacy plaintext file is not modified by migration | Same test compares SHA-256 before and after | **PASS** |
| Wrong passphrase aborts migration and rolls back | `test_wrong_passphrase_rolls_back_without_touching_legacy` | **PASS** |
| Migration refuses to overwrite an existing workspace | `test_refuses_to_overwrite_an_existing_encrypted_workspace` | **PASS** |
| New workspace created by the application is encrypted on disk | E2E run: `encrypted workspace files checked on disk: 1`; no plaintext header; no record text | **PASS** (one workspace) |
| Wrong passphrase rejected at unlock | E2E run | **PASS** |
| Data survives lock and unlock | E2E run: 8 transactions and 12 chat messages after unlock | **PASS** |
| Backup captures the active workspace's blobs | E2E run: `blobCount: 4` (previously 0) | **PASS** |
| Restore rejects an existing workspace id | Code path only; no test yet | **NOT VERIFIED** |
| Restore success path (staging, promotion, unlock of restored copy) | Not run | **NOT VERIFIED** |

## Shared blobs (defect found and fixed)

The legacy blob store was shared across workspaces, and each blob was encrypted under the key of the workspace that imported it first. When two workspaces imported the same file, the second workspace could not decrypt its own document. The migration now decrypts each shared blob with whichever migrated key works and re-encrypts a copy under the owning workspace's key. Per-workspace blob directories prevent this from happening for new imports.

## Not yet done

1. **The user's real legacy database has not been migrated.** Per your instruction, nothing was run on your actual data. Until migration runs, the app shows no existing workspaces, because the plaintext file is no longer opened. Migration needs your passphrases and your explicit go-ahead.
2. **No migration UI.** The migration is a Rust function with tests, not a button in the application.
3. **Restore success path is untested.** The code is written; it must be exercised before it is trusted.
4. **Temporary plaintext files.** Decrypted working copies (`TempPlaintextGuard`) and export outputs are written under the system temp directory. Ephemeral working files should move to a private directory inside the data folder, and be overwritten before deletion.
5. **WAL and SHM sidecar files** for the encrypted databases are not yet inspected on disk.
6. **Chat history** lives inside the encrypted per-workspace database. It has been covered by the E2E run, but not by a disk-level scan of its own.
7. **Offline network verification** was not run in this pass.
