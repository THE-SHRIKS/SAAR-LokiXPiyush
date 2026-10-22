# OpenDeck v0.1.0 — Production Release Notes

**Release Version:** v0.1.0 (General Availability)  
**Target Environments:** Debian GNU/Linux 13 (trixie) x86_64 • Microsoft Windows 10 / 11 64-bit  
**Security Profile:** 100% Offline • Strict Air-Gap • Zero External Network Calls • Zero Telemetry  
**Organization:** THE SHRIKS  

---

## 1. Executive Summary

OpenDeck is an offline, air-gapped financial intelligence desktop workspace engineered from the ground up for strict confidentiality, mathematical auditability, and deterministic accounting. It combines an embedded SQLite ledger, an AES-256-GCM authenticated blob store, local CPU-optimized machine learning sidecars (RapidOCR + Qwen2.5-3B via llama.cpp), and an editorial desktop interface with dual-pane document evidence inspection.

---

## 2. Core Capabilities

### A. Mathematical Financial Invariants
- **Zero Floating-Point Arithmetic:** All money values (inflows, outflows, taxes, balances, adjustments) are strictly represented and calculated using signed 64-bit integer minor units (cents). Floating-point roundoff errors are structurally prevented.
- **Deterministic Multi-Pass Reconciliation:** Automated 4-tier matching engine connects invoices to bank statement transactions with confidence scoring (Exact Amount & Date = 1.0, Token Similarity = 0.85, Candidate Splits, Proximity Window = 0.50).
- **Anomaly & Duplicate Detection:** Proactively flags duplicate billings within a 3-day window while intelligently suppressing recurring monthly SaaS subscription false positives.
- **Immutable Audit Trail:** All manual corrections, link approvals, and document extractions append tamper-evident before/after records into SQLite `audit_log`.

### B. High-Fidelity Document Ingestion & Evidence Inspector
- **Two-Tier Document Pipeline:** High-speed digital PDF extraction (<100ms) with seamless fallback to CPU-accelerated RapidOCR (ONNXRuntime AVX2) for scanned PDFs and thermal receipts.
- **Coordinate-Accurate Bounding Boxes:** Normalized fields preserve original document bounding boxes. Selecting a field in the ledger highlights the exact source rectangle on the rendered preview.
- **Decrypted In-Memory Preview:** Documents remain encrypted at rest with AES-256-GCM and are decrypted on-demand in memory into base64 visual previews without ever writing unencrypted temporary files to disk.

### C. Grounded Local Intelligence & GBNF Classification
- **Air-Gapped Qwen2.5-3B:** Runs locally via bundled `llama-cli` on CPU with AVX2 acceleration and deterministic greedy decoding (temperature 0.0).
- **Provenance Citations:** Answers to natural language queries cite specific source records (e.g. `[inv:ID]`, `[tx:ID]`) compiled into compact, bounded Fact Packets.
- **Anti-Hallucination Guard:** Validates all generated monetary amounts against internal ledger balances before displaying them in chat.
- **Formal Grammar Constrained Classification:** Uses GBNF grammars to strictly force transaction categorization into 7 allowed business categories with confidence rationales.

### D. Source-Traceable Exports & Encrypted Disaster Recovery
- **Deterministic PDF 1.4 Reports:** Pure-Rust vector PDF generator formatting financial summaries, monthly cash flow charts, and reconciliation tables with SHA-256 snapshot hashes. Verified with `pdfinfo` and `pdftotext`.
- **RFC 4180 CSV Exports:** Clean, escaped tabular data matching database records with minor-unit integer precision.
- **Authenticated `.opendeck` Archives:** Backups are packaged into an encrypted container using Argon2id memory-hard key derivation and AES-256-GCM. Wrong passphrases or tampered bytes are rejected mathematically with zero database contamination.

---

## 3. Verified Verification Gates (G0 – G7)

| Gate | Focus | Verification Evidence | Status |
|---|---|---|---|
| **G0** | Environment & Baseline | Debian 13.7 x86_64, Core i3 AVX2, pinned Qwen2.5 weights, 10 synthetic fixtures | **PASSED** |
| **G1** | Native Shell & Storage | Tauri 2 shell, Argon2id KDF, SQLite WAL migrations v1, AES-256-GCM blobs | **PASSED** |
| **G2** | Ingestion & OCR | Tier A digital parser, Tier B RapidOCR sidecar, RFC 4180 CSV mapper | **PASSED** |
| **G3** | Financial Core | Minor-unit math, multi-pass matching, duplicate/SaaS anomaly detection, review inbox | **PASSED** |
| **G4** | Grounded Local AI | Local llama-cli, Fact Packets, GBNF classifier, prompt injection & anti-hallucination guard | **PASSED** |
| **G5** | Editorial Interface | Dual-resolution GUI (1280px / 960px), evidence inspector, inline correction modal | **PASSED** |
| **G6** | Reliability & Backup | Pure-Rust PDF/CSV exports, `.opendeck` backup/restore, offline egress denial (`unshare -rn`) | **PASSED** |
| **G7** | Native Windows Release | Native Windows sidecars, MSVC/CMake pipelines, NSIS installer, acceptance report | **PASSED** |

---

## 4. Hardware & System Requirements

- **Processor:** x86_64 processor with AVX2 instruction support (Intel 4th Gen Core+ / AMD Zen+)
- **Memory:** 8 GB RAM minimum (11 GB recommended for full OCR + LLM execution)
- **Storage:** 4 GB free disk space (includes model weights and local database)
- **Operating Systems:**
  - Debian GNU/Linux 12 / 13 (x86_64)
  - Microsoft Windows 10 / 11 64-bit (x64)
- **Network Requirement:** **Zero.** Operates in permanent Airplane Mode.
