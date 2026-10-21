# Non-Negotiable Implementation Rules

## Truthfulness / anti-hallucination
1. Never say implemented unless compiled, exercised, and verified. Record command, environment, result and screenshot/log ID where relevant.
2. No placeholders, TODOs, fake buttons, canned chats, mock transactions, simulated imports, synthetic progress, stub model inference, demo charts or inactive tabs in **release code**. Clearly marked test fixtures are permissible in `tests/` only.
3. No invention of user data, invoices, merchant names, source citations, model results, release artifacts, benchmarks or successful tests.
4. If an API/library/version is uncertain, inspect installed code or official documentation and prove with a small real integration test before adopting.
5. If blocked, report the blocker and viable options; never hide it with a UI that appears to work.
6. Never auto-claim PMF, users, bookkeeping certification or tax-compliance accuracy.

## Privacy / local-first
7. All inference, document processing, storage, indexing, calculations, logs and reports run on user-owned device. No API keys or network services in app runtime. Development downloads may be necessary for packages/model weights; that does not authorize runtime network traffic.
8. No hidden updates, telemetry, CDN assets, browser trackers, account flows or automatic model downloads.
9. No unencrypted persistent import/derived previews. Do not log sensitive finance fields, file contents or passphrases.
10. Offline failure must be explicit and recoverable; **never** silently call remote inference.

## Financial correctness
11. Rust deterministic functions own amounts, arithmetic, reconciliation, status and exports. LLM proposes or explains only.
12. Never use floating-point money arithmetic. Respect currency and direction, exact parse errors and manual exchange inputs.
13. Always show traceable source evidence. Failed verification -> `needs review`; no optimistic auto-approvals.
14. Chat text inside documents is untrusted input, never system instructions or executable SQL/code.
15. All record edits are transactional, audited and reversible where feasible. No hidden writes from chat.

## Product design
16. Native installed desktop app, not browser-only front end. Chat-first editorial workspace, **no CRM/dashboard UI**.
17. No neon, gradient glows, stock look, floating HUDs, decorative fake data or generic SaaS template.
18. Every visible control must work, have loading/error/empty states and be keyboard accessible. Hide unimplemented features rather than shipping dormant controls.
19. UI copy must reflect actual system state. If OCR is unavailable, show `OCR unavailable`, not `AI ready`.

## Engineering and delivery
20. Keep code modular, typed, tested, pinned in lockfiles with documented licenses and reproducible builds. Exclude model weights, customer data, encrypted keys and logs from Git.
21. Tasks must satisfy explicit evidence before moving to next phase. Update `STATUS.md` continuously and link issues to task IDs.
22. Unit + integration + end-to-end tests required, plus manually verified real documents, offline traffic checks and destructive-operation checks.
23. Model hardware/OS support, performance and license terms must be proven before release. Debian execution is not Windows compatibility.
24. Do not ship until imported data → OCR/extraction → classification → matching → user review → grounded chat → report → encrypted backup/restore works in the **installed Windows app**.
25. On mismatch between goals and feasibility, stop, propose technically sound alternatives and obtain approval before changing locked scope.
