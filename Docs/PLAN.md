# Execution Plan and Gates

## Strategy
We are not creating a prototype made of unconnected views. Build a vertical slice of **real import → extracted provenance → verified data → answer/report**, then deepen each stage. Feature acceptance is based on executable outcomes, not screenshots alone.

### Phase 0 — Discovery, feasibility and baseline
Inspect Debian environment (architecture, RAM, GPU, CUDA drivers, disk, Python/Rust/Node), the existing repo and the eventual Windows target. Decide supported file types/limits. Verify PaddleOCR deployment and Qwen3-4B GGUF source hashes, runtime binary, local quantization and license. Run OCR and llama.cpp inference **from a real local document** outside UI, record actual timing/memory. Verify packaging approach for native Windows. **Gate G0:** architecture decision record, working CLI inference, known hardware constraints, model manifest.

### Phase 1 — Native shell + secure workspace
Initialize Tauri 2, React/TS, typed bridge, filesystem capabilities, SQLCipher, authenticated encryption, passphrase flow and secure local file store. Build app skeleton with genuine states. **Gate G1:** desktop launches on Debian, encrypted workspace persists through restart, passphrase rejects incorrectly, no runtime network.

### Phase 2 — Document ingestion
Implement file picker/drop, signature check, encrypted storage, PDF text extraction, OCR worker and CSV column mapping, progress, cancellation, retry, page/field provenance. **Gate G2:** real eligible files become actual extracted records; no hidden fake success; parser failures recover safely.

### Phase 3 — Deterministic financial core
Implement exact money representations, import normalization, transactions, invoices, reconciliation candidate matching, duplicate candidates, immutable audit, exception queue and user review. **Gate G3:** reference test cases pass including duplicates/splits/refunds/mismatches/currencies; user corrections survive restart.

### Phase 4 — Local model and grounded chat
Bundle llama.cpp, candidate Qwen3 4B quantization, strict intents/fact packets, model category proposals, source-backed read-only chat and explicit approval for writes. **Gate G4:** user query computes correct totals from persisted records with direct citations; prompt injection and unsupported questions cannot change data.

### Phase 5 — Editorial interface and integrated workflows
Refine all screens against `UI_DESIGN.md`: conversation, source inspector, import, review, reports and settings. Chart is an on-demand view. Keyboard, responsive layout, errors, progress, crash recovery. **Gate G5:** full workflow on Debian, no dead controls, screenshots and actual recording for each major user path.

### Phase 6 — Exports, backups, security, offline hardening
Implement CSV/PDF reports with source manifests, passphrase-protected encrypted backup/restore, log redaction and parser limits. Test airplane mode and deny-all-egress. **Gate G6:** reports match deterministic source totals, restoration on another profile works, traffic zero, no plaintext leaks in managed storage.

### Phase 7 — Transfer and native Windows release
Transfer source repository, **not just Linux binaries**, plus documented model assets and licenses. Recreate locked build toolchain on Windows. Build Windows sidecars and installer natively, bundle offline WebView2 option where needed, verify model startup (CPU and GPU if supported), import real docs, finish reports and backup. Perform clean offline install and smoke/full regressions. **Gate G7:** signed-off native Windows release evidence pack; no known blocking defects.

## Dependencies and parallel work
- UI shell may proceed alongside document/OCR experimentation, **but UI must never present unimplemented outcomes**.
- Rust storage and accounting math land before AI-generated explanations and reports.
- Local model decisions remain reversible behind adapters until Windows compatibility and accuracy are proven.
- Windows release work should have a feasibility test early; **do not defer discovering incompatible model packaging until the final phase**.

## Release boundary and risk
Major risks: OCR runtime support on Windows, GPU memory contention between models, secure encrypted temp file handling, model bundle size, WebView2 offline installation, user data safety, PDF parser attacks and finance accuracy. These are gating risks, not cosmetic issues.

Do not promise a calendar completion date absent an actual team/available hours and successful feasibility gates. Report real milestone state instead.

## Decision log (initial)
| Item | Working decision | Status |
|---|---|---|
| Desktop framework | Tauri 2 + React TS + Rust | Proposed, verify in Phase 0 |
| OCR | PP-OCRv6 baseline, PaddleOCR-VL-1.6 as benchmark candidate | Benchmark/Windows compatibility pending |
| Language model | Qwen3-4B quantized local llama.cpp | Compatibility/performance pending |
| Database | SQLCipher encrypted local | Binding/build/keys to validate |
| Target | Windows first; Debian development | Confirm Windows hardware/version |
| Cloud | None at runtime | Hard requirement |
| Product identity | Original editorial workspace, chat-first | UI approval pending |
| Scope | Financial document operations, not certified bookkeeping | Hard requirement |
