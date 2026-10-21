# Product UI: Editorial Financial Workspace

## Intent
It must feel like a focused private desktop tool built for serious work, **not** a CRM, business intelligence dashboard, fintech landing page, admin theme, or chatbot pasted onto a chart grid. Chat is the primary surface, but every operation must offer inspectable evidence and explicit review actions.

## Visual direction
- A **quiet document studio**: generous open space, compact typography, precise baseline alignment, restrained dividers, deliberate information density.
- Palette proposal (product-specific; not automatically borrowed from another brand): warm paper `#F1F0EA`, ink `#20231F`, graphite `#30342F`, deep pine `#263C32` for sparing functional emphasis, muted stone `#858780`; dark reading mode `#111411` / `#EEEFE8` only if genuinely polished.
- Use 1 restrained accent; no neon, magenta gradients, glassmorphism, glows, floating HUDs, stock illustrations, bubbly cards, fake financial charts, generic emoji labels, or needless animations.
- Use system fonts with sober text hierarchy; don't depend on CDN font loading.
- Motion: understated, meaningful status changes, reduced-motion support.

## Main layout (1200–1600px target, responsive down to 900px)
```
┌─ App title / workspace ──────────────────────── local model status ──┐
│ slim rail │        conversation + action context         │ evidence  │
│          │                                                │ inspector │
│ import   │  You: reconcile these September invoices       │ source PDF│
│ docs     │  App: 4 matches · 2 uncertain                   │ field box │
│ review   │       [Review 2] [Open sources]                │          │
│ reports  │                                                │          │
│          │     natural language composer / drop zone      │          │
└─────────────────────────────────────────────────────────────────────┘
```
The evidence inspector is closed by default; users open it on a citation or record. When a table is needed, it opens in a dedicated review/document scene, not as permanent dashboard chrome.

## Core screens
1. **Workspace / Conversation:** quiet empty state with import affordance and examples only for supported commands; threaded messages with footnoted local evidence IDs, native copy, follow-up actions, actual progress/error states.
2. **Import tray:** drag/drop and file picker, eligible formats, extraction progress with per-file fail/retry, no fabricated animation.
3. **Document reader:** PDF/image page preview, recognized data alongside source highlights, corrections and review history.
4. **Review queue:** intentional table/list for ambiguous fields, proposed categories and candidate invoice-payment matches with compare, approve, reject; full keyboard actions.
5. **Reports:** focused period selection, selected currencies, export controls, last export receipts, and opt-in tiny trend visualization.
6. **Settings:** model readiness/hash, hardware mode, storage location, encrypted backup/restore, offline guarantee and warnings.

## Key behavior details
- Commands use subtle inline action chips/buttons under responses (e.g., `Review 2 exceptions`), not a global set of KPI widgets.
- No fake assistant typing or invented source cards. Processing indicators reflect real worker events.
- Data presented as source-linked evidence, with clear `AI suggestion`, `Approved`, `Needs review`, `Unmatched` semantics.
- Document panels show provenance: file, page, field or matching bank transaction.
- Destructive and financial-changing actions require precise confirmation and outcome states.
- Chat cannot be the only access path: keyboard-friendly actions/menus available for import, review, export, backup.
- Empty states provide real next steps but never fake charts, customers, dates or amounts.

## Mandatory UI review evidence
At each UI milestone capture **actual running app** screenshots: first launch/no models, blank workspace, import in progress, parsed document with evidence, review/approval, grounded conversation, exported report, offline error/fallback and Windows installed app. Evaluate typography, spacing, focus contrast and overflow at two window sizes. Never substitute Figma mocks or generated images as implementation evidence.

## Anti-template rejection list
Reject a UI if it resembles: CRM kanban; 4 KPI cards on top; left nav with 12 dummy modules; gradient floating assistant orb; identical round dashboard cards; bright graphs in every viewport; black-on-neon cyberpunk; unreadable tiny grey text; chat responses containing made-up statistics; passive buttons with no callbacks; component-library demo styling.
