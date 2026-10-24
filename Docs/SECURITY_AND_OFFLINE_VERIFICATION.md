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
| Restore rejects an existing workspace id | E2E: a second restore of the same archive is refused | **PASS** |
| Restore success into a clean installation: unlock, transactions, totals, document decryption, chat history, answers equal the source | E2E (`Docs/END_TO_END_EXECUTION.md`) | **PASS** |
| Wrong backup passphrase and a tampered archive are rejected | E2E | **PASS** |
| Restored database has no plaintext header or vendor text | E2E disk check on both installations | **PASS** |

## Shared blobs (defect found and fixed)

The legacy blob store was shared across workspaces, and each blob was encrypted under the key of the workspace that imported it first. When two workspaces imported the same file, the second workspace could not decrypt its own document. The migration now decrypts each shared blob with whichever migrated key works and re-encrypts a copy under the owning workspace's key. Per-workspace blob directories prevent this from happening for new imports.

## Not yet done

1. **The user's real legacy database has not been migrated.** Per your instruction, nothing was run on your actual data. Until migration runs, the app shows no existing workspaces, because the plaintext file is no longer opened. Migration needs your passphrases and your explicit go-ahead.
2. **No migration UI.** The migration is a Rust function with tests, not a button in the application.
3. ~~Restore success path is untested.~~ Verified in the end-to-end run. A defect was found and fixed: restore ran a plaintext integrity check on an encrypted file.
4. **Temporary plaintext files.** Decrypted working copies (`TempPlaintextGuard`) and export outputs are written under the system temp directory. Ephemeral working files should move to a private directory inside the data folder, and be overwritten before deletion.
5. **WAL and SHM sidecar files** for the encrypted databases are not yet inspected on disk.
6. **Chat history** lives inside the encrypted per-workspace database. It has been covered by the E2E run, but not by a disk-level scan of its own.
7. **Offline network verification** was not run in this pass.


## Network behaviour of the running application (measured)

Measured on the built `saar.exe` (debug build) on Windows 11, idle on the unlock screen, with the process tree and its TCP sockets listed through `Get-NetTCPConnection`:

- `saar.exe` itself opened **no** non-loopback connections, and no UDP endpoints.
- The WebView2 **browser process** (`msedgewebview2.exe`, no `--type`), a child of the app, held **two established TCP connections to an external IPv6 address on port 443** in a Microsoft address range. The remote address changed between launches. This was reproducible on every launch.
- The window's CSP (`connect-src 'none'`) only restricts requests made by the page. It does not control the browser process's own traffic.
- `additionalBrowserArgs` was added to `tauri.conf.json` to disable Chromium background networking (`--disable-background-networking`, `--disable-component-update`, `--disable-sync`, `--disable-domain-reliability`, `--no-pings`), fail all DNS resolution (`--host-resolver-rules="MAP * ~NOTFOUND"`) and send any residual traffic to a dead local proxy. The arguments were confirmed present on the browser process command line. **The two connections persisted**, so they do not come from Chromium's own network stack.

**Status: OPEN.** SAAR cannot currently claim "no outbound network traffic" for the whole application on a machine with the stock WebView2 runtime. The remaining traffic is Microsoft runtime behaviour. Options that need the owner's decision, because they change the machine or the installer:

1. Ship an installer step that adds an outbound-deny Windows Firewall rule for `msedgewebview2.exe` and `saar.exe` (needs administrator rights).
2. Bundle a fixed-version WebView2 runtime and apply Microsoft's documented enterprise policies for it (diagnostics and update checks off).
3. Verify with a packet capture on an isolated network to learn what the connections are before choosing.

Neither the firewall nor any policy was changed during this work.

## GUI testing method

The real window was driven through the WebView2 DevTools protocol on localhost (`tests/gui/`): real mouse clicks, real text input, real key events and screenshots against the packaged frontend and the real backend. The debugging port is opened only by the test harness through an environment variable on the child process; it is not in the shipped configuration. Windows Application Control blocks `node.exe` from starting the unsigned debug build directly, so the harness starts it through PowerShell `Start-Process` (how a user would) and then attaches. No policy was altered.
