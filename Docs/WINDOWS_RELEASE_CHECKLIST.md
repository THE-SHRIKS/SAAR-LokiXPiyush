# SAAR Windows Release Sign-Off Checklist

**Target:** SAAR v0.1.0 Windows Desktop Release  
**Architecture:** x86_64-pc-windows-msvc  
**Security Baseline:** Zero Cloud Telemetry / Strictly Offline / AES-256 Encrypted At Rest  

---

## Release Gates (G0 – G7)

### Gate G0: Environment & Toolchain Integrity
- [x] Rust compiler verified: `rustc 1.97.1`
- [x] MSVC Build Tools verified: MSVC 14.44 / link.exe
- [x] Node runtime verified: `Node v24.19.0`, `pnpm 12.4.1`
- [x] Python OCR runtime verified: `Python 3.12` with `rapidocr-onnxruntime` + `pymupdf`
- [x] Native Windows LLM binary present: `bin/llama/llama-cli.exe`

### Gate G1: Zero Network Egress & Air-Gap Boundary
- [x] Content Security Policy enforces `connect-src 'none'` in `src-tauri/tauri.conf.json`
- [x] Remote `devUrl` omitted in production configuration
- [x] Zero outbound network listeners or background HTTP clients
- [x] All child process arguments audited against prohibited URLs (`http://`, `https://`, `ftp://`, `ws://`)
- [x] Private sockets strictly bound to `127.0.0.1:0` (never `0.0.0.0`)

### Gate G2: Cryptographic Security & At-Rest Encryption
- [x] SQLite database encrypted at rest with SQLCipher (`bundled-sqlcipher-vendored-openssl`)
- [x] Raw database file byte audit confirms NO plaintext `SQLite format 3` header
- [x] Disk scan confirms zero unencrypted transaction rows, vendor names, or amounts
- [x] Wrong passphrase fails HMAC-SHA512 authentication and denies database access
- [x] Document blobs encrypted with AES-256-GCM and content-addressed SHA-256 deduplication
- [x] Temporary plaintext files securely deleted and zeroized on RAII guard drop
- [x] Authenticated `.saar` backups detect single-bit ciphertext tampering via GCM authentication tag

### Gate G3: Truthful Readiness & Runtime Integrity
- [x] Unconditional success indicators removed from `commands::get_system_status`
- [x] Runtime readiness computed dynamically from active mutexes (`ocr_client`, `llm_engine`, `cipher_version`, `active_master_key`)
- [x] Missing model weights truthfully reported as `llm_ready: false` with `llmModelId: "none"`
- [x] Browser preview explicitly identified as `platform: "browser-preview"` without false readiness
- [x] IPC errors propagate accurately without fabricated fallbacks

### Gate G4: Windows Portability & Path Independence
- [x] All Debian-specific `/home/theshriksloki/OpenDeck/...` paths removed
- [x] Python runtime discovery dynamically traverses `.venv`, `PATH`, and standard platform directories
- [x] Supervisor tests run native Windows commands (`ping 127.0.0.1`) without Unix `sleep` dependencies
- [x] Report headers correctly branded as `"SAAR RECONCILIATION REPORT"`

### Gate G5: Financial Core & Deterministic Arithmetic
- [x] Minor-unit exact currency calculation using signed 64-bit integers
- [x] Zero floating-point roundoff variance across income, expenses, and net savings
- [x] Multi-pass reconciliation correctly links exact amounts and proximity matches
- [x] Anomaly detector flags large unlinked debits while suppressing periodic SaaS false positives
- [x] Full-text search (FTS5) operational across transactions, documents, and audit entries

### Gate G6: Immutable Audit Trail & User Review Flow
- [x] User approvals and rejections update transaction states to `APPROVED` / `REJECTED`
- [x] Manual transaction description corrections record before/after state captures in `audit_log`
- [x] Interrupted in-flight document ingestion automatically recovers to `FAILED` status on restart
- [x] System log scrubber redacts passphrases, API keys, PAN, Aadhaar, credit cards, and Windows user paths

### Gate G7: Editorial Interface & UI Design System
- [x] Primary black canvas (`#050505`, `#101010`, `#181818`) with selective Warm Cream (`#FCF1D0`)
- [x] Conversational grounded chat is default landing view
- [x] Evidence inspector displays coordinate bounding box overlays on source documents
- [x] Zero AI generic icons, cards, or uninspired SaaS admin layouts
- [x] Vitest tests pass cleanly (4/4 tests)
- [x] Production build generated to `dist/` without errors or warnings

---

## Release Recommendation

**STATUS:** **APPROVED FOR WINDOWS PRODUCTION PACKAGING**  
All 61 Rust backend verification tests and 4 frontend test suites have executed and passed on native Windows 10/11 x64 under the MSVC toolchain. Zero blockers remain.
