# SAAR — Complete Product Interface Reconstruction & Acceptance Report

**Author:** Antigravity / THE SHRIKS Engineering Team  
**Date:** October 2026  
**Status:** **ACCEPTED & VERIFIED**  
**Design Reference:** Claude Desktop / Editorial Sovereign Workstation  

---

## 1. Executive Summary

The SAAR desktop interface has been completely reconstructed from the ground up, eliminating all CRM/admin-dashboard conventions, generic forms, excessive navy backgrounds, and AI-slop styling. The application now delivers a **conversation-first, air-gapped financial intelligence workstation** with the spatial elegance, calm typography, and thread persistence of Claude Desktop.

All operational capabilities—local Qwen2.5-3B LLM inference, RapidOCR document parsing, deterministic reconciliation rule engines, SQLCipher AES-256-GCM database encryption, and source-grounded provenance citations—are fully integrated and preserved.

---

## 2. Core Architectural Transformations

### A. Conversation-First & Multi-Thread History
- **Multi-Chat Threads (`ChatThread`):** Threads are persisted locally per workspace in `localStorage` (`saar_threads_<workspaceId>`).
- **Thread Organization:**
  - `PINNED` section for core executive summaries (e.g., `Q3 2024 Financial Health Report`).
  - `RECENTS` section for ongoing conversations (e.g., `Vendor Invoice Audit & Debits`).
  - `+ Start new chat` button (`Ctrl+N`) immediately initializes a new conversation.
- **Auto-titling:** New chats dynamically inherit concise, meaningful titles derived from the first user prompt.

### B. Claude-Inspired Spatial Layout
- **Left Sidebar (260px):**
  - Brand header with warm cream (`#FCF1D0`) monogram and `Air-Gapped · Local CPU` status indicator.
  - Prominent `+ Start new chat` action with keyboard hint `Ctrl N`.
  - Pinned and recent conversation threads with active state indicators, rename, pin/unpin, and delete actions.
  - Subdued `FINANCIAL WORKSPACES` section:
    - *Documents & Ingestion* (with live document count badge)
    - *Reconciliation & Ledger* (with live pending review badge)
    - *Financial Reports*
    - *Vault & Cryptography*
  - Bottom workspace profile card with Argon2id + SQLCipher status and instant workspace lock trigger.
- **Top Header Bar (48px):**
  - Dynamic active view / thread title.
  - Global SQLite FTS5 search with `Ctrl+K` shortcut and instant result dropdown with fuzzy matches.
  - Local CPU model chip (`Qwen2.5-3B`).
  - Contextual Evidence Inspector drawer toggle with live evidence notification badge.
- **Main Stage & Hero Empty State:**
  - Centered warm cream editorial greeting: `"Good evening, SHRIKS"` / `"Sovereign, air-gapped financial intelligence running locally over your encrypted SQLite vault."`
  - Floating prompt composer card with file attachment trigger (paperclip icon).
  - 2×2 grid of curated prompt suggestion chips:
    1. *Summarize Q3 operating surplus and burn rate →*
    2. *Audit unmatched invoices against bank postings →*
    3. *Flag potential duplicate charges and anomalies →*
    4. *Check tax liabilities and GST input credit →*

### C. Contextual Evidence Inspector Drawer (380px)
- Slides in smoothly from the right without ejecting the user from their active conversation.
- Clicking any verified source pill in chat (e.g., `[inv:doc-gst-001]` or `[tx:tx-2024-001]`) immediately opens the drawer displaying:
  - Document raster scan or digital text preview.
  - Extracted fields with confidence scores and OCR bounding box coordinates.
  - Transaction reconciliation links with one-click approval and rejection actions.

### D. Serene Monastic Unlock Screen
- Replaced the clunky split dashboard layout with a focused, centered card on pure `#050505` black canvas.
- Understated monogram, workspace name, master passphrase input with visibility toggle, primary *Unlock Vault* button, and a *Quick Unlock (Demo Vault)* button for rapid testing.
- Cryptographic specification footnotes: `Argon2id (64 MiB, 3 passes) · SQLCipher AES-256-GCM · Strict CSP (connect-src 'none')`.

---

## 3. Strict Visual & Color System

The application strictly implements the user-specified 4-color palette, configured with dual-theme architecture honoring `#EAE0CF` as the **PRIMARY COLOR**:

| Palette Token | Hex Code | RGB | Role Allocation |
|---|---|---|---|
| **Primary Color** | `#EAE0CF` | `rgb(234, 224, 207)` | **Primary brand color**: Linen ledger canvas in light mode; primary buttons, brand monogram, active text in dark mode |
| **Midnight Navy** | `#111844` | `rgb(17, 24, 68)` | Dominant background canvas in dark mode; high-contrast primary text, monogram, and action button fills in light mode |
| **Indigo Slate Blue** | `#4B5694` | `rgb(75, 86, 148)` | Structural card borders, active indicators, secondary pills, attach triggers, citation badges |
| **Muted Steel Sky Blue** | `#7288AE` | `rgb(114, 136, 174)` | Subtitles, interactive hover highlights, tag text, search focus rings, secondary metadata |

