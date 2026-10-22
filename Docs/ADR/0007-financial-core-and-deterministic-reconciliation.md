# ADR 0007: Financial Core & Deterministic Reconciliation Engine

## Status
Accepted and Verified (Gate G3).

## Context
Phase 3 establishes the financial core of OpenDeck, implementing strict deterministic money representation, invoice-to-bank statement reconciliation, duplicate and anomaly detection, immutable user review and audit logging, and financial aggregates.
In accordance with `RULES.md`, `SPECS.md`, and `TASKS.md` (P3-01 through P3-05):
1. **Zero Binary Floating-Point Currency Arithmetic:** All monetary amounts across the database, business logic, IPC, and frontend must strictly be represented as signed 64-bit integer minor units (`i64`, e.g. cents, pence, paise, yen). Any attempt to add or reconcile different currencies without explicit conversion must be rejected at runtime.
2. **Deterministic Matching Engine:** Reconciliation between invoices and ledger transactions must be rule-based and explainable. Never hallucinate links or guess balances without transparent confidence scores and evidence chains.
3. **Anomaly & Duplicate Detection with SaaS Recurring Suppression:** Detect duplicate invoice numbers, duplicate transaction billings within short time windows, and high-value unlinked transactions, while recognizing recurring monthly SaaS subscriptions (e.g. GitHub $42/month) to eliminate false-positive duplicate alerts.
4. **Immutable Review Inbox & Audit Trail:** Proposed reconciliation links require explicit user review (Approve, Reject, Correct). Every mutation must be recorded in an immutable, append-only `audit_log` with before/after state captures.
5. **Deterministic Aggregates:** Monthly income, expense, net cash flow, and Month-over-Month (MoM) calculations must match independent manual arithmetic exactly.

## Architecture & Implementation Decisions

### 1. Exact Minor-Unit Money System (`src-tauri/src/finance/money.rs`)
- **Structure:** `Money { amount_minor: i64, currency: String }`.
- **Operations:** Checked addition, subtraction, multiplication with overflow/underflow detection (`checked_add`, `checked_sub`). Mismatched currencies return `MoneyError::CurrencyMismatch`.
- **Formatting:** Deterministic string rendering with zero float division (e.g. `$1,234.56`, `₹118,000.00`, `¥5,000`).
- **Credit Note / Refund Rules:** Applying credit notes reduces net payable deterministically down to zero; over-application returns `RefundExceedsBalance`.

### 2. Multi-Pass Deterministic Reconciliation Matcher (`src-tauri/src/finance/matching.rs`)
- **Pass 1 — Exact Match (Confidence 1.0):**
  Matches when `inv.currency == tx.currency`, `inv.total_minor == abs(tx.amount_minor)`, either the invoice number or vendor tokens appear in the transaction description/reference, and the transaction date is within 45 days of invoice date.
- **Pass 2 — Partial / Fuzzy Reference Match (Confidence 0.85):**
  Matches when currency and amount match exactly, vendor tokens appear in transaction text, and transaction occurs within 60 days.
- **Pass 3 — Split / Multiple Payment Aggregation:**
  Detects when 2 or 3 transactions combine to equal an invoice amount, or multiple invoices match a single consolidated batch payment.
- **Pass 4 — Date Proximity Match (Confidence 0.50):**
  Suggests matches when amount matches and transaction occurs within 10 days of invoice, accompanied by an explicit warning of missing reference tokens.
- **Explainability:** Every proposed link includes an evidence explanation string (e.g. `"Exact match on amount (12000 USD) and reference (invoice number) within 0 days."`).

### 3. Anomaly & Duplicate Detection (`src-tauri/src/finance/anomaly.rs`)
- **Duplicate Invoice Detection:** Flags identical invoice numbers from the same vendor, or duplicate charges from the same vendor on the same amount within 3 days.
- **Recurring SaaS Subscription Recognition:** Charges with identical amounts spaced 25 to 35 days apart (e.g., $42/month SaaS) are classified as expected recurring charges, preventing spurious duplicate billing alerts.
- **High-Value Unmatched Transactions:** Flags unlinked debit transactions exceeding $1,000.00 / 100,000 minor units to ensure large business expenditures are never silently omitted.

### 4. Review Inbox & Immutable Audit Trail (`src-tauri/src/finance/review.rs`)
- **State Machine:**
  - `invoice_payment_links`: `PROPOSED` -> `APPROVED` | `REJECTED`
  - `transactions`: `UNREVIEWED` -> `APPROVED` (with `matched_invoice_id` foreign key)
  - `invoices`: `UNREVIEWED` -> `APPROVED`
- **Transactions & Rollback:** All review state updates occur inside ACID SQLite transactions.
- **Audit Logging:** Every decision writes to the `audit_log` table with timestamp, workspace ID, entity type, action (`APPROVE_LINK`, `REJECT_LINK`, `CORRECT_TRANSACTION`), and full JSON payload capturing prior values and user rationale.

### 5. Deterministic Aggregates (`src-tauri/src/finance/aggregates.rs`)
- Aggregates income (positive minor units), expenses (absolute negative minor units), and net savings per calendar month (`YYYY-MM`).
- Computes percentage change in expenses between consecutive months rounded to one decimal place.
- Prevents cross-currency contamination by isolating aggregates strictly to a target currency.

## Verification & Test Evidence
- **Unit Tests:** 37 Rust unit tests passing (`cargo test --lib`).
- **Frontend Tests:** 4 Vitest tests passing (`pnpm test`).
- **Frontend Production Build:** `tsc && vite build` succeeded without errors.
- **End-to-End Integration Test:** `src-tauri/tests/p3_financial_core_verification.rs` passed in 15.96s:
  - Ingested GST invoice (`gst_b2b_valid.pdf`), scanned thermal receipt (`thermal_receipt_scanned.png`), US bank statement (`bank_us_sample.csv`), and HDFC bank statement (`bank_hdfc_sample.csv`).
  - Proved exact match: CloudHosting Corp `INV-88912` matched transaction debit `-120.00 USD` (12000 minor) with confidence 1.0.
  - Proved GST invoice match: ACME Tech Services `ACME-2026-0915` matched HDFC transaction `-118,000.00 INR` (11800000 minor).
  - Proved anomaly detection: Detected large unmatched business debits and duplicate invoices.
  - Proved review inbox: Approved link transitioned transaction and invoice states to `APPROVED`, updated `matched_invoice_id`, and recorded immutable entry in `audit_log`.
  - Proved transaction correction: Updated transaction description and verified before/after audit log.
  - Proved aggregate hand-calculation: Calculated income, expenses, and net savings matching independent manual arithmetic without 1 minor unit of discrepancy.
- **Visual Desktop UI Verification:** Tauri application launched on Debian desktop with full Reconciliation Review Inbox and Financial Reports tabs ([`tests/opendeck_debian_p3_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p3_verification.png)).
