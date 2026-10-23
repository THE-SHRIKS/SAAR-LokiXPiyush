# SAAR end-to-end run (command layer, workspace_beta corpus)

## 1. Workspace creation and encryption at rest

- PASS: workspace created and unlocked
- PASS: database file has no plaintext SQLite header
- PASS: status reports an encrypted database

## 2. Ingestion

- PASS: all 20 files ingested ([])
- PASS: 20 documents stored
- PASS: every bank row stored
- PASS: corrupted PDF is rejected
- PASS: re-importing the same file is reported as a duplicate

## 3. Reconciliation and anomalies

- PASS: invoice-to-payment links proposed
- PASS: a duplicate payment is flagged
- PASS: recurring subscription is not flagged

## 4. Financial summary

- PASS: USD expenses equal the ledger
- PASS: USD income equals the ledger
- PASS: INR transactions are reported as excluded, not dropped silently

## 5. Chat

- PASS: all-time expenses = 124,285.28 USD
- PASS: April 2025 expenses = 79,922.81 USD
- PASS: INR expenses 2025 = 113,542.93 INR
- PASS: invoice "RPW-2025-0004" total = 9,123.76 INR
- PASS: an invoice that does not exist is reported as absent
- PASS: duplicate payments reported
- PASS: a wrong figure in the question does not change the answer
- PASS: prompt injection is quarantined
- PASS: a model-written answer is never marked verified
- PASS: the unverified answer carries an explanation

## 6. Reports

- PASS: CSV has 72 data rows
- PASS: PDF export is a PDF file
- PASS: PDF and CSV cover the same records

## 7. Backup, lock and unlock

- PASS: backup contains 21 document blobs
- PASS: locked workspace exposes no transactions
- PASS: wrong passphrase rejected
- PASS: correct passphrase unlocks
- PASS: chat history persisted: 9 questions and 9 answers (18 messages)

## 8. Restore into a clean installation

- PASS: restore rejects a wrong backup passphrase
- PASS: restore succeeded: Ok(String("18ad3d2d-1fea-4349-84ca-a44c32704fa2"))
- PASS: restored workspace is listed
- PASS: restored workspace rejects a wrong passphrase
- PASS: restored workspace unlocks
- PASS: restored transaction count equals the source
- PASS: restored totals equal the source
- PASS: a restored document decrypts and previews
- PASS: restored chat history is present
- PASS: the same question gives the same figure after restore
- PASS: with no model loaded, a free-form question is answered safely and unverified
- PASS: a second restore refuses to overwrite
- PASS: a tampered archive is rejected

## 9. Disk check

- PASS: 18ad3d2d-1fea-4349-84ca-a44c32704fa2.saardb has no plaintext header
- PASS: no vendor name appears in plaintext
- PASS: 18ad3d2d-1fea-4349-84ca-a44c32704fa2.saardb has no plaintext header
- PASS: no vendor name appears in plaintext
- PASS: 2 encrypted databases inspected
