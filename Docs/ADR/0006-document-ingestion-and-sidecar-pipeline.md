# ADR 0006: Document Ingestion Engine & Local OCR Sidecar Pipeline

## Status
Accepted and Verified (Gate G2).

## Context
OpenDeck requires an air-gapped document ingestion pipeline that processes digital invoices (PDF), scanned receipts (images/scanned PDFs), and multi-format bank export statements (CSV) without cloud vision APIs, remote parsers, or telemetry. In accordance with `RULES.md`, `SPECS.md`, and `TASKS.md` (P2-01 through P2-06):
1. Ingested files must be validated against file signature magic bytes to block malicious file boundaries.
2. Persisted files must be encrypted with AES-256-GCM via the `BlobStore`, with duplicate files deduplicated by SHA-256 content address.
3. Plaintext buffers passed to sidecar workers must use temporary zeroized storage (`TempPlaintextGuard`).
4. Financial amounts extracted from documents and statements must be stored strictly as integer minor units (cents) with zero floating-point arithmetic.
5. Extracted invoice fields must maintain provenance (page number and bounding box coordinates) to enable highlight navigation in the Evidence Inspector.
6. Malicious inputs (CSV bombs, corrupted PDFs, giant documents) must be caught and rejected gracefully with user-visible errors rather than application crashes.

## Architecture & Implementation Decisions

### 1. Two-Tier Extraction Strategy (`workers/ocr/ocr_worker.py`)
- **Tier A (Born-Digital PDFs):** Fast digital text layer parser using PyMuPDF / `pdfplumber`. Extracts layout, words, lines, font metrics, and bounding boxes (`[x0, top, x1, bottom]`) per page in under 100 ms on CPU with 100% deterministic accuracy.
- **Tier B (Scanned Documents & Receipts):** Local OCR inference using `RapidOCR` with ONNXRuntime AVX2 CPU execution. Extracts text lines, confidence scores, and polygon bounding boxes from raster images and scanned PDF pages.
- **IPC Protocol:** Supervised child process via `WorkerSupervisor`. Inputs passed via CLI flag `--input-file <temp_path>` and structured JSON results emitted to stdout:
  ```json
  {
    "status": "success",
    "tier": "digital" | "ocr",
    "page_count": 1,
    "pages": [
      {
        "page_num": 1,
        "width": 612.0,
        "height": 792.0,
        "lines": [
          {
            "text": "TAX INVOICE",
            "confidence": 1.0,
            "bbox": [50.0, 29.3, 152.2, 45.3]
          }
        ]
      }
    ],
    "error": null
  }
  ```
- **Execution Limits:** Maximum 50 pages for digital PDFs, maximum 20 pages for scanned PDFs. Execution is bounded by a 30-second timeout.

### 2. Zero-Dependency RFC 4180 CSV Mapper (`src-tauri/src/ingestion/csv_mapper.rs`)
- **Format Agnostic:** Auto-detects delimiters (comma, semicolon, tab).
- **Date Normalization:** Normalizes multiple date formats (ISO `YYYY-MM-DD`, British/Indian `DD/MM/YYYY`, US `MM/DD/YYYY`, and month names `15-Sep-2026`) into standard ISO 8601 strings.
- **Sign Convention & Layouts:** Supports single signed `Amount` columns (e.g. US bank statement) as well as split `Withdrawal Amount` / `Deposit Amount` columns (e.g. HDFC bank statement).
- **Exact Minor-Unit Arithmetic:** Parses currency strings directly into integer minor units without binary floating-point representation (`f32`/`f64`).
- **Safety Boundary:** Implements a strict row limit (`MAX_CSV_ROWS = 10,000`) to reject CSV bomb attacks (e.g., `tests/fixtures/adversarial/bomb_50k_rows.csv`).

### 3. Field Schema Normalizer (`src-tauri/src/ingestion/normalizer.rs`)
- Maps extracted lines and tokens to standard financial invoice schemas:
  - `vendor_name`, `invoice_number`, `invoice_date`, `due_date`, `currency`, `subtotal_minor`, `tax_minor`, `total_minor`, and line items.
- Preserves full provenance: each field stores its original raw text, normalized value, confidence score, page number, and bounding box JSON.
- Arithmetic Verification: Cross-checks `subtotal_minor + tax_minor == total_minor`. If a discrepancy is detected, it raises an uncertainty flag for user review.

### 4. Document Orchestrator & Deduplication (`src-tauri/src/ingestion/orchestrator.rs`)
- Validates file signatures (`%PDF-`, `\x89PNG`, `\xff\xd8\xff`, valid UTF-8 CSV).
- Computes SHA-256 and queries the database for existing records within the active workspace. If found, returns `IngestResult::Duplicate` and skips re-storage.
- Atomic SQLite insertions: writes document record, extracted fields, invoice records, line items, and transaction postings within database transactions with rollback on error.
- Appends immutable audit log entries recording file name, doc type, SHA-256, and extracted record counts.

## Verification & Test Evidence

| Test Case | Scope | Execution Command | Result |
|---|---|---|---|
| CSV Amount & Date Normalization | Minor unit math, multi-currency, date variants | `cargo test ingestion::csv_mapper::tests::test_parse_amount_minor_exact` | **Passed (6 tests)** |
| US Bank Statement Parsing | US bank CSV fixture (`bank_us_sample.csv`) | `cargo test ingestion::csv_mapper::tests::test_parse_us_bank_csv_fixture` | **Passed (7 transactions)** |
| HDFC Bank Statement Parsing | Indian bank CSV fixture (`bank_hdfc_sample.csv`) | `cargo test ingestion::csv_mapper::tests::test_parse_hdfc_bank_csv_fixture` | **Passed (6 transactions)** |
| CSV Bomb Rejection | 50,001 row attack vector (`bomb_50k_rows.csv`) | `cargo test ingestion::csv_mapper::tests::test_csv_bomb_rejection` | **Passed (Rejected)** |
| Digital PDF Text Extraction | Born-digital invoice (`gst_b2b_valid.pdf`) | `cargo test ingestion::ocr_client::tests::test_extract_digital_pdf_fixture` | **Passed (Tier A digital)** |
| Corrupted PDF Error Handling | Malformed PDF (`corrupted_truncated.pdf`) | `cargo test ingestion::ocr_client::tests::test_corrupted_pdf_returns_recoverable_error` | **Passed (Recoverable)** |
| Field Normalization (GST) | GST B2B invoice fields & minor-unit amounts | `cargo test ingestion::normalizer::tests::test_normalize_gst_invoice` | **Passed (11800000 minor)** |
| Field Normalization (Receipt) | Thermal receipt scanned image OCR | `cargo test ingestion::normalizer::tests::test_normalize_thermal_receipt` | **Passed (4550 minor)** |
| Full End-to-End Ingestion Pipeline | Ingests PDF, PNG, US CSV, HDFC CSV, Deduplication, Adversarial checks | `cargo test --test p2_ingestion_verification` | **Passed (1/1 full E2E)** |
| Front-end Ingestion UI & Linting | React 18, Evidence Inspector, Types, Lint | `pnpm run lint && pnpm test && pnpm run build` | **0 errors, 4/4 tests** |
| Desktop GTK 3 Verification | Native app launch on Debian `DISPLAY=:0.0` | Launch binary and screenshot via `scrot` | **Screenshot: [`tests/opendeck_debian_p2_verification.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p2_verification.png)** |

**Total Ingestion Tests:** 30/30 Rust tests + 4 Vitest tests passing.
Gate G2 criteria satisfied.
