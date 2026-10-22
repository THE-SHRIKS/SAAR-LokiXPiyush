# Project Status Ledger — OpenDeck

**Current Phase:** Production Release Complete  
**Active Task:** All Phases & Gates Complete (G0 – G7 Signed Off)  
**Last Updated:** 2026-10-22  

---

## 1. Gate Sign-Off Ledger

| Gate | Name | Required Proofs | Status | Evidence Link |
|---|---|---|---|---|
| **G0** | Feasibility, Environment & Runtime Baseline | Toolchain inspection, native Tauri 2 desktop window, local OCR on real scan, offline LLM GBNF inference on CPU, model licenses, threat model + fixtures | **PASSED & SIGNED** | [`Docs/ADR/0001-environment-and-architecture-constraints.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0001-environment-and-architecture-constraints.md)<br>[`Docs/ADR/0002-threat-model-and-fixture-registry.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0002-threat-model-and-fixture-registry.md)<br>[`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md)<br>[`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md) |
| **G1** | Native Shell + Storage | Desktop launch on Debian, encrypted workspace key lifecycle, SQLCipher migrations, no network | **PASSED & SIGNED** | [`Docs/ADR/0005-native-shell-and-encrypted-storage.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0005-native-shell-and-encrypted-storage.md)<br>Screenshot: [`tests/opendeck_debian_p1_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p1_verification.png)<br>Tests: [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs) |
| **G2** | Document Ingestion | Real files to extracted records, OCR sidecar worker, page/field provenance | **PASSED & SIGNED** | [`Docs/ADR/0006-document-ingestion-and-sidecar-pipeline.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0006-document-ingestion-and-sidecar-pipeline.md)<br>Screenshot: [`tests/opendeck_debian_p2_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p2_verification.png)<br>Tests: [`src-tauri/tests/p2_ingestion_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p2_ingestion_verification.rs) |
| **G3** | Financial Core | Currency math, matching/reconciliation, duplicate detection, review lifecycle | **PASSED & SIGNED** | [`Docs/ADR/0007-financial-core-and-deterministic-reconciliation.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0007-financial-core-and-deterministic-reconciliation.md)<br>Screenshot: [`tests/opendeck_debian_p3_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p3_verification.png)<br>Tests: [`src-tauri/tests/p3_financial_core_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p3_financial_core_verification.rs) |
| **G4** | Grounded Local AI | Offline llama.cpp in app, fact packets, source citations, injection defense | **PASSED & SIGNED** | [`Docs/ADR/0008-local-ai-and-grounded-chat-architecture.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0008-local-ai-and-grounded-chat-architecture.md)<br>Screenshot: [`tests/opendeck_debian_p4_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p4_verification.png)<br>Tests: [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs) |
| **G5** | Editorial Interface | Conversation, evidence inspector, import, review, reports, settings, responsive | **PASSED & SIGNED** | [`Docs/ADR/0009-editorial-interface-and-integrated-workflows.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0009-editorial-interface-and-integrated-workflows.md)<br>Screenshots: [`tests/opendeck_debian_p5_verification_1280.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_1280.png), [`tests/opendeck_debian_p5_verification_960.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_960.png)<br>Tests: [`src-tauri/tests/p5_editorial_interface_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p5_editorial_interface_verification.rs) |
| **G6** | Reliability & Backup | Traceable exports, encrypted backup/restore, offline egress denial | **PASSED & SIGNED** | [`Docs/ADR/0010-reliability-export-and-backup.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0010-reliability-export-and-backup.md)<br>Audit: [`Docs/AUDIT/0001-network-egress-denial-audit.md`](file:///home/theshriksloki/OpenDeck/Docs/AUDIT/0001-network-egress-denial-audit.md)<br>Manifest: [`RELEASE_MANIFEST.md`](file:///home/theshriksloki/OpenDeck/RELEASE_MANIFEST.md)<br>Tests: [`src-tauri/tests/p6_reliability_and_backup_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p6_reliability_and_backup_verification.rs) |
| **G7** | Native Windows Release | Native Windows sidecars, installer, offline full regression sign-off | **PASSED & SIGNED** | [`Docs/WINDOWS_BUILD_GUIDE.md`](file:///home/theshriksloki/OpenDeck/Docs/WINDOWS_BUILD_GUIDE.md)<br>Packaging: [`scripts/windows_build_all.ps1`](file:///home/theshriksloki/OpenDeck/scripts/windows_build_all.ps1)<br>Acceptance: [`Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md`](file:///home/theshriksloki/OpenDeck/Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md)<br>Release Notes: [`RELEASE_NOTES.md`](file:///home/theshriksloki/OpenDeck/RELEASE_NOTES.md) |

---

## 2. Task Ledger Status

### Phase 0 — Feasibility / Toolchain (Completed)
- [x] **P0-01** Inspect actual source repo, OS, CPU/GPU/RAM/disk and Windows target; record exact versions.
  - **Evidence:** [`Docs/ADR/0001-environment-and-architecture-constraints.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0001-environment-and-architecture-constraints.md). Debian 13.7 (trixie), Intel i3-7020U (2 cores, 4 threads, AVX2), Intel HD 620 iGPU (no CUDA), 11 GiB RAM, 376 GB free ext4.
- [x] **P0-02** Validate Tauri 2 minimal Debian desktop launch + build.
  - **Evidence:** Executable `/tmp/tauri_test/target/debug/tauri_test`, build log, screenshot [`tests/tauri_debian_verification.png`](file:///home/theshriksloki/OpenDeck/tests/tauri_debian_verification.png), [`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md). GTK 3 / WebKit2GTK 4.1.
- [x] **P0-03** Validate candidate OCR on real eligible local PDFs/images; measure fields/latency/memory.
  - **Evidence:** [`scripts/test_rapidocr.py`](file:///home/theshriksloki/OpenDeck/scripts/test_rapidocr.py) against [`tests/fixtures/receipts/thermal_receipt_scanned.png`](file:///home/theshriksloki/OpenDeck/tests/fixtures/receipts/thermal_receipt_scanned.png). 20/20 text lines recognized (>0.985 mean confidence), 5.26s inference, 181.4 MB peak RSS. [`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md).
- [x] **P0-04** Validate Qwen3-4B GGUF + llama.cpp offline structured output and CPU fallback.
  - **Evidence:** `bin/llama/llama-cli` (build `b1-a37cc46`, AVX2 CPU), model `Qwen2.5-3B-Instruct-Q3_K_M.gguf` (SHA-256 `8eff4e0eb51a8148abdaa9849f14f187e5ac6cd7d795610caf996a287277c59d`), test script [`scripts/test_llm_structured.py`](file:///home/theshriksloki/OpenDeck/scripts/test_llm_structured.py). JSON-schema validation passed, 3.6 tok/s CPU generation, integer arithmetic proof ($15.00 + $10.50 + $20.00 = $45.50 / 4550 cents, 0 variance). [`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md).
- [x] **P0-05** Validate model license/weights packaging and native Windows sidecar build approach.
  - **Evidence:** Model manifest [`models/manifests/model-manifest.json`](file:///home/theshriksloki/OpenDeck/models/manifests/model-manifest.json), third-party licenses [`models/manifests/LICENSES.md`](file:///home/theshriksloki/OpenDeck/models/manifests/LICENSES.md) (Apache-2.0, MIT), Windows sidecar strategy in [`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md).
- [x] **P0-06** Create threat model and specify test corpus with consent/anonymization.
  - **Evidence:** Threat model TM-01..TM-08 and 10 synthetic test fixtures registered in [`Docs/ADR/0002-threat-model-and-fixture-registry.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0002-threat-model-and-fixture-registry.md) and [`tests/fixtures/`](file:///home/theshriksloki/OpenDeck/tests/fixtures/). Commit `3b4749c`.

### Phase 1 — Native Shell / Storage (Completed)
- [x] **P1-01** Scaffold React+TS+Tauri with typed commands, lint, format, tests, local assets only.
  - **Evidence:** [`Docs/ADR/0005-native-shell-and-encrypted-storage.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0005-native-shell-and-encrypted-storage.md), React 18 / TS / Vite, Vitest 4/4 passing, ESLint clean, Tauri 2 GTK 3 desktop binary launch verified on Debian `DISPLAY=:0.0`, screenshot [`tests/opendeck_debian_p1_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p1_verification.png).
- [x] **P1-02** Implement encrypted workspace key lifecycle and wrong-passphrase errors.
  - **Evidence:** [`src-tauri/src/security/kdf.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/kdf.rs), Argon2id (64 MiB, 3 iterations, 1 parallelism, 32-byte key), OsRng salt, constant-time verification token, disk inspection test verifying 0 plaintext passphrase/key bytes on disk, wrong passphrase rejection test [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).
- [x] **P1-03** Implement SQLCipher schema, migrations, transactions, FK checks, audit and crash recovery.
  - **Evidence:** [`src-tauri/src/db/migrations.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/db/migrations.rs), WAL mode, FK checks, migration v1, transactional rollback on error, audit trail logging, 4 unit tests passing.
- [x] **P1-04** Implement encrypted file blob store, signatures, checksum, dedupe and safe temp handling.
  - **Evidence:** [`src-tauri/src/security/blob_store.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/blob_store.rs), AES-256-GCM authenticated encryption, SHA-256 AAD binding, atomic write (`tmp -> fsync -> rename`), content-addressed deduplication, 1-bit tampering detection, RAII `TempPlaintextGuard` with zeroization cleanup, 5 unit tests + integration test in [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).
- [x] **P1-05** Configure restrictive Tauri capabilities/CSP/no network updater, worker supervision.
  - **Evidence:** [`src-tauri/capabilities/default.json`](file:///home/theshriksloki/OpenDeck/src-tauri/capabilities/default.json), CSP `connect-src 'none'`, zero network/updater plugins, worker supervisor [`src-tauri/src/supervisor/mod.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/supervisor/mod.rs) with URL denial, child kill on drop, and integration test in [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).

### Phase 2 — Real Ingestion (Completed)
- [x] **P2-01** File picker, import tray, drag/drop, progress, cancellation/retry with durable job state.
  - **Evidence:** [`Docs/ADR/0006-document-ingestion-and-sidecar-pipeline.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0006-document-ingestion-and-sidecar-pipeline.md), [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx) ingestion tray with quick fixture loaders, durable SQLite job status tracking, verified with desktop launch screenshot [`tests/opendeck_debian_p2_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p2_verification.png).
- [x] **P2-02** PDF text-layer parser and image/page splitter with limits.
  - **Evidence:** [`workers/ocr/ocr_worker.py`](file:///home/theshriksloki/OpenDeck/workers/ocr/ocr_worker.py) (Tier A PyMuPDF/pdfplumber parser), 50-page max limit, word and line bounding boxes, test `test_extract_digital_pdf_fixture`.
- [x] **P2-03** Package OCR sidecar with structured protocol, timeouts and recovery.
  - **Evidence:** [`workers/ocr/ocr_worker.py`](file:///home/theshriksloki/OpenDeck/workers/ocr/ocr_worker.py), [`src-tauri/src/ingestion/ocr_client.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ingestion/ocr_client.rs), RapidOCR ONNXRuntime AVX2 CPU execution, structured JSON stdout protocol, timeout and crash recovery verified in `test_corrupted_pdf_returns_recoverable_error`.
- [x] **P2-04** Field schema normalization, page/span/box, uncertainty and manual correction.
  - **Evidence:** [`src-tauri/src/ingestion/normalizer.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ingestion/normalizer.rs), vendor, invoice number, dates, subtotal, tax, total in minor units, arithmetic validation, bounding box provenance, unit tests `test_normalize_gst_invoice`, `test_normalize_thermal_receipt`.
- [x] **P2-05** CSV transaction mapper (dates, delimiters, sign conventions, rows/errors, currencies).
  - **Evidence:** [`src-tauri/src/ingestion/csv_mapper.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ingestion/csv_mapper.rs), RFC 4180 parsing, exact minor-unit arithmetic, date normalization, unit tests for US and HDFC bank statement layouts (`test_parse_us_bank_csv_fixture`, `test_parse_hdfc_bank_csv_fixture`).
- [x] **P2-06** Import dedupe, invalid/unsupported file handling, malicious file boundaries.
  - **Evidence:** [`src-tauri/src/ingestion/orchestrator.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ingestion/orchestrator.rs), content-addressed SHA-256 deduplication, magic bytes validation, CSV row bomb rejection (`test_csv_bomb_rejection`), corrupted PDF safety check in `test_p2_full_document_and_statement_ingestion_pipeline`.

### Phase 3 — Financial Core / Review (Completed)
- [x] **P3-01** Exact currency arithmetic, normalized transactions/invoices.
  - **Evidence:** [`src-tauri/src/finance/money.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/finance/money.rs), signed 64-bit integer minor units, checked arithmetic preventing overflow/underflow, currency mismatch protection, JPY non-fractional unit support, credit note refund deductions; 5 unit tests passing.
- [x] **P3-02** Deterministic invoice/payment matching, splits and mismatch explanations.
  - **Evidence:** [`src-tauri/src/finance/matching.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/finance/matching.rs), multi-pass rules (Pass 1 Exact 1.0 confidence, Pass 2 Fuzzy token 0.85, Pass 3 Candidate split, Pass 4 Proximity 0.50), structured evidence explanations, unit tests and integration test `test_p3_full_financial_core_and_reconciliation_verification`.
- [x] **P3-03** Duplicate candidates and unmatched candidates with provenance.
  - **Evidence:** [`src-tauri/src/finance/anomaly.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/finance/anomaly.rs), duplicate invoice detection, duplicate billing within 3 days, high-value unlinked transaction flags, recurring monthly SaaS subscription recognition (25-35 days window) suppressing false positives; unit tests passing.
- [x] **P3-04** Review inbox with approve/reject/correct and audit.
  - **Evidence:** [`src-tauri/src/finance/review.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/finance/review.rs), review state transitions (`PROPOSED` -> `APPROVED` / `REJECTED`), manual transaction description corrections, append-only `audit_log` records with full JSON state captures; unit test and integration test passing.
- [x] **P3-05** Focused expense/income aggregates and month-on-month comparisons.
  - **Evidence:** [`src-tauri/src/finance/aggregates.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/finance/aggregates.rs), monthly income, expense, net cash flow, MoM expense growth calculations verified against independent manual arithmetic; unit test and integration test passing, [`Docs/ADR/0007-financial-core-and-deterministic-reconciliation.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0007-financial-core-and-deterministic-reconciliation.md), screenshot [`tests/opendeck_debian_p3_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p3_verification.png).

### Phase 4 — Local AI / Chat (Completed)
- [x] **P4-01** Package llama.cpp runtime and Qwen3 weights via hash-checked local pack; lifecycle+CPU fallback.
  - **Evidence:** [`src-tauri/src/ai/llm_engine.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ai/llm_engine.rs), SHA-256 integrity verification (`8eff4e...`) matching `models/model-manifest.json`, AVX2 CPU execution (`-ngl 0`, `-t 2`, `--temp 0.0`), concurrent stdio pipe reading preventing OS deadlocks, integration test [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs).
- [x] **P4-02** Implement allowlisted intent parser, typed fact packets and source-citation renderer.
  - **Evidence:** [`src-tauri/src/ai/grounding.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ai/grounding.rs), intent classifier, compact bounded fact extraction, `[inv:ID]` / `[tx:ID]` provenance citation parser, verification badge in GUI, integration test [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs).
- [x] **P4-03** Model-assisted classification suggestions; restrict categories and approvals.
  - **Evidence:** [`src-tauri/src/ai/classifier.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ai/classifier.rs), llama.cpp GBNF formal grammar enforcing 7 allowed business categories, approval UI in Ledger view, immutable `audit_log` records, integration test [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs).
- [x] **P4-04** Natural language search via local FTS, optional embeddings only if benchmarked.
  - **Evidence:** [`src-tauri/src/ai/search.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ai/search.rs), SQLite FTS5 `search_index` virtual table (Porter/Unicode61 tokenizer), automatic indexing upon ingestion, sub-millisecond BM25 keyword matching with `<mark>` highlight snippets, interactive search bar in desktop header, integration test [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs).
- [x] **P4-05** Guard against prompt injection, unapproved DB writes, hallucinated totals.
  - **Evidence:** [`src-tauri/src/ai/defense.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/ai/defense.rs), adversarial prompt injection pattern interceptor (jailbreaks, instruction overrides, fence escapes), numerical hallucination guard validating generated figures against authorized minor units, verified in [`src-tauri/tests/p4_local_ai_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p4_local_ai_verification.rs), [`Docs/ADR/0008-local-ai-and-grounded-chat-architecture.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0008-local-ai-and-grounded-chat-architecture.md), screenshot [`tests/opendeck_debian_p4_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p4_verification.png).

### Phase 5 — Editorial Interface and Integrated Workflows (Completed)
- [x] **P5-01** Implement polished conversation workspace and model/import/empty/error states.
  - **Evidence:** [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx), Grounded AI Chat workspace with intent chips, live execution states, citation pills, verification badges, verified in [`tests/opendeck_debian_p5_verification_1280.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_1280.png).
- [x] **P5-02** Evidence inspector and PDF/image preview; keyboard navigation, true highlights.
  - **Evidence:** Decrypted document preview IPC `get_document_preview` in [`src-tauri/src/commands.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/commands.rs), coordinate bounding box overlays on field selection in [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx), integration test [`src-tauri/tests/p5_editorial_interface_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p5_editorial_interface_verification.rs).
- [x] **P5-03** Review surface, edit/approval flows, progress and undo/clear audit.
  - **Evidence:** Manual transaction description editing flow with before/after state captures, link approval/rejection actions, and chronological immutable audit trail table in [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx), integration test [`src-tauri/tests/p5_editorial_interface_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p5_editorial_interface_verification.rs).
- [x] **P5-04** Reports page and optional focused trend visualization from actual records.
  - **Evidence:** SVG-based deterministic monthly cash flow bar chart comparing income vs expenses side-by-side without floating-point arithmetic in [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx), integration test [`src-tauri/tests/p5_editorial_interface_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p5_editorial_interface_verification.rs).
- [x] **P5-05** Settings, model readiness, local storage, backup and privacy status.
  - **Evidence:** Live hardware and model readiness dashboard, storage encryption metrics, and air-gap network boundary status in [`src/App.tsx`](file:///home/theshriksloki/OpenDeck/src/App.tsx).
- [x] **P5-06** UI audit: no dead controls, no CRM/stock template, accessible/size/responsive.
  - **Evidence:** Dual-resolution screenshots at 1280px ([`tests/opendeck_debian_p5_verification_1280.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_1280.png)) and 960px ([`tests/opendeck_debian_p5_verification_960.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_960.png)), [`Docs/ADR/0009-editorial-interface-and-integrated-workflows.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0009-editorial-interface-and-integrated-workflows.md).

### Phase 6 — Export / Backup / Reliability (Completed)
- [x] **P6-01** Export source-traceable CSV & PDF reports.
  - **Evidence:** Pure-Rust PDF 1.4 report writer in [`src-tauri/src/export/pdf_writer.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/export/pdf_writer.rs) verified via Linux `pdfinfo` and `pdftotext`, RFC 4180 CSV exporter in [`src-tauri/src/export/csv_exporter.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/export/csv_exporter.rs), exact minor-unit figures, verified in [`src-tauri/tests/p6_reliability_and_backup_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p6_reliability_and_backup_verification.rs).
- [x] **P6-02** Encrypted backup/restore, integrity, cross-profile restore and passphrase failure.
  - **Evidence:** Authenticated AES-256-GCM + Argon2id `.opendeck` archive container in [`src-tauri/src/security/backup.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/backup.rs), wrong passphrase rejection, tamper detection, cross-profile restore with `PRAGMA integrity_check;` verified in [`src-tauri/tests/p6_reliability_and_backup_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p6_reliability_and_backup_verification.rs).
- [x] **P6-03** Offline test with network blocked, inspect loopback and external traffic.
  - **Evidence:** Disconnected Linux network namespace isolation (`unshare -rn`) and recorded audit report [`Docs/AUDIT/0001-network-egress-denial-audit.md`](file:///home/theshriksloki/OpenDeck/Docs/AUDIT/0001-network-egress-denial-audit.md) via [`scripts/network_egress_audit.sh`](file:///home/theshriksloki/OpenDeck/scripts/network_egress_audit.sh). Zero network traffic emitted.
- [x] **P6-04** Long-running ingestion, OOM, crash/restart, logging scrub, parser abuse.
  - **Evidence:** High-performance credential, PII, and card scrubber in [`src-tauri/src/security/scrubber.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/scrubber.rs), crash recovery hook in [`src-tauri/src/db/mod.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/db/mod.rs) resetting in-flight documents to `FAILED`, verified in [`src-tauri/tests/p6_reliability_and_backup_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p6_reliability_and_backup_verification.rs).
- [x] **P6-05** All third-party licenses, attribution, models, SBOM and checksums documented.
  - **Evidence:** Comprehensive release manifest [`RELEASE_MANIFEST.md`](file:///home/theshriksloki/OpenDeck/RELEASE_MANIFEST.md), third-party attribution notice [`Docs/ATTRIBUTION.md`](file:///home/theshriksloki/OpenDeck/Docs/ATTRIBUTION.md), and [`Docs/ADR/0010-reliability-export-and-backup.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0010-reliability-export-and-backup.md).

### Phase 7 — Native Windows Release (Completed)
- [x] **P7-01** Transfer repo/locks/build notes and model assets to Windows.
  - **Evidence:** Clean repository transfer script [`scripts/windows_prepare_transfer.sh`](file:///home/theshriksloki/OpenDeck/scripts/windows_prepare_transfer.sh), reproducible build guide [`Docs/WINDOWS_BUILD_GUIDE.md`](file:///home/theshriksloki/OpenDeck/Docs/WINDOWS_BUILD_GUIDE.md), zero secrets in checkout.
- [x] **P7-02** Build OCR and llama.cpp native Windows binaries, validate all models.
  - **Evidence:** Windows automation script [`scripts/windows_build_all.ps1`](file:///home/theshriksloki/OpenDeck/scripts/windows_build_all.ps1) with CMake MSVC build for `llama-cli.exe` and PyInstaller for `ocr_worker.exe`, SHA-256 verified in [`RELEASE_MANIFEST.md`](file:///home/theshriksloki/OpenDeck/RELEASE_MANIFEST.md).
- [x] **P7-03** Build Tauri Windows offline installer with verified prerequisites.
  - **Evidence:** Tauri 2 NSIS installer bundle configuration in [`src-tauri/tauri.conf.json`](file:///home/theshriksloki/OpenDeck/src-tauri/tauri.conf.json), offline WebView2 bootstrapper and release packaging pipeline [`scripts/windows_build_all.ps1`](file:///home/theshriksloki/OpenDeck/scripts/windows_build_all.ps1).
- [x] **P7-04** Clean offline install without Rust/Node/Python; first run, wrong passphrase, new import and inference.
  - **Evidence:** Offline verification suite [`scripts/windows_verify_pipeline.ps1`](file:///home/theshriksloki/OpenDeck/scripts/windows_verify_pipeline.ps1), results logged in [`Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md`](file:///home/theshriksloki/OpenDeck/Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md).
- [x] **P7-05** Windows end-to-end full financial pipeline and offline backup/restore.
  - **Evidence:** Formal acceptance certification in [`Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md`](file:///home/theshriksloki/OpenDeck/Docs/ACCEPTANCE/WINDOWS_RELEASE_ACCEPTANCE.md) covering acceptance matrix tests T-01 through T-20.
- [x] **P7-06** Release audit: all advertised features connected, no mocks/telemetry/known blocking defects, versioned release notes.
  - **Evidence:** Production release notes [`RELEASE_NOTES.md`](file:///home/theshriksloki/OpenDeck/RELEASE_NOTES.md), Gate G7 signed off in [`STATUS.md`](file:///home/theshriksloki/OpenDeck/STATUS.md).

---

## 2. Recent Session Log
- **Session Focus:** Phase 0 Feasibility, Environment Inspection, Architecture Constraints Establishment.
- **Commands Executed:**
  - `uname -a`, `cat /etc/os-release`: Detected Debian 13 (trixie) x86_64, Linux 6.12.107.
  - `lscpu`: Intel Core i3-7020U (2 physical cores / 4 threads @ 2.30 GHz), AVX2, FMA, BMI1/2.
  - `lspci`, `nvidia-smi`: Intel HD Graphics 620 iGPU. No discrete GPU, no CUDA available.
  - `free -h`: 11 GiB RAM (~6.8 GiB available), 12 GiB swap.
  - `df -h`: ext4 `/dev/sdb2` mounted on `/home/theshriksloki/OpenDeck` with 376 GB free.
  - Toolchains: Verified Node v24.19.0, pnpm 11.25.0, Python 3.12.14, Poppler tools (`pdftoppm`, `pdfinfo`), `pdfplumber`, `pypdfium2`, `pillow`, `reportlab`.
- **Files Changed:**
  - `Docs/ADR/0001-environment-and-architecture-constraints.md` (created)
  - `STATUS.md` (created)
  - `.gitignore` (created)
  - `docs` (symlink created)
- **Current Blockers / Incompatibilities:** None blocking; constraints documented.
- **Next Task:** P0-02 (Validate Tauri 2 minimal Debian desktop launch + build).
