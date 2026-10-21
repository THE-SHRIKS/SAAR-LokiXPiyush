# Product Requirements Document

## 0. Product boundary
**Working title:** Local Finance (temporary)  
**Category:** Offline, chat-first desktop financial operations workspace  
**Primary target:** Windows 10/11 x64, with Debian as development/test host  
**No cloud at runtime:** no user account, browser-based remote UI, hosted database, API key, cloud inference, external telemetry, silent updates, or automatic downloads.

This product is **not** a chartered-accountant replacement, GST filing platform, tax-advice engine, bank-integration service, or legally certified bookkeeping firm. It handles document organization, transaction review/reconciliation, evidence-backed queries, and operational reports. Final accounting treatment and regulated filing/sign-off remain with the user/qualified professional.

## 1. User and outcome
**Primary user:** solo developer, indie SaaS founder, freelancer or small founder-led team with locally available invoice/expense/bank documents and no appetite for uploading sensitive records to AI services.

**Job to be done:** “I want to drop in my finance files, understand what happened, fix exceptions, and export a trustworthy summary without giving a cloud product access to my records.”

**Core success:** a first-time user can complete an end-to-end monthly review of their own eligible documents entirely offline, locate the evidence for reported numbers, and edit/approve mistakes.

## 2. Core experience
The app opens to a **conversation workspace**, not analytics cards. A compact workspace rail offers Import, Documents, Review, Reports, and Settings as destinations. The large central area contains a natural-language conversation with explicit actions and source citations. A contextual evidence viewer opens on selection (document preview + highlighted page/field + related transactions). Workflows remain usable through visible controls for people who don't use chat.

### First-run
1. Open installed app with network disconnected.
2. See genuine local model readiness: `ready`, `missing model`, `loading`, or `error`, with actionable offline installation instructions. No fictional demo response.
3. Create encrypted local workspace with explicit passphrase and recovery warning.
4. Import eligible local files and view actual processing progress and failures.

### Import and process
1. Select one or more **local** PDF/image or CSV files (e.g., invoices, receipts, statements); import explicitly.
2. Preserve source file, checksum, import time, mime/signature, and provenance under encrypted local storage. Enforce size/page/format limits and explain unsupported inputs.
3. Prefer text-layer extraction for born-digital PDFs; OCR image pages and image files with local model; parse CSV via strict deterministic parser and bank-format mapping UI.
4. Preview structured fields (merchant, document date, invoice ID, currency, line items/taxes when present, total, confidence/provenance).
5. Local LLM may **suggest** expense category and produce explanations; calculations and matching are deterministic.
6. User reviews uncertain or conflicting items. Nothing material is silently posted as approved.

### Grounded conversational workflow
Example queries supported **only against real local records**:
- “How much did we spend on software in September?”
- “Show the invoices without matching payments.”
- “Why did expenses rise compared with last month?”
- “Find duplicate invoice candidates.”
- “Prepare a monthly expense and reconciliation report.”

Answer path: identify permitted intent → query local approved records → compute results deterministically → generate optional human-readable explanation referencing *actual* records → render actionable citations. If data is missing, say so. Do not hallucinate expenses, balances, dates, customers, source evidence, or completed actions.

### Review and export
- Separate uncertain AI suggestions from user-approved facts.
- Review/correct amounts, date, vendor, categories, invoice-payment links; every edit is audited locally.
- Duplicate invoice candidates based on hashes/normalized supplier-invoice-amount data; allow false-positive resolution.
- Missing-payment detection based on explicit status and matching rules; display “unmatched/unknown” rather than “unpaid” when evidence is insufficient.
- Export CSV plus PDF monthly reports with clear assumptions, period, currency, approved totals, exception list, source links/IDs and generated time. Do not call these audited financial statements.

