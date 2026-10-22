# ADR 0008: Local AI Runtimes, Grounded Fact Packets, GBNF Grammar Classification, and SQLite FTS5 Search

## Status
Accepted and Verified (Gate G4).

## Context
Phase 4 implements the offline AI capabilities of OpenDeck in accordance with `RULES.md`, `SPECS.md`, and `TASKS.md` (P4-01 through P4-05):
1. **Real Offline Local LLM Runtime (Zero Cloud / Zero API Mocks):** Execute `models/weights/Qwen2.5-3B-Instruct-Q3_K_M.gguf` via pinned `bin/llama/llama-cli` on the dual-core Intel Core i3-7020U AVX2 CPU. Verify SHA-256 checksum against `models/model-manifest.json` (`8eff4e4ba10be3060a4f6cf1523aa786c52a32ec42ea3b2bc777411648a4c8a5`) at startup.
2. **Deterministic Bounded Fact Packets & Citations:** The local language model must never query raw databases or make unconstrained assertions. The Rust core compiles structured `FactPacket`s from SQLite records and enforces citation tags (`[inv:ID]`, `[tx:ID]`, `[doc:ID]`) with zero tolerance for floating-point hallucinations.
3. **GBNF Grammar-Constrained Expense Classification:** Transaction categorization must be deterministic and guaranteed to conform to the 7 allowed business expense categories using llama.cpp GBNF formal grammar constraints.
4. **SQLite FTS5 Full-Text Search:** Sub-millisecond keyword and entity search indexing documents and transactions with BM25 ranking and highlighted snippet generation.
5. **Adversarial Prompt Defense & Numerical Hallucination Guard:** Intercept prompt injection attacks before reaching the model, and validate any numbers emitted by the model against authorized minor-unit figures in the fact packet.

## Architecture & Implementation Decisions

### 1. Local Model Runtime Engine (`src-tauri/src/ai/llm_engine.rs`)
- **Execution Subprocess:** Wraps `bin/llama/llama-cli` with `-ngl 0` (AVX2 CPU fallback), `-t 2` (pinned to 2 hardware threads), `-c 2048` context window, and `--temp 0.0` (strict greedy decoding for deterministic output).
- **Concurrency & Deadlock Prevention:** Avoids OS pipe buffer deadlocks on extensive stderr logging by running `cmd.output()` inside dedicated execution threads with configurable timeouts (300s default for multi-turn CPU reasoning).
- **Model Checksum Verification:** Verifies SHA-256 integrity of the 1.59 GiB GGUF weights, accelerated using compiler opt-level 3 in dev profiles (`~8s` calculation).
- **Banner Stripping:** Strips llama-cli progress markers (`[ Prompt: ... | Generation: ... ]`) and isolates JSON tokens.

### 2. Grounded Fact Packets & Provenance Verification (`src-tauri/src/ai/grounding.rs`)
- **Intent Classifier:** Detects query domain (`FINANCIAL_SUMMARY`, `DOCUMENT_QUERY`, `RECONCILIATION_STATUS`, `TRANSACTION_SEARCH`, `GENERAL_FINANCIAL`).
- **Structured Fact Extraction:** Queries active workspace SQLite database for relevant invoices, ledger transactions, anomalies, and exact integer minor-unit sums (`total_income_minor`, `total_expense_minor`, `net_savings_minor`).
- **Citation Extraction:** Parses emitted citation brackets (`[inv:ID]`, `[tx:ID]`, `[doc:ID]`) and verifies that cited identifiers exist in the input fact packet.
- **Provenance Verification:** Flags responses as `grounded: true` only when all citations resolve and numerical amounts match authorized figures.

### 3. GBNF Formal Grammar Classifier (`src-tauri/src/ai/classifier.rs`)
- **Restricted Output Grammar:** Constrains model sampling to valid JSON matching schema:
  `{"category": "...", "confidence": 0.xx, "rationale": "..."}`
- **Allowed Categories:** Strictly restricted to:
  `Software & Subscriptions`, `Hosting & Cloud Infrastructure`, `Office Supplies & Stationery`, `Professional Services & Advisory`, `Travel & Transportation`, `Equipment & Hardware`, `Other Expense`.
- **Decimal Termination:** Bounded to at most 4 digits (`[0-1] ("." [0-9]{1,4})?`) to prevent infinite token generation loops on CPU.
- **Audit Integration:** Proposals are stored in `transaction_categories` and written to `audit_log` upon explicit user approval.

### 4. SQLite FTS5 Full-Text Search Engine (`src-tauri/src/ai/search.rs`)
- **Virtual Table Schema:** `search_index USING fts5(entity_type, entity_id, title, content, vendor, description, tokenize='porter unicode61')`.
- **Automatic Ingestion Indexing:** Integrated into `DocumentOrchestrator::ingest_file` to index digital text and OCR-extracted fields immediately upon ingestion.
- **Search Execution:** Queries matching terms with SQLite FTS5 `snippet(search_index, -1, '<mark>', '</mark>', '...', 16)` and BM25 rank ordering.

### 5. Adversarial Defense & Hallucination Guard (`src-tauri/src/ai/defense.rs`)
- **Injection Pattern Interceptor:** Catches jailbreak prompts ("ignore previous instructions", "system prompt override", "act as", "forget rules", markdown fence breakouts) and returns sanitized quarantine responses without invoking the model.
- **Numerical Hallucination Guard:** Parses all currency figures (`$X.YY`, `X.YY`) in model output, converts to integer minor units, and asserts that every generated monetary number exists in the authorized set of fact packet amounts.

## Verification & Test Evidence
- **All Rust Lib Tests:** 48 unit tests passing (`cargo test --lib`).
- **Frontend Test Suite:** 4 Vitest unit tests passing (`pnpm test`).
- **Frontend Production Build:** Vite + TypeScript production bundle built in 5.39s with 0 errors.
- **Phase 4 Full Integration Test (`src-tauri/tests/p4_local_ai_verification.rs`):** Passed in 172.07s on dual-core CPU:
  - Step 1: Verified model weight SHA-256 integrity matches manifest exactly.
  - Step 2: Ingested documents into encrypted workspace.
  - Step 3: Verified SQLite FTS5 search indexing and sub-millisecond retrieval with `<mark>` snippets.
  - Step 4: Verified GBNF-constrained classification categorized Office Supplies Mart transaction into `"Office Supplies & Stationery"`.
  - Step 5: Verified grounded Q&A correctly answered CloudHosting Corp invoice total ($120.00 / 12000 minor units) with citations and zero hallucinations.
  - Step 6: Verified prompt injection detection intercepted malicious prompts, and hallucination validator rejected fictional `$9,999.00` expenditure.
- **Desktop GUI Verification:** OpenDeck desktop window launched and rendered on Debian `DISPLAY=:0.0` with full Grounded AI Chat, FTS5 Search bar, and Category Classification controls ([`tests/opendeck_debian_p4_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p4_verification.png)).
