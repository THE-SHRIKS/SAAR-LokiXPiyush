# ADR 0009: Editorial Interface, Evidence Inspector, Manual Audit Corrections, and Cash Flow Trends

## Status
Accepted and Verified (Gate G5).

## Context
Phase 5 integrates all OpenDeck capabilities into a cohesive, responsive editorial desktop interface adhering to `RULES.md`, `SPECS.md`, and `TASKS.md` (P5-01 through P5-06):
1. **Polished Conversation Workspace (P5-01):** Real offline Qwen2.5-3B model status, hardware utilization indicators, interactive prompt chips, and conversation bubbles with provenance citation links.
2. **Evidence Inspector & Coordinate Overlays (P5-02):** Decrypt authentic document file blobs from `BlobStore` on-demand into base64 data URIs. Support images (PNG, JPEG), PDFs, and CSVs. Highlight bounding box rectangles directly over document canvases at exact normalized coordinates (`[x1, y1, x2, y2]`) upon selecting or hovering over normalized fields.
3. **Review Surface, Manual Corrections & Immutable Audit Trail (P5-03):** Interactive match approval/rejection actions, manual transaction description corrections with before/after state captures, and a dedicated audit trail table displaying append-only records from `audit_log`.
4. **Deterministic Financial Reports & Trend Visualizations (P5-04):** SVG-based monthly cash flow bar chart comparing income vs expenses, MoM growth calculations, and tabular breakdowns derived from integer minor units.
5. **Security & Runtime Dashboard (P5-05):** Transparent display of storage encryption (Argon2id + AES-256-GCM), air-gap boundary (`connect-src 'none'`), zero telemetry, and model integrity.
6. **UI Audit & Accessibility (P5-06):** Verified at 1280px and 960px widths with zero dead controls and keyboard navigation support.

## Architecture & Implementation Decisions

### 1. Document Preview Retrieval (`src-tauri/src/commands.rs`)
- **Command:** `get_document_preview(document_id: String) -> Result<DocumentPreview, String>`.
- **Decryption:** Decrypts file payload using active `master_key` from `BlobStore`.
- **MIME Type Handling:** Identifies `image/png`, `image/jpeg`, `application/pdf`, and `text/csv`.
- **Memory Safety:** Raw bytes are held only in memory, encoded into standard base64 strings, and immediately zeroized/dropped.

### 2. Evidence Inspector Coordinate Overlay (`src/App.tsx`)
- Reads normalized bounding boxes (`[x1, y1, x2, y2]`) from `document_fields`.
- Injects an absolute highlight rectangle (`.bbox-overlay-highlight`) over the rendered document image when a user selects or hovers over a normalized field.

### 3. Manual Transaction Correction & Audit Trail (`src-tauri/src/commands.rs`, `src/App.tsx`)
- Inline transaction editing modal for updating transaction descriptions with full audit provenance.
- Calls `ReviewService::correct_transaction`, writing previous and updated descriptions into the immutable `audit_log` table.
- Exposes `list_audit_log` Tauri command to render chronological audit events in the review inbox.

### 4. Deterministic Monthly Trend Chart (`src/App.tsx`)
- Renders an inline SVG bar chart comparing monthly income (green) vs expenses (red) side-by-side.
- Heights and values are computed strictly from integer minor units with zero binary floating-point inaccuracy.

## Verification & Test Evidence
- **All Rust Unit Tests:** 48 unit tests passing (`cargo test --lib`).
- **Frontend Unit Tests:** 4 Vitest unit tests passing (`pnpm test`).
- **Frontend Production Build:** `tsc && vite build` completed in 5.25s with 0 errors.
- **Phase 5 End-to-End Test (`src-tauri/tests/p5_editorial_interface_verification.rs`):** Passed in 8.75s:
  - Step 1: Ingested receipt scan, decrypted PNG bytes verifying `\x89PNG` magic bytes, and verified coordinate bounding boxes.
  - Step 2: Corrected ledger transaction description and verified immutable before/after audit log entry.
  - Step 3: Approved proposed reconciliation link and verified transition to `APPROVED` with audit capture.
  - Step 4: Computed deterministic financial summary and monthly breakdown.
- **Desktop Window Verification at Two Resolutions:**
  - 1280px: [`tests/opendeck_debian_p5_verification_1280.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_1280.png)
  - 960px: [`tests/opendeck_debian_p5_verification_960.png`](file:///home/theshriksloki/OpenDeck/tests/opendeck_debian_p5_verification_960.png)
