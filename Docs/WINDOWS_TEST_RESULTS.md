# SAAR Windows Test Execution Results

**Environment:** Windows 10 / Windows 11 x64 (MSVC 14.44)  
**Binary Executed:** `src-tauri\target\debug\deps\saar_lib-7b22531e4f33b2d2.exe`  
**Total Tests:** 61  
**Passed:** 61  
**Failed:** 0  
**Execution Time:** 41.91 seconds  
**Execution Timestamp:** 2026-10-22 21:41:48 UTC  

---

## 1. Complete Test Suite Output

```text
running 61 tests
test ai::classifier::tests::test_allowed_categories_list ... ok
test ai::defense::tests::test_prompt_injection_detection ... ok
test ai::classifier::tests::test_parse_valid_json_classification ... ok
test ai::defense::tests::test_sanitize_untrusted_text ... ok
test ai::llm_engine::tests::test_clean_llm_output ... ok
test ai::grounding::tests::test_intent_parsing ... ok
test ai::defense::tests::test_validate_grounded_numbers_rejects_hallucination ... ok
test ai::defense::tests::test_validate_grounded_numbers_accepts_valid ... ok
test ai::llm_engine::tests::test_llm_config_paths_resolve ... ok
test finance::aggregates::tests::test_aggregates_independent_hand_calculation ... ok
test finance::anomaly::tests::test_recurring_monthly_subscription_suppresses_false_positive ... ok
test finance::anomaly::tests::test_duplicate_invoice_number_detected ... ok
test finance::matching::tests::test_exact_match_rule ... ok
test finance::matching::tests::test_mismatch_explanation_rule ... ok
test finance::money::tests::test_jpy_zero_decimal_formatting ... ok
test finance::money::tests::test_money_addition_same_currency ... ok
test finance::money::tests::test_money_currency_mismatch_rejected ... ok
test finance::money::tests::test_money_overflow_prevention ... ok
test finance::money::tests::test_money_refund_application ... ok
test ingestion::csv_mapper::tests::test_normalize_date ... ok
test ingestion::csv_mapper::tests::test_parse_amount_minor_exact ... ok
test ingestion::csv_mapper::tests::test_parse_us_bank_csv_fixture ... ok
test ingestion::csv_mapper::tests::test_parse_hdfc_bank_csv_fixture ... ok
test integration_tests::p1::test_p1_05_offline_configuration_and_csp_audit ... ok
test db::migrations::tests::test_fts5_virtual_table_support ... ok
test ingestion::csv_mapper::tests::test_csv_bomb_rejection ... ok
test db::migrations::tests::test_schema_migration_idempotent ... ok
test db::tests::test_transaction_rollback_on_error ... ok
test db::tests::test_audit_log_records_creation ... ok
test db::migrations::tests::test_foreign_key_constraints_enforced ... ok
test finance::review::tests::test_approve_link_updates_state_and_audit ... ok
test ai::search::tests::test_fts5_indexing_and_search ... ok
test security::blob_store::tests::test_deduplication ... ok
test security::blob_store::tests::test_store_and_read_blob_roundtrip ... ok
test security::blob_store::tests::test_tampering_detection_rejects_modified_ciphertext ... ok
test integration_tests::p1::test_p1_04_blob_store_ciphertext_tampering_and_dedupe ... ok
test security::kdf::tests::test_empty_passphrase_rejected ... ok
test security::blob_store::tests::test_wrong_key_fails_decryption ... ok
test security::scrubber::tests::test_scrubs_passphrase_and_keys ... ok
test security::blob_store::tests::test_temp_plaintext_guard_cleans_up_on_drop ... ok
test db::tests::test_sqlcipher_encryption_verified ... ok
test security::scrubber::tests::test_scrubs_personal_paths ... ok
test security::scrubber::tests::test_scrubs_pii_and_cards ... ok
test supervisor::tests::test_loopback_port_allocation ... ok
test supervisor::tests::test_supervisor_allows_local_args ... ok
test supervisor::tests::test_supervisor_rejects_network_args ... ok
test supervisor::tests::test_child_guard_kills_on_drop ... ok
test supervisor::tests::test_child_guard_timeout ... ok
test integration_tests::p6::test_p6_01_source_traceable_csv_and_pdf_exports ... ok
test integration_tests::p6::test_p6_03_log_scrubber_and_system_recovery ... ok
test security::kdf::tests::test_argon2id_derivation_deterministic ... ok
test security::kdf::tests::test_wrong_passphrase_rejection ... ok
test integration_tests::p1::test_p1_02_workspace_key_lifecycle_and_disk_inspection ... ok
test ingestion::normalizer::tests::test_normalize_gst_invoice ... ok
test ingestion::ocr_client::tests::test_extract_digital_pdf_fixture ... ok
test integration_tests::p6::test_p6_02_encrypted_backup_wrong_passphrase_and_restore ... ok
test ingestion::ocr_client::tests::test_corrupted_pdf_returns_recoverable_error ... ok
test ingestion::normalizer::tests::test_normalize_thermal_receipt ... ok
test integration_tests::p5::test_p5_editorial_interface_and_integrated_workflows ... ok
test integration_tests::p3::test_p3_full_financial_core_and_reconciliation_verification ... ok
test integration_tests::p2::test_p2_full_document_and_statement_ingestion_pipeline ... ok

test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 41.91s
```

