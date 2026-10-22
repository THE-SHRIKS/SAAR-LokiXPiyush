# OpenDeck Windows Release Acceptance Report

- **Target OS:** Windows 10 / Windows 11 x64 (MSVC ABI)
- **Application Version:** v0.1.0 (Build Native Release)
- **Air-Gap Boundary:** 100% Offline (Zero external network egress)
- **Verification Date:** 2026-10-22 15:00:00 UTC
- **Sign-off Status:** **ALL GATES PASSED (G0 - G7)**

---

## 1. Acceptance Matrix Results (T-01 .. T-20)

| ID | Test Name | Required Observation | Status |
|---|---|---|---|
| **T-01** | Windows Cold Launch | Native desktop window initialized; local WebView2 rendered React UI without remote web server requirement. | **PASSED** |
| **T-02** | Zero-Network Egress Denial | CSP `connect-src 'none'` strictly active; process sockets audit confirms zero WAN listeners or background outbound requests. | **PASSED** |
| **T-03** | Argon2id Wrong Key Rejection | Incorrect passphrase yields instant AuthError; zero memory leakage or plaintext database exposure. | **PASSED** |
| **T-04** | Document Ingestion & AES-256 Storage | Encrypted blob storage created; content-addressed SHA-256 deduplication rejects repeated uploads idempotently. | **PASSED** |
| **T-05** | OCR Bounding Box Provenance | Native RapidOCR sidecar returned normalized fields with exact page bounding box coordinates. | **PASSED** |
| **T-06** | Deterministic Minor-Unit Arithmetic | Signed 64-bit integer minor unit arithmetic ($1250.00 = 125000 cents); zero binary floating-point roundoff variance. | **PASSED** |
| **T-07** | Reconciliation Matching | Deterministic multi-pass matcher proposed exact and proximity links with structured confidence rationales. | **PASSED** |
| **T-08** | Duplicate & Recurring SaaS Anomaly | Detected duplicate invoice within 3-day window; suppressed false positive on monthly recurring SaaS subscription. | **PASSED** |
| **T-09** | Local AI Grounded Q&A | Qwen2.5-3B generated answers restricted strictly to provided Fact Packets with provenance citations `[inv:ID]`. | **PASSED** |
| **T-10** | Prompt Injection Defense | Adversarial instructions inside document text intercepted; unauthorized write operations blocked. | **PASSED** |
| **T-11** | Model Absence / CPU Fallback | Graceful fallback to CPU thread execution with deterministic parameters when discrete GPU is absent. | **PASSED** |
| **T-12** | User Approval & Immutable Audit Trail | Manual transaction description edit recorded with before/after state captures in SQLite `audit_log` table. | **PASSED** |
| **T-13** | PDF & CSV Source-Traceable Reports | Generated pure-Rust PDF 1.4 report and RFC 4180 CSV matching ledger totals with cryptographic SHA-256 snapshot hash. | **PASSED** |
| **T-14** | AES-256-GCM Backup & Profile Restore | Created authenticated `.opendeck` backup archive; wrong key failed auth; clean restore verified with `PRAGMA integrity_check`. | **PASSED** |
| **T-15** | Malformed / Untrusted Documents | Corrupted PDF headers, zero-length files, path traversal characters handled gracefully with structured errors. | **PASSED** |
| **T-16** | UI Ergonomics & Accessibility | Dual-resolution verified (1280px / 960px); zero dead buttons, zero generic admin templates, full keyboard accessibility. | **PASSED** |
| **T-17** | Model Manifest & Checksums | Local weights verified against SHA-256 digest (`8eff4e0eb51...`), zero online downloads. | **PASSED** |
| **T-18** | Native Windows Packaging | Standalone offline NSIS installer with bundled WebView2 offline bootstrapper and pre-packaged sidecars. | **PASSED** |
| **T-19** | Full Windows Workflow | Ingestion → OCR → Categorize → Reconcile → Review → Chat → Export → Backup verified end-to-end. | **PASSED** |
| **T-20** | Restart Under Load | In-flight processing operations cleanly recovered to `FAILED` status upon restart, preventing state lockups. | **PASSED** |

---

## 2. Release Certification
- All advertised features (OCR, Matching, Anomaly Detection, GBNF Classifier, Grounded Chat, PDF/CSV Reports, Encrypted Backup) are fully functional.
- Zero mock modules, zero simulated data, zero cloud endpoints, and zero telemetry active in the binary release.
- Cryptographic SHA-256 integrity verified across model weights, binaries, and database schemas.

**Signed by:** THE SHRIKS Engineering Quality Team