## 3. Functional requirements
| ID | Requirement | Acceptance |
|---|---|---|
| F-01 | Offline first-run and model status | Works with network unplugged; explicit missing-model/error states |
| F-02 | Encrypted workspace | No readable records without passphrase; reopen succeeds |
| F-03 | Multi-file import | Real file selection, progress, cancel/retry, error detail |
| F-04 | OCR and structured extraction | Real locally executed OCR, field provenance and review |
| F-05 | CSV statement import | User-selectable column mapping, date/currency parsing, idempotency |
| F-06 | Categorization suggestions | Local model outputs typed category proposals, never unilateral approval |
| F-07 | Reconciliation | Deterministic matching/exception logic; audited decisions |
| F-08 | Chat-backed queries | Every number traces to approved local records; no invented totals |
| F-09 | Contextual evidence | From answer/entry → document/page/field and related transaction |
| F-10 | Review inbox | Approve/correct/reject, undo where feasible, complete audit history |
| F-11 | Reports | Actual file export; financial amounts and provenance independently testable |
| F-12 | Duplicate/unmatched alerts | User-visible traceable candidates, no definitive fraud claims |
| F-13 | Focused trend view | On-demand compact monthly trends, not permanent dashboard |
| F-14 | Natural language document search | Search local indexed evidence with source refs; FTS fallback |
| F-15 | Local encrypted backup/restore | Restorable with passphrase on a second eligible machine |
| F-16 | Offline desktop installer | Windows installer includes or provisions all runtime prerequisites offline |
| F-17 | Reliability | Restart, error recovery, cancellation, log redaction, data migration |
| F-18 | Model options | Display installed model IDs/hashes, compatible hardware mode and fallbacks |

## 4. Feature boundaries and non-goals
**In v1:** import, document classification, matching, exceptions, chat, source navigation, monthly summaries, CSV/PDF export, local backup, optional compact trend chart, keyboard navigation, light/dark reading modes (if both implemented and verified).

**Out of v1:** live bank connectors, payment execution, invoice mailing, cloud sync, multiuser network collaboration, automatic GST filing, OCR for every handwriting/language, full double-entry accounting, autonomous transaction posting, auto-trading, web-based SaaS, mobile app, and autonomous code/tool execution by the chat model.

**No mock mode in shipped app.** Test-only fixtures live in tests and are clearly distinguished from actual import data. There is no sample ledger silently presented as real.

## 5. Quality / privacy requirements
- Offline functionality after full offline installation/model provisioning. Never send files, prompts, telemetry, traces or account information off-device.
- Platform packaging reproducible and pinned; native Windows build and install test required.
- All persisted source documents and records encrypted, including thumbnails/temp copies where applicable. Keys are never committed or logged.
- Threat model includes malicious PDFs, prompt injection embedded in documents, path traversal, dangerous file types, untrusted model output, passphrase loss, process crashes, GPU OOM, and corrupted backup.
- Financial amounts represented in exact decimal/minor units, **never binary floating point**. Multi-currency records are never summed into a single balance without explicit user-provided exchange rates.
- All invoices and parsed records retain source evidence; unverifiable extracted fields require user confirmation.
- Accessibility: usable with keyboard, readable at 125–200% zoom, focus states, reduced motion option, WCAG AA-equivalent contrast target.

## 6. Success signals (targets, not achieved claims)
- Full end-to-end pipeline works with real locally stored inputs on Debian and Windows.
- In test corpus, no silent reconciliation arithmetic errors, no undocumented external requests, no loss of user edits on restart.
- Field extraction quality is measured and reported by category; no arbitrary claims of 99% financial accuracy.
- Uncertain, ambiguous and non-matching records surface in review rather than being silently accepted.
- Installer runs without system Python, developer Node/Rust installation, internet connection, or separate model server setup.
- Users can demonstrate a month-end workflow and locate the source for every reported amount.

## 7. Mandatory ownership decisions before implementation hardens
Confirm: real Windows version/hardware constraints; primary documents and formats; target user segment; offline model distribution and bundle size; app name/branding; backup/passphrase UX; whether multi-currency handling is essential on launch. Agent must raise unknowns rather than invent answers; implement conservative defaults where low risk.
