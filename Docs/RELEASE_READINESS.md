# SAAR — Release Readiness (this pass)

## Verdict

**NOT READY — FAILING GATES.** Several mandatory gates are failed or unverified. The Windows installed-application workflow has not been executed.

## Gate matrix

| Gate | Requirement | Result | Evidence |
|---|---|---|---|
| 1 — Source and requirements audit | Architecture inspected, gaps documented, baseline tests run | **PASS (partial)** | `Docs/FINAL_ENGINEERING_AUDIT.md`; Rust suite run |
| 2 — Financial correctness | Deterministic results and currency handling match independent expected values | **FAIL** | Totals regression passes; currency exclusion (D3) and chat figure accuracy (D5) still wrong |
| 3 — Data security | Production storage encrypted; isolation, backup and restore verified | **FAIL** | D4: production database is plaintext (file header `SQLite format 3`). Backup/restore not re-verified (D6) |
| 4 — AI reliability | Answers are correct or fail safely; zero unsupported verified claims | **FAIL** | Previous command-layer E2E run: wrong figures, sign and currency errors labelled grounded |
| 5 — Feature integration | Every production UI action connected to a backend operation | **NOT VERIFIED** | GUI not exercised in this pass |
| 6 — Native end-to-end | Installed Windows app completes the documented workflow | **BLOCKED** | No desktop-control tool connected; Application Control blocks some test binaries |
| 7 — Offline release | Full workflow offline, recovery works, security checks pass | **NOT VERIFIED** | Not run |

## Test results in this pass

Rust test binaries run directly from PowerShell, because Cargo's own spawn was blocked for some binaries:

- `saar_lib` unit tests: **76 passed, 0 failed**, including 6 duplicate-payment tests and the all-rows totals regression.
- `p1_native_storage_verification`: 3 passed.
- `p2_ingestion_verification`: 1 passed.
- `p4_local_ai_verification`: 1 passed in this run. Its answer-content check was failing in the previous pass, and this result is not stable evidence until it passes repeatedly.
- `p5_editorial_interface_verification`: 1 passed.
- `p6_reliability_and_backup_verification`: 3 passed.
- `p3_financial_core_verification`: **BLOCKED**. Windows Application Control policy refused to launch the binary. Not executed.
- `e2e_bills` (ignored, command layer): not rebuilt or run in this pass.

Frontend (`pnpm test`): not re-run in this pass.

## Outstanding blockers

1. **Plaintext production database (critical).** Requires owner decision on key management and data migration. Not changed without approval.
2. **Currency exclusion** not reported.
3. **Chat figures not computed in code.** The model still writes the figures.
4. **Backup/restore** paths not corrected and not re-verified.
5. **GUI and installed-application workflow** not run.
6. **Model evaluation** (Qwen3.5-4B on RTX 4060) not performed; hardware not available here.
7. **Windows Application Control** blocks some test binaries in this environment.

## Documents not produced in this pass

The brief also asks for `Docs/FINANCIAL_CORRECTNESS_RESULTS.md`, `Docs/END_TO_END_EXECUTION.md` and `Docs/SECURITY_AND_OFFLINE_VERIFICATION.md`. They are outstanding.

## Honest final statement

This pass fixed two verified financial defects with regression tests and identified a critical storage defect. It did not complete the brief, and the release is not ready.


## Update after the corpus and security work

Verdict is unchanged: **NOT READY**.

| Gate | Status now | Basis |
|---|---|---|
| 2 - Financial correctness | **PARTIAL.** Deterministic extraction, reconciliation, duplicate detection and totals pass against independent ground truth for two workspaces (`Docs/FINANCIAL_CORRECTNESS_RESULTS.md`). Chat figures are still written by the model; currency conversion is not implemented. | 97 lib tests, corpus: 0 mismatches |
| 3 - Data security | **PARTIAL.** Per-workspace encrypted databases, verified migration with rollback, and on-disk checks pass (`Docs/SECURITY_AND_OFFLINE_VERIFICATION.md`). Restore success path untested; decrypted temp files still use the system temp folder. | migration (3) and E2E disk check |
| 4 - AI reliability | **FAIL.** Deterministic answers from code are not implemented; model figures can be labelled grounded while wrong. | earlier E2E answers |
| 5 - Feature integration | **NOT VERIFIED.** The GUI was not exercised. | none |
| 6 - Native end-to-end | **BLOCKED.** No desktop-control tool is available in this environment. | none |
| 7 - Offline release | **NOT VERIFIED.** | none |

Test infrastructure note: Windows Application Control intermittently refuses to start newly built test binaries. Every suite ran after retrying; nothing was bypassed.


## Update after the verified-answer layer and end-to-end restore

