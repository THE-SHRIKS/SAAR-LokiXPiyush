# Implementation Task Ledger

**Instructions:** All tasks start unchecked; NOTHING below is presently implemented. Add evidence links and commit IDs when completed. An unchecked task cannot be described as finished. Task IDs map to `TEST_PLAN.md`.

## P0 — Feasibility / toolchain
- [x] **P0-01** Inspect actual source repo, OS, CPU/GPU/RAM/disk and Windows target; record exact versions. **Proof:** [`Docs/ADR/0001-environment-and-architecture-constraints.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0001-environment-and-architecture-constraints.md), commit `ad4207f`.
- [x] **P0-02** Validate Tauri 2 minimal Debian desktop launch + build. **Proof:** executable `/tmp/tauri_test/target/debug/tauri_test`, build log, screenshot [`tests/tauri_debian_verification.png`](file:///home/theshriksloki/OpenDeck/tests/tauri_debian_verification.png), [`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md).
- [x] **P0-03** Validate candidate OCR on real eligible local PDFs/images; measure fields/latency/memory. **Proof:** RapidOCR ONNXRuntime script [`scripts/test_rapidocr.py`](file:///home/theshriksloki/OpenDeck/scripts/test_rapidocr.py), 20/20 fields extracted, 5.26s latency, 181.4 MB peak RSS, [`Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0003-desktop-shell-and-ocr-feasibility.md).
- [x] **P0-04** Validate Qwen3-4B GGUF + llama.cpp offline structured output and CPU fallback. **Proof:** SHA-256 hash match (`8eff4e...`), GBNF contract test [`scripts/test_llm_structured.py`](file:///home/theshriksloki/OpenDeck/scripts/test_llm_structured.py), 3.6 tok/s CPU generation, integer arithmetic proof, [`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md).
- [x] **P0-05** Validate model license/weights packaging and native Windows sidecar build approach. **Proof:** pinned manifest [`models/manifests/model-manifest.json`](file:///home/theshriksloki/OpenDeck/models/manifests/model-manifest.json), license records [`models/manifests/LICENSES.md`](file:///home/theshriksloki/OpenDeck/models/manifests/LICENSES.md), [`Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0004-local-model-runtime-and-sidecar-specification.md).
- [x] **P0-06** Create threat model and specify test corpus with consent/anonymization. **Proof:** threat table TM-01..TM-08 and fixture registry in [`Docs/ADR/0002-threat-model-and-fixture-registry.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0002-threat-model-and-fixture-registry.md), 10 test fixtures in [`tests/fixtures/`](file:///home/theshriksloki/OpenDeck/tests/fixtures/), commit `3b4749c`.

## P1 — Native shell / storage
- [x] **P1-01** Scaffold React+TS+Tauri with typed commands, lint, format, tests, local assets only. **Proof:** [`Docs/ADR/0005-native-shell-and-encrypted-storage.md`](file:///home/theshriksloki/OpenDeck/Docs/ADR/0005-native-shell-and-encrypted-storage.md), React 18 / TS / Vite workspace, Vitest tests 4/4 passing, ESLint clean, Tauri 2 GTK 3 desktop binary running on Debian `DISPLAY=:0.0`, screenshot [`tests/opendeck_debian_p1_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p1_verification.png).
- [x] **P1-02** Implement encrypted workspace key lifecycle and wrong-passphrase errors. **Proof:** [`src-tauri/src/security/kdf.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/kdf.rs), Argon2id (64 MiB, 3 iterations, 1 parallelism, 32-byte key), OsRng salt, constant-time verification token, disk inspection test verifying 0 plaintext passphrase/key bytes on disk, wrong passphrase rejection test [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).
- [x] **P1-03** Implement SQLCipher schema, migrations, transactions, FK checks, audit and crash recovery. **Proof:** [`src-tauri/src/db/migrations.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/db/migrations.rs), WAL mode, FK checks, migration v1, transactional rollback on error, audit trail logging, unit tests `test_schema_migration_idempotent`, `test_foreign_key_constraints_enforced`, `test_transaction_rollback_on_error`, `test_audit_log_records_creation`.
- [x] **P1-04** Implement encrypted file blob store, signatures, checksum, dedupe and safe temp handling. **Proof:** [`src-tauri/src/security/blob_store.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/security/blob_store.rs), AES-256-GCM authenticated encryption, SHA-256 AAD binding, atomic write (`tmp -> fsync -> rename`), content-addressed deduplication, 1-bit tampering detection, RAII `TempPlaintextGuard` with zeroization cleanup, integration test in [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).
- [x] **P1-05** Configure restrictive Tauri capabilities/CSP/no network updater, worker supervision. **Proof:** [`src-tauri/capabilities/default.json`](file:///home/theshriksloki/OpenDeck/src-tauri/capabilities/default.json), CSP `connect-src 'none'`, zero network/updater plugins, worker supervisor [`src-tauri/src/supervisor/mod.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/src/supervisor/mod.rs) with URL denial, child kill on drop, and integration test in [`src-tauri/tests/p1_native_storage_verification.rs`](file:///home/theshriksloki/OpenDeck/src-tauri/tests/p1_native_storage_verification.rs).

## P2 — Real ingestion
- [ ] **P2-01** File picker, import tray, drag/drop, progress, cancellation/retry with durable job state. **Proof:** real import video, restart behavior.
- [ ] **P2-02** PDF text-layer parser and image/page splitter with limits. **Proof:** actual file tests.
- [ ] **P2-03** Package OCR sidecar with structured protocol, timeouts and recovery. **Proof:** actual extraction from bundled worker on Debian.
- [ ] **P2-04** Field schema normalization, page/span/box, uncertainty and manual correction. **Proof:** click field opens its original source.
- [ ] **P2-05** CSV transaction mapper (dates, delimiters, sign conventions, rows/errors, currencies). **Proof:** import of two real bank export layouts.
- [ ] **P2-06** Import dedupe, invalid/unsupported file handling, malicious file boundaries. **Proof:** test matrix.

## P3 — Financial core / review
- [ ] **P3-01** Exact currency arithmetic, normalized transactions/invoices. **Proof:** unit tests for boundaries/refunds/multiple currencies.
- [ ] **P3-02** Deterministic invoice/payment matching, splits and mismatch explanations. **Proof:** reference ledger cases.
- [ ] **P3-03** Duplicate candidates and unmatched candidates with provenance. **Proof:** cases + false-positive suppression.
- [ ] **P3-04** Review inbox with approve/reject/correct and audit. **Proof:** persisted real record lifecycle.
- [ ] **P3-05** Focused expense/income aggregates and month-on-month comparisons. **Proof:** compare with independent hand calculations.

## P4 — Local AI / chat
- [ ] **P4-01** Package llama.cpp runtime and Qwen3 weights via hash-checked local pack; lifecycle+CPU fallback. **Proof:** offline inference in app.
- [ ] **P4-02** Implement allowlisted intent parser, typed fact packets and source-citation renderer. **Proof:** no fictitious facts from query test.
- [ ] **P4-03** Model-assisted classification suggestions; restrict categories and approvals. **Proof:** reviewed real records.
- [ ] **P4-04** Natural language search via local FTS, optional embeddings only if benchmarked. **Proof:** find local document with source navigation.
- [ ] **P4-05** Guard against prompt injection, unapproved DB writes, hallucinated totals. **Proof:** adversarial document tests.

## P5 — UI polish / integration
- [ ] **P5-01** Implement polished conversation workspace and model/import/empty/error states. **Proof:** screenshot and working controls.
- [ ] **P5-02** Evidence inspector and PDF/image preview; keyboard navigation, true highlights. **Proof:** real citations to document pages.
- [ ] **P5-03** Review surface, edit/approval flows, progress and undo/clear audit. **Proof:** E2E sequence.
- [ ] **P5-04** Reports page and optional focused trend visualization from actual records. **Proof:** data matches core calculations.
- [ ] **P5-05** Settings, model readiness, local storage, backup and privacy status. **Proof:** statuses derived from running services.
- [ ] **P5-06** UI audit: no dead controls, no CRM/stock template, accessible/size/responsive. **Proof:** review captures at two widths + tab-only flow.

## P6 — Export/backup/reliability
- [ ] **P6-01** Export source-traceable CSV & PDF reports. **Proof:** independent recalculation + actual files.
- [ ] **P6-02** Encrypted backup/restore, integrity, cross-profile restore and passphrase failure. **Proof:** restore test and hashes.
- [ ] **P6-03** Offline test with network blocked, inspect loopback and external traffic. **Proof:** recorded network audit.
- [ ] **P6-04** Long-running ingestion, OOM, crash/restart, logging scrub, parser abuse. **Proof:** test results and issues closed.
- [ ] **P6-05** All third-party licenses, attribution, models, SBOM and checksums documented. **Proof:** release manifest.

## P7 — Windows release
- [ ] **P7-01** Transfer repo/locks/build notes and model assets to Windows. **Proof:** cloned checkout; no environment secrets needed.
- [ ] **P7-02** Build OCR and llama.cpp native Windows binaries, validate all models. **Proof:** native inference and hashes.
- [ ] **P7-03** Build Tauri Windows offline installer with verified prerequisites. **Proof:** installer output + measured size/hash.
- [ ] **P7-04** Clean offline install without Rust/Node/Python; first run, wrong passphrase, new import and inference. **Proof:** video and logs.
- [ ] **P7-05** Windows end-to-end full financial pipeline and offline backup/restore. **Proof:** signed acceptance report.
- [ ] **P7-06** Release audit: all advertised features connected, no mocks/telemetry/known blocking defects, versioned release notes. **Proof:** `STATUS.md` gate G7 signed.