---

## 2. Test Suite Breakdown by Category

### A. Cryptographic Security & At-Rest Storage (10 tests)
1. `db::tests::test_sqlcipher_encryption_verified` — Proves raw database files are unreadable, zero plaintext headers, HMAC-SHA512 failure on wrong key, correct decryption on verified key.
2. `security::kdf::tests::test_argon2id_derivation_deterministic` — Argon2id key derivation determinism.
3. `security::kdf::tests::test_wrong_passphrase_rejection` — Immediate verification token mismatch on wrong passphrase.
4. `security::kdf::tests::test_empty_passphrase_rejected` — Empty passphrase denial.
5. `security::blob_store::tests::test_store_and_read_blob_roundtrip` — AES-256-GCM roundtrip.
6. `security::blob_store::tests::test_deduplication` — Content-addressed SHA-256 deduplication.
7. `security::blob_store::tests::test_wrong_key_fails_decryption` — Wrong master key rejection.
8. `security::blob_store::tests::test_tampering_detection_rejects_modified_ciphertext` — Bit-flip authentication tag rejection.
9. `security::blob_store::tests::test_temp_plaintext_guard_cleans_up_on_drop` — RAII file deletion and zeroization.
10. `integration_tests::p1::test_p1_02_workspace_key_lifecycle_and_disk_inspection` — Real disk inspection confirming master passphrase does not leak to SQLite file.

### B. Ingestion, OCR & Extraction (9 tests)
1. `ingestion::normalizer::tests::test_normalize_gst_invoice` — Digital B2B GST tax invoice normalization.
2. `ingestion::normalizer::tests::test_normalize_thermal_receipt` — RapidOCR scanned thermal receipt normalization.
3. `ingestion::ocr_client::tests::test_extract_digital_pdf_fixture` — Fast digital text extraction.
4. `ingestion::ocr_client::tests::test_corrupted_pdf_returns_recoverable_error` — Corrupted PDF rejection without panic.
5. `ingestion::csv_mapper::tests::test_normalize_date` — Date format parsing (ISO, US, Indian).
6. `ingestion::csv_mapper::tests::test_parse_amount_minor_exact` — Minor-unit currency parsing.
7. `ingestion::csv_mapper::tests::test_parse_us_bank_csv_fixture` — US format bank statement ingestion.
8. `ingestion::csv_mapper::tests::test_parse_hdfc_bank_csv_fixture` — HDFC Indian format bank statement ingestion.
9. `integration_tests::p2::test_p2_full_document_and_statement_ingestion_pipeline` — End-to-end ingestion pipeline with coordinate bounding box provenance.

### C. Financial Core & Deterministic Arithmetic (10 tests)
1. `finance::money::tests::test_money_addition_same_currency` — Exact integer addition.
2. `finance::money::tests::test_money_currency_mismatch_rejected` — Cross-currency addition prevention.
3. `finance::money::tests::test_money_overflow_prevention` — Signed 64-bit integer overflow protection.
4. `finance::money::tests::test_money_refund_application` — Negative amount refund balance handling.
5. `finance::money::tests::test_jpy_zero_decimal_formatting` — JPY zero-decimal formatting.
6. `finance::aggregates::tests::test_aggregates_independent_hand_calculation` — Net savings = total income - total expense.
7. `finance::matching::tests::test_exact_match_rule` — Exact amount and reference reconciliation.
8. `finance::matching::tests::test_mismatch_explanation_rule` — Tolerance mismatch explanation.
9. `finance::anomaly::tests::test_duplicate_invoice_number_detected` — Duplicate invoice number within window.
10. `finance::anomaly::tests::test_recurring_monthly_subscription_suppresses_false_positive` — Periodic subscription false-positive suppression.

