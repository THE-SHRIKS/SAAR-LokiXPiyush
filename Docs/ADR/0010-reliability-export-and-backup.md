# ADR 0010: Reliability, Source-Traceable Exports, and Encrypted Backups

## Status
Accepted (Gate G6 Passed)

## Context
As an air-gapped financial intelligence workspace, OpenDeck must satisfy strict auditability and disaster recovery guarantees without ever making external cloud calls or relying on binary floating-point representations:
1. **Source-Traceable Exports:** Users must be able to export deterministic reconciliation and summary reports to standard PDF and CSV files. Every exported number must match internal signed 64-bit integer minor units exactly, and reports must include cryptographic SHA-256 snapshot hashes of the underlying records.
2. **Encrypted Backup & Disaster Recovery:** persisted database files and encrypted blob stores must be packagable into a single portable `.opendeck` archive. Backups must be strongly encrypted with AES-256-GCM using keys derived via Argon2id. Wrong passphrase attempts must be rejected mathematically with zero state corruption.
3. **Egress Denial & Air-Gap Enforcement:** The application must operate with zero external network calls (`connect-src 'none'`), no background telemetry, and zero unauthenticated process sockets.
4. **Crash Recovery & Log Scrubbing:** System restarts must recover interrupted in-flight jobs to `FAILED` states rather than leaving ghost jobs, and logs must be automatically scrubbed of all credentials, credit card numbers, PAN, and Aadhaar numbers.

## Decision
1. **Zero-Dependency PDF 1.4 & RFC 4180 CSV Engine:**
   - Implemented a custom pure-Rust PDF 1.4 writer (`src-tauri/src/export/pdf_writer.rs`) using standard Type 1 fonts (Helvetica/Helvetica-Bold), vector layouts, and deterministic minor-unit formatting. Verified with standard Linux tools `pdfinfo` and `pdftotext` with zero syntax warnings.
   - Implemented an RFC 4180 compliant CSV generator (`src-tauri/src/export/csv_exporter.rs`) that exports reconciled rows, amounts in cents and decimal representations, and SHA-256 provenance hashes.
   - All export events are recorded in `report_exports` (database migration v4) and in the append-only `audit_log`.
2. **Authenticated `.opendeck` Container Format:**
   - File layout: `OPENDECK` (8 bytes) + version `1` (2 bytes) + salt (16 bytes) + nonce (12 bytes) + AES-256-GCM ciphertext with Poly1305 authentication tag covering `OPENDECK_V1_BACKUP_CONTAINER` AAD.
   - Restores verify database SHA-256, execute SQLite `PRAGMA integrity_check;`, and enforce path traversal sanitization on all blob files.
3. **Offline Egress Denial Audit:**
   - Verified via `scripts/network_egress_audit.sh` and Linux network namespace isolation (`unshare -rn`). Verified that all sidecars run via standard I/O pipes with zero open WAN sockets.
4. **Automated Log Scrubber & Crash Recovery:**
   - Implemented `LogScrubber` with compiled regular expressions scrubbing passphrases, tokens, credit cards, Indian PAN/Aadhaar IDs, and home directory paths.
   - Implemented `recover_interrupted_operations` in `Database::open` marking `PROCESSING`/`EXTRACTING` documents as `FAILED`.

## Consequences
- Reports can be independently recalculated and audited against source documents.
- Encrypted backup archives can be safely transported across workstations or profiles without leaking master keys or plaintext records.
- Complete SBOM and attribution manifest documented in `RELEASE_MANIFEST.md` and `Docs/ATTRIBUTION.md`.
