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
