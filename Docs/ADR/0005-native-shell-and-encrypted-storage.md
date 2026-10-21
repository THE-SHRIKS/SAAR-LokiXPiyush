# ADR 0005: Native Shell Architecture and Encrypted Storage Engine

## Status
Accepted and Verified (Gate G1).

## Context
OpenDeck requires a strictly local-first, offline financial intelligence workspace that protects user data against at-rest exfiltration, offline tampering, process hijacking, and memory residual extraction. In accordance with `RULES.md` and `SPECS.md`:
1. Persian-rug cryptographic keys must never be hard-coded; workspace keys must be derived from user passphrases using a memory-hard KDF.
2. Persisted records (structured transactions and ingested document blobs) must be encrypted at rest with authenticated encryption (AES-256-GCM), preventing undetectable bit-flip or metadata tampering.
3. Plaintext files and decrypted buffers must have guaranteed lifecycle scrubbing (RAII zeroization and secure unlink).
4. Runtime network egress must be blocked unconditionally via strict Content Security Policy (`connect-src 'none'`) and stripped Tauri capabilities.
5. All financial amounts must be tracked strictly in integer minor units (cents).

## Architecture & Implementation Decisions

### 1. Key Derivation Function (KDF) Lifecycle
- **Algorithm:** Argon2id (Version 0x13).
- **Parameters:** Memory size: 64 MiB (65,536 KiB), Iterations: 3, Parallelism: 1, Derived Key: 32 bytes (256-bit).
- **Salt:** 16-byte cryptographically secure pseudorandom salt generated per workspace via `OsRng`.
- **Verification Token:** `SHA-256(master_key || "opendeck_master_key_verification_v1")`. This allows constant-time verification of passphrase validity upon unlock without ever persisting the master key or the passphrase on disk.
- **Memory Scrubbing:** The master key is wrapped in `zeroize::Zeroizing<[u8; 32]>`, ensuring key bytes are overwritten with zeros immediately upon workspace lock or scope drop.

### 2. Encrypted Document Blob Storage (`BlobStore`)
- **Cipher:** AES-256-GCM authenticated encryption.
- **Nonce:** Unique 96-bit (12-byte) random nonce generated per document using `OsRng`.
- **Additional Authenticated Data (AAD):** SHA-256 content checksum bound into the GCM authentication tag, ensuring ciphertext cannot be decoupled from its content identity.
- **On-Disk File Format:**
  ```text
  +-------------------------------------------------------------------+
  | Magic Header: b"ODBLOB01" (8 bytes)                                |
  +-------------------------------------------------------------------+
  | Nonce: 12 bytes (OsRng)                                           |
  +-------------------------------------------------------------------+
  | SHA-256 Checksum: 32 bytes (plaintext digest)                    |
  +-------------------------------------------------------------------+
  | Ciphertext + GCM Authentication Tag (variable length + 16 bytes) |
  +-------------------------------------------------------------------+
  ```
- **Deduplication:** Content-addressed indexing via plaintext SHA-256 hex digest (`<sha256>.blob.enc`). Re-importing identical files returns `is_duplicate: true` with zero duplicate disk allocation.
- **Atomic Persistence:** Writes proceed through `<sha256>.tmp.<uuid>`, followed by physical disk flush (`file.sync_all()`), and atomic filesystem rename.
- **Safe Decrypted Temp Files:** Sub-process workers requiring local file access use `TempPlaintextGuard`, which writes to `.tmp_safe/work_<uuid>.<ext>` and securely overwrites the buffer with zeros before unlinking on drop.

### 3. Database Schema & WAL Durability
- **Engine:** SQLite with WAL journal mode, `foreign_keys = ON`, `synchronous = NORMAL`, and `busy_timeout = 5000`.
- **Tables (Migration v1):**
  - `workspaces`: Encrypted workspace credentials, Argon2id salt, and verification token.
  - `documents`: Document metadata, SHA-256 content address, parser status.
  - `document_fields`: Extracted text, bounding box coordinates, extraction confidence.
  - `invoices`: Extracted supplier details, invoice dates, integer minor-unit totals and taxes.
  - `line_items`: Line-item breakdown with integer minor-unit amounts.
  - `transactions`: Normalized ledger postings with exact minor-unit currency amounts.
  - `audit_log`: Immutable append-only audit trail logging all entity creations and user mutations.
  - `schema_migrations`: Version tracking table ensuring idempotent migration execution.

### 4. Restrictive Tauri 2 Security Boundaries
- **Content Security Policy (CSP):**
  ```text
  default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset:; font-src 'self'; connect-src 'none'
  ```
- **Capabilities (`src-tauri/capabilities/default.json`):** Restricts webview permissions to core window/event/app handling. Completely excludes `http:*`, `network:*`, `updater:*`, and `shell:*`.
- **Sidecar Process Supervision (`WorkerSupervisor`):**
  - Validates all invocation arguments against remote URL protocols (`http://`, `https://`, `ftp://`, `ws://`).
  - Implements `ChildGuard` RAII process lifecycle: kills spawned workers immediately if dropped or if execution exceeds configured timeouts.
  - Ephemeral loopback port allocator strictly binds to `127.0.0.1` and forbids `0.0.0.0`.

## Verification & Test Evidence

| Test Case | Execution Command | Result | Evidence |
|---|---|---|---|
| Front-end domain types & currency math | `pnpm test` | **4/4 passed** | Minor-unit formatting, negative amounts, JPY non-fractional |
| Front-end ESLint compliance | `pnpm run lint` | **0 errors, 0 warnings** | Full TypeScript / React 18 compliance |
| Front-end production build | `pnpm run build` | **Clean build (dist/)** | Zero remote asset references |
| Tauri GTK 3 desktop GUI launch | `target/debug/opendeck` | **Launched on Debian DISPLAY=:0.0** | Screenshot: [`tests/opendeck_debian_p1_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p1_verification.png) |
| SQLite Schema & Rollback | `cargo test db::` | **4/4 passed** | Idempotent migrations, FK enforcement, transactional rollback, audit logging |
| Argon2id Key Derivation | `cargo test security::kdf` | **3/3 passed** | Deterministic derivation, wrong passphrase rejection, empty passphrase rejection |
| AES-256-GCM Blob Store | `cargo test security::blob_store` | **5/5 passed** | Encrypt/decrypt roundtrip, deduplication, 1-bit tampering detection, wrong key rejection, temp cleanup |
| Worker Supervisor Lifecycle | `cargo test supervisor::` | **5/5 passed** | Child kill on drop, execution timeout, URL argument denial, 127.0.0.1 port allocation |
| Disk Inspection & Reopen Integration | `cargo test --test p1_native_storage_verification` | **3/3 passed** | Disk inspection: 0 passphrase bytes on disk; correct key recovery; CSP audit; capability audit |

**Total Phase 1 Rust Tests:** 20/20 passed in 17.4s (unit) + 16.5s (integration).
