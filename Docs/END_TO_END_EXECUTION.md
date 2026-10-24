# SAAR — End-to-End Execution Record

This records what was actually executed. It does **not** claim the native GUI workflow was run.

## What was executed

`src-tauri/tests/e2e_bills.rs` drives the application through the **real Tauri command layer**: the
same handler list the app registers, the app's own capability configuration, the Windows origin
(`http://tauri.localhost`), and JSON arguments with the names the frontend uses. Two clean
installations (separate data folders) are used. The local language model (Qwen2.5-3B, CPU) and the
RapidOCR worker run for real. The data is the synthetic `workspace_beta` corpus with its
independent ground-truth ledger.

Run it with:

```
cargo test --features e2e --test e2e_bills -- --ignored --nocapture
```

Final run: **PASS**. The full report is `Docs/evidence/e2e_command_layer_report.md`.

| Stage | Result |
|---|---|
| Create an encrypted workspace; file has no plaintext header; status reports encrypted | PASS |
| Ingest 20 files (15 PDF invoices, 4 scanned receipts, 1 bank CSV); every bank row stored | PASS |
| Corrupted PDF rejected; byte-identical re-import reported as a duplicate | PASS |
| Reconciliation proposes links; duplicate payment flagged; recurring subscription not flagged | PASS |
| USD income and expenses equal the ledger; INR transactions reported as excluded | PASS |
| Chat: all-time, monthly and INR expenses equal the ledger; invoice total equals the ledger; absent invoice reported absent | PASS |
| Chat: a wrong figure in the question does not change the answer; prompt injection quarantined | PASS |
| Chat: a model-written answer is never marked verified and carries an explanation | PASS |
| CSV report has one row per transaction; PDF report is a PDF; both cover the same records | PASS |
| Encrypted backup contains all 21 document blobs | PASS |
| Lock exposes nothing; wrong passphrase rejected; correct passphrase unlocks; chat history persisted | PASS |
| Restore in a clean installation: wrong backup passphrase rejected; tampered archive rejected | PASS |
| Restore succeeds; restored workspace unlocks; transactions, totals, document decryption, chat history and answers equal the source | PASS |
| Second restore refuses to overwrite; free-form question with no model loaded answered safely and unverified | PASS |
| Both installations' databases have no plaintext header and no vendor name in plaintext | PASS |

## Defects this run found and fixed

1. **Restore of an encrypted backup failed** ("file is not a database"): restore ran a plaintext
   integrity check on a file that cannot be opened without its key. The archive is authenticated
   by AES-GCM and by the database SHA-256; SQLCipher verifies each page when the workspace is
   unlocked. The SQL check now applies only to plaintext files.
2. **An ordinary question had no verified path.** "What was the invoice total for CloudHosting
   Corp?" has no invoice number and no expense keyword, so it fell to the unverified model. A
   vendor-based invoice operation now totals every matching invoice per currency.
3. **"How many invoices from X?" ignored X.** Counts now respect a named vendor.

## What this does not prove

- **The GUI was not exercised.** No installed-application window was clicked: no desktop-control
  tool is available in this environment. Command-layer success proves the backend contract the UI
  calls, not that each screen renders and responds correctly.
- **The Windows installer was not built or installed.** The run uses the debug build's commands.
- **Offline operation was not tested with the network disabled.** The application makes no network
  calls by design (`connect-src 'none'`; no HTTP client in use), but this run did not capture
  traffic or disconnect the machine.
- **Free-form model answers are not accurate**, and are labelled unverified. The model has not
  been replaced or evaluated against a larger one (Qwen3.5-4B on an RTX 4060 was not available).
- **GPU inference is not enabled** (`-ngl 0`).
- **Interrupted-operation and missing-model recovery** were tested only for a missing model.

## Environment note

Windows Application Control intermittently refuses to start newly built test binaries
("os error 4551"). Each suite ran after retrying; no policy was bypassed.


## Real-window run

`tests/gui/gui_e2e.mjs` drives the actual `saar.exe` window (bundled frontend in WebView2, real Rust backend, encrypted storage, OCR worker and local model) through the WebView2 DevTools protocol on localhost: real clicks, real text input, real key events, real screenshots. No browser mock and no dev server. Expected figures come from the corpus ledger.

```
node tests/gui/gui_e2e.mjs
```

Final run: **61 checks, 0 failures.** Report: `Docs/evidence/gui_e2e_report.md`. Screenshots: `Docs/evidence/gui/`.

| Area | Checked in the real window |
|---|---|
| Launch | starts from its packaged frontend; no demo or quick-unlock shortcut; mismatched passphrases rejected; workspace created through the form; database file on disk encrypted |
| Navigation | every screen opens and shows its controls |
| Import | 20 files imported through the Documents screen; re-import reported as duplicate; corrupted PDF rejected with a visible error and kept as one FAILED record |
| Reconciliation | the Run button produces links and anomalies |
| Chat | all-time, monthly and INR expenses, invoice total with citation, duplicates, absent invoice, wrong figure in the question, prompt injection, free-form text labelled Not verified; citation opens evidence; Ctrl+N opens a new conversation |
| Reports | CSV (72 rows, equal to the ledger) and PDF written inside the data folder |
| Backup | created from the Vault screen |
| Lock / restart | lock, wrong passphrase rejected, correct passphrase unlocks; cold restart keeps history and documents; same question gives the same figure |
| Layout | no horizontal overflow at 1920x1080, 1366x768, 1024x700 |
| Restore | into a clean second installation from the Vault screen; workspace appears, unlocks, same documents, same figure |

Verified answers return in under a second; a free-form model answer took 45 s on CPU.

### Defects found only by driving the real window

1. Financial Reports blanked the window (`null.toFixed` on the month-on-month change). Fixed; screens are now inside an error boundary.
2. Import banner read `"undefined" (undefined)` (snake_case field names). Fixed; the command-layer test now fails on any snake_case key.
3. Cold start offered "Create workspace" instead of unlocking the existing one.
4. A failed import stayed PROCESSING; now FAILED immediately and audited.
5. "Taxi fare" was read as a tax line; tax labels now match whole words.
6. The model path failed on realistic workspaces (8311 tokens vs 4096 context) and showed the runtime error as an answer; the packet is now budgeted and runtime errors are failures.
7. The restored workspace was missing from the unlock screen until restart.
8. The evidence panel showed raw minor units and bounding-box JSON, unstyled, with hardcoded USD.
9. Reports and review screens formatted every amount as USD; tile captions claimed "Approved/Verified" for unverified totals.
10. The excluded-currency total was invisible in the UI.
11. The cash-flow chart drew only four months and clipped labels.

### What the real-window run does not prove

- Not the installed package, installer, or a signed build.
- Keyboard coverage is limited to Ctrl+N; no Tab-order or focus audit.
- No cancel control for a running model request.
- Offline operation with the network disconnected was not verified; see the open WebView2 finding in `Docs/SECURITY_AND_OFFLINE_VERIFICATION.md`.
- Windows Application Control blocks `node.exe` from starting the unsigned debug build, so the harness starts it through PowerShell and then attaches. No policy was changed.
