# Technical Specification — Offline Desktop Application

## 1. Target runtime (proposed stack)
- **Desktop shell:** Tauri 2.x, native Windows application using bundled local HTML/CSS/TypeScript frontend, not a website or hosted dashboard.
- **Presentation:** React + TypeScript + Vite + custom CSS. Strongly typed app services and components; do not install generic admin-dashboard templates.
- **Trusted core:** Rust Tauri commands, local file ingestion, SQL/SQLCipher, encryption/key management, deterministic accounting/reconciliation, export, IPC and model lifecycle.
- **Database:** SQLCipher via an actively maintained Rust binding/build mode, evaluated on Debian and Windows. SQLite migrations, FK constraints, transactions, WAL handling and backup checkpoint rules required.
- **Local document worker:** Python-based OCR worker (PaddleOCR), launched as a bundled **Tauri sidecar**. Worker must be executable on each target OS without global Python/pip or model downloads.
- **Local text model:** Qwen3-4B GGUF quantized candidate under Apache-2.0, driven by bundled llama.cpp binary/library. **Model candidate, not untested commitment to performance.** CPU fallback mandatory; 8 GB VRAM target conditional on hardware test. No Ollama dependency for end-users.
- **OCR:** baseline PP-OCRv6 small/medium and text-layer PDF extraction; candidate **PaddleOCR-VL-1.6 (0.9B)** behind adapter/benchmark gate. Use only if startup, inference quality, model distribution, licensing and Windows packaging are verified. Do not silently add a new VLM runtime that makes Windows shipping impossible.
- **App output:** native file exports (CSV and PDF) written locally. No server URLs or CDN fonts/assets.

**Rationale:** Tauri allows polished desktop UI and Tauri-documented packaged sidecars. Build and test native Windows artifacts **on Windows**, since Linux-to-Windows cross-compilation/packaging has caveats. For the OCR worker, packaging must occur on each target OS (PyInstaller is not a cross-compiler).

## 2. Component boundaries
```
Tauri Window (React UI)
  ├─ Chat + task composer, document rail, review queue, evidence inspector
  └─ Typed invoke() commands and Tauri events ONLY
          ↓
Rust trusted core
  ├─ Authorization/permissions + typed intents + validation
  ├─ SQLCipher database, migration, audit + encrypted file store
  ├─ Exact arithmetic / matching / report engine (no LLM math)
  ├─ OCR worker supervisor → local Python executable (sidecar)
  ├─ LLM supervisor → bundled llama.cpp binary or local library
  └─ Offline export / backup / restore / log redaction
```

- UI does **not** read arbitrary disk paths or call external endpoints.
- Sidecars never receive database passphrases. Rust supplies necessary file content through a controlled IPC protocol or isolated temporary decrypted buffers; cleanup is guaranteed on success/failure.
- Preferred model communication: private local process IPC. If llama.cpp uses loopback HTTP, bind to `127.0.0.1` only, allocate randomized available port, use a per-launch strong token, block remote listening, never expose it through the UI, terminate on parent exit, and cover with tests. Loopback is local, not cloud.
- Worker calls are bounded by timeouts, memory budget, cancellation, and structured error types. No silent fallbacks to hosted inference.
- Log **metadata only** by default. No invoices, bank numbers, raw prompt content, unredacted paths or secrets.

## 3. Local model policy
Models are on-device **after offline installation/model provisioning**. Model assets must be explicitly packaged in an offline model pack or read from a user-chosen local directory; there is no runtime auto-download. `MODEL_MANIFEST.json` must include name, exact file hash, source, license URL, quantization, minimum runtime version, RAM/VRAM estimate (measured or clearly labeled estimate), and supported target platforms.

For the text model: produce JSON following schema; parse and validate Rust-side; allowlist categories; include evidence IDs; limit output length, temperature, cost and time. Never let the LLM write SQL, update data directly or invent command execution. Sources are untrusted: treat documents and user text as **data**, not directives overriding system rules.

For OCR: favor digital text layer first, then local OCR. Extract source spans/boxes and page numbers. Monetary/date/vendor fields remain proposals until rule validation and, when material/ambiguous, user review. Keep OCR provider pluggable: `TextLayerExtractor`, `TraditionalOCR`, `VLDocumentExtractor` implementations share a typed interface.

## 4. Data model (minimum)
IDs are UUID/ULID; times UTC plus local display; amounts stored as integer minor units with ISO 4217 currency and original value/scale retained where necessary.