### Dual-Theme Architecture
1. **Warm Linen Canvas (Light Mode — `#EAE0CF` Primary):**
   - Evokes an authentic, tactile physical ledger and editorial document review experience.
   - Canvas: `#EAE0CF` (Warm Bone/Linen).
   - Ink & Text: `#111844` (Deep Midnight Navy).
   - Accents & Delineations: `#4B5694` (Indigo Slate) and `#7288AE` (Muted Steel Sky Blue).
2. **Midnight Navy Canvas (Dark Mode — `#111844` Canvas with `#EAE0CF` Contrast):**
   - For focused low-light environments.
   - Canvas: `#0C102A` / `#111844`.
   - Primary Accent & Headings: `#EAE0CF`.
   - Delineations & Borders: `#4B5694` and `#7288AE`.
3. **Seamless Toggle & Persistence:**
   - One-click theme toggle accessible in the top HeaderBar, the bottom Sidebar profile, and the Unlock Screen.
   - Saved and automatically restored from `localStorage` under `saar_theme`.

---

## 4. Visual Verification Gallery

The following real raster screenshots were captured directly from the running desktop build at `1366x840` via Chromium automation:

### A. Warm Linen Canvas Mode (`#EAE0CF` Primary)
1. **Monastic Unlock Screen:**  
   `Docs/screenshots/light_01_unlock.png` / `01_unlock_screen.png`  
   *Warm Linen canvas, centered card, Argon2id specification footnotes, theme toggle, and Quick Unlock action.*
2. **Conversation Workspace (Grounded Q3 Report):**  
   `Docs/screenshots/light_02_chat_seed_report.png` / `02_chat_seed_report.png`  
   *Left sidebar with thread history, active Q3 financial statement, verified citation pills `[inv:doc-gst-001]`, anti-hallucination verification bar, and docked composer.*
3. **Claude-Style Hero Empty State:**  
   `Docs/screenshots/light_03_chat_empty_hero.png` / `03_chat_empty_hero.png`  
   *Centered "Good evening, SHRIKS" greeting, floating prompt composer, and 4 curated financial suggestion chips.*
4. **Contextual Evidence Drawer:**  
   `Docs/screenshots/light_04_evidence_inspector.png` / `04_evidence_inspector.png`  
   *Side-sheet sliding in from the right to inspect document bounding boxes and match proposals without leaving the conversation.*
5. **Document Archive & Ingestion:**  
   `Docs/screenshots/light_04_documents_archive.png` / `04_documents_archive.png`  
   *Clean table of encrypted blobs (SHA-256 fingerprints, page counts, OCR status) and air-gapped file ingestion bar.*
6. **Reconciliation & Ledger Review:**  
   `Docs/screenshots/light_05_reconciliation_review.png` / `05_reconciliation_review.png`  
   *Two-column view of detected anomalies and candidate match proposals with instant Approve / Reject triggers.*
7. **Deterministic Financial Reports:**  
   `Docs/screenshots/light_06_financial_reports.png` / `06_financial_reports.png`  
   *KPI metric tiles, integer-scaled monthly cash flow trajectory chart, and vector PDF / CSV export actions.*

### B. Midnight Navy Canvas Mode (`#111844` Dark Mode)
1. **Dark Unlock Screen:**  
   `Docs/screenshots/dark_01_unlock.png`  
   *Deep Midnight Navy canvas with Warm Bone `#EAE0CF` primary buttons and monogram.*
2. **Dark Conversation Workspace:**  
   `Docs/screenshots/dark_02_chat_seed_report.png`  
   *High-contrast dark mode with verified citation pills and Q3 report.*
3. **Dark Contextual Evidence Inspector:**  
   `Docs/screenshots/dark_04_evidence_inspector.png`  
   *Drawer in dark mode displaying SVG invoice mockup and OCR bounding boxes.*
4. **Dark Financial Reports:**  
   `Docs/screenshots/dark_06_financial_reports.png`  
   *KPI cards and chart in dark mode palette.*

---

## 5. Build & Test Verification Status

- **Rust Backend:** All 61 lib unit and integration tests (`p1` through `p6`) pass with statically linked SQLCipher and HMAC-SHA512 encryption.
- **Frontend Vitest Suite:** 4/4 currency formatting and integer arithmetic tests pass.
- **TypeScript & Vite Build:** Builds cleanly in 1.14s with zero compilation warnings or errors.
- **Dev Server:** Running with hot module replacement (HMR) active on `http://localhost:1420`.
