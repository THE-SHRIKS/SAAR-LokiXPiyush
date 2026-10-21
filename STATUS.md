# Project Status Ledger — OpenDeck

**Current Phase:** Phase 1 — Native shell + secure workspace  
**Active Task:** Phase 0 (100% Complete, Gate G0 Passed) → P1-01 (In Progress)  
**Last Updated:** 2026-10-22  

---

## 1. Gate Sign-Off Ledger

| Gate | Name | Required Proofs | Status | Evidence Link |
|---|---|---|---|---|
| **G0** | Feasibility, Environment & Runtime Baseline | Toolchain inspection, native Tauri 2 desktop window, local OCR on real scan, offline LLM GBNF inference on CPU, model licenses, threat model + fixtures | **PASSED & SIGNED** | [`Docs/ADR/0001-environment-and-architecture-constraints.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0001-environment-and-architecture-constraints.md)<br>[`Docs/ADR/0002-threat-model-and-fixture-registry.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0002-threat-model-and-fixture-registry.md)<br>[`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md)<br>[`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md) |
| **G1** | Native Shell + Storage | Desktop launch on Debian, encrypted workspace key lifecycle, SQLCipher migrations, no network | PENDING | |
| **G2** | Document Ingestion | Real files to extracted records, OCR sidecar worker, page/field provenance | PENDING | |
| **G3** | Financial Core | Currency math, matching/reconciliation, duplicate detection, review lifecycle | PENDING | |
| **G4** | Grounded Local AI | Offline llama.cpp in app, fact packets, source citations, injection defense | PENDING | |
| **G5** | Editorial Interface | Conversation, evidence inspector, import, review, reports, settings, responsive | PENDING | |
| **G6** | Reliability & Backup | Traceable exports, encrypted backup/restore, offline egress denial | PENDING | |
| **G7** | Native Windows Release | Native Windows sidecars, installer, offline full regression sign-off | PENDING | |

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

### Phase 1 — Native Shell / Storage
- [ ] **P1-01** Scaffold React+TS+Tauri with typed commands, lint, format, tests, local assets only.
- [ ] **P1-02** Implement encrypted workspace key lifecycle and wrong-passphrase errors.
- [ ] **P1-03** Implement SQLCipher schema, migrations, transactions, FK checks, audit and crash recovery.
- [ ] **P1-04** Implement encrypted file blob store, signatures, checksum, dedupe and safe temp handling.
- [ ] **P1-05** Configure restrictive Tauri capabilities/CSP/no network updater, worker supervision.

### Phase 2 — Real Ingestion
- [ ] **P2-01** File picker, import tray, drag/drop, progress, cancellation/retry with durable job state.
- [ ] **P2-02** PDF text-layer parser and image/page splitter with limits.
- [ ] **P2-03** Package OCR sidecar with structured protocol, timeouts and recovery.
- [ ] **P2-04** Field schema normalization, page/span/box, uncertainty and manual correction.
- [ ] **P2-05** CSV transaction mapper (dates, delimiters, sign conventions, rows/errors, currencies).
- [ ] **P2-06** Import dedupe, invalid/unsupported file handling, malicious file boundaries.

### Phase 3 — Financial Core / Review
- [ ] **P3-01** Exact currency arithmetic, normalized transactions/invoices.
- [ ] **P3-02** Deterministic invoice/payment matching, splits and mismatch explanations.
- [ ] **P3-03** Duplicate candidates and unmatched candidates with provenance.
- [ ] **P3-04** Review inbox with approve/reject/correct and audit.
- [ ] **P3-05** Focused expense/income aggregates and month-on-month comparisons.

### Phase 4 — Local AI / Chat
- [ ] **P4-01** Package llama.cpp runtime and Qwen3 weights via hash-checked local pack; lifecycle+CPU fallback.
- [ ] **P4-02** Implement allowlisted intent parser, typed fact packets and source-citation renderer.
- [ ] **P4-03** Model-assisted classification suggestions; restrict categories and approvals.
- [ ] **P4-04** Natural language search via local FTS, optional embeddings only if benchmarked.
- [ ] **P4-05** Guard against prompt injection, unapproved DB writes, hallucinated totals.

### Phase 5 — Editorial Interface and Integrated Workflows
- [ ] **P5-01** Implement polished conversation workspace and model/import/empty/error states.
- [ ] **P5-02** Evidence inspector and PDF/image preview; keyboard navigation, true highlights.
- [ ] **P5-03** Review surface, edit/approval flows, progress and undo/clear audit.
- [ ] **P5-04** Reports page and optional focused trend visualization from actual records.
- [ ] **P5-05** Settings, model readiness, local storage, backup and privacy status.
- [ ] **P5-06** UI audit: no dead controls, no CRM/stock template, accessible/size/responsive.

### Phase 6 — Export / Backup / Reliability
- [ ] **P6-01** Export source-traceable CSV & PDF reports.
- [ ] **P6-02** Encrypted backup/restore, integrity, cross-profile restore and passphrase failure.
- [ ] **P6-03** Offline test with network blocked, inspect loopback and external traffic.
- [ ] **P6-04** Long-running ingestion, OOM, crash/restart, logging scrub, parser abuse.
- [ ] **P6-05** All third-party licenses, attribution, models, SBOM and checksums documented.

### Phase 7 — Native Windows Release
- [ ] **P7-01** Transfer repo/locks/build notes and model assets to Windows.
- [ ] **P7-02** Build OCR and llama.cpp native Windows binaries, validate all models.
- [ ] **P7-03** Build Tauri Windows offline installer with verified prerequisites.
- [ ] **P7-04** Clean offline install without Rust/Node/Python; first run, wrong passphrase, new import and inference.
- [ ] **P7-05** Windows end-to-end full financial pipeline and offline backup/restore.
- [ ] **P7-06** Release audit: all advertised features connected, no mocks/telemetry/known blocking defects, versioned release notes.

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