| Entity | Essential fields |
|---|---|
| `workspaces` | id, display_name, base_currency, schema_version, created_at, passphrase_params |
| `documents` | id, workspace_id, encrypted_blob_ref, sha256, type, name, imported_at, page_count, parse_state |
| `document_fields` | id, document_id, key, original_text, normalized_value, page, span/box, extractor_id/version, confidence, review_state |
| `transactions` | id, source_doc_id, posted_date, description, amount_minor, currency, direction, external_ref, provenance, approval_state |
| `invoices` | id, document_id, supplier_customer, invoice_number, invoice_date, due_date, amount_minor, currency, state |
| `invoice_payment_links` | id, invoice_id, transaction_id, matched_amount_minor, confidence_reason, status, approved_by_user |
| `category_suggestions` | id, transaction_id, proposed_category, model_id, evidence_json, status |
| `reviews` | id, target_type, target_id, proposed_change, reason, status, decided_at |
| `audit_events` | id, target_id, action, before_hash, after_hash, timestamp, actor, provenance |
| `chat_threads/messages` | id, role, text, created_at, referenced_record_ids, operation_type |
| `report_exports` | id, period, currencies, file_path, timestamp, record_snapshot_hash |
| `model_installations` | model_id, version, manifest_hash, local_path_ref, readiness, last_verified |

All link IDs must reference real local records (enforced with foreign keys and Rust validation). Uploaded data remains untrusted until parsed and validated. Do not store OCR/AI claims as authoritative financial facts without explicit state change.

## 5. Ingestion state machine
`SELECTED → IMPORTING → STORED → EXTRACTING → EXTRACTED → NEEDS_REVIEW → VERIFIED`.
Branches: `FAILED` with reason/retry, `CANCELLED`, `UNSUPPORTED`; `VERIFIED` only after required checks/approval. Retried imports use checksum-based dedupe and must not create duplicates accidentally. All steps durable across crash/restart. Progress reports actual files/pages processed, not arbitrary percentages.

## 6. Financial invariants
- Exact decimal/arbitrary precision arithmetic, currency-aware.
- Sum of matched payment allocations cannot exceed observed eligible transaction amount or invoice amount without explicit exceptions.
- No automatic classification of payment status from absence of bank records; label `not matched in imported data` rather than `unpaid`.
- Overpayments, split payments, credits/refunds, identical amounts, differing currencies, and date windows must be treated explicitly.
- No invented exchange rate, GST treatment, journal entry or tax liability.
- Source hash, immutable raw import and audit chain retained even after user corrections.

## 7. Chat intent and operation security
- Read-only intents: `summarize_period`, `search_documents`, `list_unmatched`, `explain_transaction`, `find_duplicates`, `compare_periods`.
- Mutating intents: `propose_category_change`, `propose_match`, `prepare_report`, `delete_workspace`. All writes go through preview + explicit user confirmation; destructive actions require specific confirmation.
- Assistant text must be produced from a typed **fact packet** from the local database, with source IDs and exact computed numeric results. Unsupported queries return an honest limitation and offer a supported action.
- LLM returns suggested language only. Deterministic core decides eligibility, access, arithmetic, state transitions and audit entries.

## 8. Security and offline boundary
- No remote URLs in production `connect-src`, no arbitrary shell execution, no automatic browser opening, no remote scripts/fonts/image assets.
- Minimize Tauri permissions/capabilities; restrict filesystem scope and command schemas.
- Encrypt persisted DB and all stored imported documents/derived previews. Avoid unencrypted temp files; secure cleanup and OS-specific data handling tested.
- Passphrase-derived master key via vetted memory-hard KDF, salt and authenticated encryption; never hard-code keys. Ensure inability to recover forgotten passphrases is communicated.
- Encrypted backups have versioned schema, integrity/authentication, manifest, password-based decrypt and a test of restore on another device. Do not assume SQLCipher alone encrypts exported PDFs: exports are user-directed plaintext files with explicit warning/choice.
- Security checks on PDF/image parsers, decompression bombs, giant CSV rows, path traversal, extensions vs signatures, worker crashes, and malicious document text.
- Disable telemetry, auto-updaters, issue reporting uploads, analytics SDKs and model downloads.

## 9. Local workspace layout (suggested)
```
/src                 React views, workspace state, document viewer, chat, review
/src-tauri           Rust core, migrations, commands, sidecar supervision, export
/workers/ocr         Python local OCR implementation and interface
/workers/llm         model runtime wrapper/manifests/config
/models/manifests    model metadata only; large weights excluded from Git
/tests              local end-to-end tests and private-safe reference corpus
/scripts            Debian dev checks, native Windows build/verify scripts
/docs               architecture decision records and evidence screenshots
```

## 10. Packaging gates
**Debian:** can run development build and Debian binary, test local OCR/LLM with actual hardware.  
**Windows:** after repo transfer, install toolchains once for BUILD only; compile Rust + Python worker/llama.cpp for Windows, create Tauri installer with offline WebView2 strategy, supply model pack, clean-install on a different Windows account with no dev tools and no network. `No network after setup` isn't sufficient: **initial offline installation** must be covered if release claims full offline installation. Installer size must be measured, not invented.

Native Windows build and full testing are mandatory. Linux cross-build alone is not final proof.
