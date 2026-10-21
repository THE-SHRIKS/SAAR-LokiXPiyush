# Agent Operating Contract — Read First

You are implementing a local-only financial desktop application for THE SHRIKS. Requirements come from this repository; do not replace them with your preferred boilerplate or invent progress.

**Mandatory precedence:** `RULES.md` → `PRD.md` → `SPECS.md` → `UI_DESIGN.md` → `PLAN.md` → `TASKS.md` → `TEST_PLAN.md`. Raise a documented conflict rather than silently choosing.

**At the beginning of each session:** inspect files and Git state; read `STATUS.md`; select the earliest incomplete task without unmet dependencies; write a brief execution plan; work; run tests; capture proof; update `STATUS.md` with exact changed files, commands/results, failures, and next task.

**Before declaring anything done:** show a test result and actual behavior. A rendered UI without backend wiring is not a feature. A model stub is not inference. A fabricated transaction is not a user record. Successful Debian execution does not prove Windows shipping.

**Research discipline:** use official API docs, inspected installed package interfaces, and actual runtime errors. If uncertain, say **UNVERIFIED**, investigate, and record the result in `SOURCE_NOTES.md`. Never invent package methods, CLI flags, license claims, benchmarks, model compatibility, or passing tests.

**Stop conditions:** security failure, inaccessible key material, unexplained network traffic, corrupt/missing financial provenance, unsupported build toolchain, extraction/reconciliation discrepancies, or unverified runtime platform requirements. Fix or report blocked; do not bypass with a mock.

**Final response format to owner:** `Completed`, `Tested with evidence`, `Not completed/blocked`, `Files changed`, `Next step`. No implied approvals.