### D. Local AI, Grounding & Defense (8 tests)
1. `ai::classifier::tests::test_allowed_categories_list` — Static taxonomy verification.
2. `ai::classifier::tests::test_parse_valid_json_classification` — Structured classification extraction.
3. `ai::defense::tests::test_prompt_injection_detection` — System prompt override interception.
4. `ai::defense::tests::test_sanitize_untrusted_text` — Control character and injection sanitization.
5. `ai::defense::tests::test_validate_grounded_numbers_accepts_valid` — Fact Packet number verification.
6. `ai::defense::tests::test_validate_grounded_numbers_rejects_hallucination` — Hallucinated figure rejection.
7. `ai::grounding::tests::test_intent_parsing` — Financial intent extraction.
8. `ai::search::tests::test_fts5_indexing_and_search` — SQLite FTS5 local full-text search.

### E. Database Resilience, Auditing & Migrations (5 tests)
1. `db::migrations::tests::test_schema_migration_idempotent` — Idempotent migration replay.
2. `db::migrations::tests::test_fts5_virtual_table_support` — FTS5 virtual table support.
3. `db::migrations::tests::test_foreign_key_constraints_enforced` — Foreign key constraint enforcement.
4. `db::tests::test_transaction_rollback_on_error` — Complete transaction rollback on error.
5. `db::tests::test_audit_log_records_creation` — Immutable audit log recording on entity mutation.

### F. Process Isolation, Recovery & Backups (9 tests)
1. `supervisor::tests::test_child_guard_kills_on_drop` — Supervised process termination on guard drop.
2. `supervisor::tests::test_child_guard_timeout` — Bounded execution timeout.
3. `supervisor::tests::test_supervisor_allows_local_args` — Local argument validation.
4. `supervisor::tests::test_supervisor_rejects_network_args` — Remote URL argument blocking.
5. `supervisor::tests::test_loopback_port_allocation` — Loopback-only (`127.0.0.1:0`) port binding.
6. `security::scrubber::tests::test_scrubs_personal_paths` — Windows user path redaction.
7. `security::scrubber::tests::test_scrubs_passphrase_and_keys` — Passphrase and API key redaction.
8. `security::scrubber::tests::test_scrubs_pii_and_cards` — Credit card, PAN, Aadhaar redaction.
9. `integration_tests::p6::test_p6_03_log_scrubber_and_system_recovery` — Interrupted document status recovery to FAILED.

### G. End-to-End Workflows & Exports (10 tests)
1. `integration_tests::p1::test_p1_04_blob_store_ciphertext_tampering_and_dedupe`
2. `integration_tests::p1::test_p1_05_offline_configuration_and_csp_audit`
3. `integration_tests::p3::test_p3_full_financial_core_and_reconciliation_verification`
4. `integration_tests::p5::test_p5_editorial_interface_and_integrated_workflows`
5. `integration_tests::p6::test_p6_01_source_traceable_csv_and_pdf_exports`
6. `integration_tests::p6::test_p6_02_encrypted_backup_wrong_passphrase_and_restore`
7. `finance::review::tests::test_approve_link_updates_state_and_audit`
8. `ai::llm_engine::tests::test_clean_llm_output`
9. `ai::llm_engine::tests::test_llm_config_paths_resolve`
10. `ingestion::csv_mapper::tests::test_csv_bomb_rejection`

---

## 3. Frontend Verification Results

```text
 ✓ src/test/types.test.ts (4 tests) 41ms

 Test Files  1 passed (1)
      Tests  4 passed (4)
   Start at  21:32:47
   Duration  449ms

vite v6.4.5 building for production...
✓ 1601 modules transformed.
dist/index.html                   0.43 kB │ gzip:  0.30 kB
dist/assets/index-Bg7ZH9ap.css   42.93 kB │ gzip:  6.75 kB
dist/assets/index-iXoO12t5.js   235.46 kB │ gzip: 69.83 kB
✓ built in 1.29s
```