Verdict is unchanged: **NOT READY**, because the installed Windows GUI workflow and the offline run have not been executed.

| Gate | Status now | Basis |
|---|---|---|
| 2 - Financial correctness | **PASS for the deterministic engine**; currency conversion is not implemented (other currencies are reported as excluded) | corpus: 0 mismatches over 47 invoices, 10 receipts, 2 statements; answers equal the ledger for every month |
| 3 - Data security | **PASS for storage, migration and backup/restore**; the user's real data was not migrated (none found); decrypted temp files still use the system temp folder; no disk scan of WAL/SHM sidecars | migration (3 tests), E2E disk checks, E2E restore |
| 4 - AI reliability | **PASS for the architecture**: every figure comes from code and is marked verified; model text is always marked unverified; injection is quarantined; the model-missing path is safe. The model's free-form text is still often wrong. | `answers`, `answer_plan`, `financial_ops` tests; E2E chat section |
| 5 - Feature integration | **NOT VERIFIED** | the GUI was not exercised |
| 6 - Native end-to-end | **BLOCKED** | no desktop-control tool; installer not built |
| 7 - Offline release | **NOT VERIFIED** | no network-disconnected run or traffic capture |

Test counts: 108 library tests (including the corpus, dynamic-result and answer tests), integration suites p1-p6, migration (3), golden dataset (2), end-to-end (1), frontend (7), all passing.

Still open: GUI testing, installer build and install test, offline traffic capture, GPU inference and the Qwen3.5-4B evaluation, currency conversion, anomaly noise on recurring large payments, partial-payment linking, and moving decrypted temp files into a private folder.


## Update after driving the real window

Verdict unchanged: **NOT READY**, for the reasons in the last two rows.

| Gate | Status now | Basis |
|---|---|---|
| 2 - Financial correctness | PASS for the deterministic engine; no currency conversion (other currencies reported as excluded, also on the reports screen) | corpus 0 mismatches; answers equal the ledger |
| 3 - Data security | PASS for encrypted storage, migration, backup and restore (restore also from the Vault screen) | migration tests; command-layer and real-window runs |
| 4 - AI reliability | PASS for the architecture: figures from code marked verified; model text always unverified; injection quarantined; model path no longer fails on realistic workspaces | answer tests; real-window chat checks |
| 5 - Feature integration | PASS on the debug build: 61 real-window checks, 0 failures, 11 defects found and fixed | `Docs/evidence/gui_e2e_report.md` |
| 6 - Native end-to-end | PARTIAL: full workflow ran in the real window, not from an installed package | `Docs/END_TO_END_EXECUTION.md` |
| 7 - Offline release | FAIL / OPEN: WebView2 keeps outbound TLS connections despite Chromium flags; no disconnected-network run | `Docs/SECURITY_AND_OFFLINE_VERIFICATION.md` |

Still open: WebView2 outbound connections (owner decision: firewall or runtime policy), installer build and install test, disconnected-network run, GPU inference and Qwen3.5-4B evaluation, currency conversion, anomaly noise on recurring large payments, partial-payment linking, model-request cancel control, keyboard/focus audit, private temp folder for decrypted files.

## Update: Qwen3 4B on GPU and interface redesign

- Model: Qwen3-4B Q4_K_M (2.5 GB, SHA-256 pinned from the official Qwen/Qwen3-4B-GGUF download), replacing Qwen2.5-3B Q3_K_M. Runs with GPU offload on the RTX 4060 (8 GB) with a CPU fallback; reasoning mode is off. Direct `llama-cli` measurement: 54 tokens/s on GPU against 15 on CPU; prompt processing about 10 times faster.
- Free-form answer in the real window: 8 s (45 s before). The text is still labelled Not verified; figures come from code.
- Classification replies are bounded by the grammar (rationale at most 160 characters), because Qwen3 wrote long rationales that were cut off mid-JSON.
- Interface: neutral graphite and off-white themes with one accent; green, red and amber only mean credit, debit and caution. Plain labels instead of jargon; anomaly codes shown as readable names; citations shown as "Transaction 2cc01800". Claims the app could not back ("Air-Gapped", "0 outbound sockets") were removed; the Security page now states that the WebView2 component's own connections are not controlled by SAAR.
- Regression after both changes: 115 library tests, p1-p6, migration (3), golden dataset (2), command-layer end-to-end, frontend (7), real-window run (61 checks, 0 failures).
- Not done: comparison of Qwen3 4B against Qwen3 8B, Phi-4-mini or Gemma 3 on the corpus questions; accuracy of free-form text is unmeasured. Verdict unchanged: NOT READY (installer, offline run, WebView2 traffic).
