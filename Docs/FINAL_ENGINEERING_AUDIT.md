# SAAR — Final Engineering Audit (this pass)

Scope: Phase 0 (baseline and reproduction) and the financial-correctness slice of Phase 1.
Status of the wider brief: **not complete**. See `Docs/RELEASE_READINESS.md`.

## Baseline

- Branch: `fix/financial-correctness`, created from `chore/e2e-chat-fixes` (commit `070cb98`, pushed earlier).
- Platform: Windows 11, PowerShell 5.1 and Git Bash, Rust toolchain via rustup, Node/pnpm.
- Hardware: development machine only. The RTX 4060 target hardware was **not** available in this environment.

## Reproduced defects

| # | Defect | Reproduction | Root cause | Status |
|---|---|---|---|---|
| D1 | Chat totals cover only the latest 50 transactions | Reading `build_fact_packet` | `LIMIT 50` used to compute income/expense totals | **Fixed**, regression test `scoped_total_tests` |
| D2 | Two distinct 742.81 debits, 12 days apart, not flagged | Reading `detect_anomalies` | The only transaction rule required identical date and description | **Fixed**, tests in `potential_duplicate_tests` |
| D3 | Other-currency transactions silently dropped from summaries | Reading `compute_summary` | `continue` on currency mismatch with no report | **Not fixed** (see Remaining risks) |
| D4 | Production database stored as plaintext SQLite | Reading the file header of `%LOCALAPPDATA%\theshriks\saar\data\saar_workspace.db` | `Database::open` never sets `PRAGMA key`; only tests do | **Not fixed**, requires owner approval (migration of existing data) |
| D5 | Chat answers often wrong (figures, signs, currency) | Command-layer E2E run from the previous pass | 3B model asked to compute financial results | **Not fixed**; architectural change pending |
| D6 | Backup/restore use hardcoded production paths | Reading `create_encrypted_backup` | Paths reconstructed per command | **Not fixed** |

## Corrective changes in this pass

- `src-tauri/src/finance/anomaly.rs`
  - New `detect_potential_duplicate_payments`: flags two debits in the same currency, with the same amount, the same normalized merchant (first alphanumeric word of the description), and posting dates 1–14 days apart. Pairs 25–35 days apart (monthly billing) are excluded. Refunds (credits) are ignored. Nothing is deleted or altered; the anomaly asks for review.
  - Six unit tests: the 742.81 regression, monthly subscription, different merchant, different amount, refund, and outside the review window.
- `src-tauri/src/ai/grounding.rs`
  - New `scoped_totals`: income, expense and net totals computed in SQL over all rows in the workspace and base currency.
  - Fact-packet totals now come from `scoped_totals`. The 50-row transaction list remains as evidence only.
  - Regression test `test_totals_cover_all_transactions_beyond_the_evidence_limit`: 241 transactions, expected totals computed independently in the test.
- `src-tauri/Cargo.toml`
  - The Tauri mock-runtime feature moved behind an opt-in `e2e` feature, so unit-test binaries no longer depend on it.
  - The `saar` binary has `test = false`; it is a launcher with no unit tests.
- `src-tauri/build.rs`
  - The test manifest is applied only when the `e2e` feature is enabled.

## Remaining risks

1. **Plaintext production database (D4), critical.** Financial records and chat history are readable by anyone with file access. A fix requires choosing a key-management design and migrating existing data, which needs owner approval.
2. **Currency exclusion (D3).** Summaries still silently ignore non-base currencies. Required behaviour: report excluded counts and totals per currency.
3. **Chat accuracy (D5).** The model still produces wrong figures. Required behaviour: deterministic figures from Rust, with the model only phrasing them.
4. **Test-runner environment.** Windows Application Control blocked at least one freshly built test binary (`p3_financial_core_verification`). That suite was not executed in this pass.
5. **Duplicate rule scope.** The new rule is limited to exact-amount, same-normalized-merchant pairs. It will miss near-equal amounts and merchants that normalize differently.
