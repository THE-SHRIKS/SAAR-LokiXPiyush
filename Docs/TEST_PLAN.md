# Verification, Test Cases and Release Evidence

## Rules
A test counts only if it ran, its input and expected behavior are recorded, and the observed result is retained. Tests are never to be invented. Production app contains no simulation data. Clearly labeled deterministic fixtures within the test harness are legitimate for reproducible correctness testing, **not** user-facing product behavior.

## Test corpus
Use consenting/anonymized **real-format** invoices, receipts, bank CSV/PDF statements and deliberately malformed documents. Keep sensitive test corpora outside Git; version a redacted manifest with file type, locale, expected ground truth and checksum. Capture permission to use any non-public data. Include Indian GST invoice layouts but do **not** imply automated GST correctness. Maintain a separate adversarial/test-only corpus with intentionally altered financial values, encrypted PDFs and injection instructions; never appear as production import results unless intentionally imported by tester.

## Acceptance matrix
| ID | Test | Required observation |
|---|---|---|
| T-01 | Debian cold launch + restart | native app opens, state persists, no dev web server requirement in packaged build |
| T-02 | Zero-network | deny all external egress, app opens/imports/infers/exports; no fallback or DNS attempts |
| T-03 | Wrong key | wrong passphrase unlock impossible; correct key succeeds; secrets never logged |
| T-04 | Import eligible PDF/image/CSV | actual bytes encrypted, progress events real, no silently accepted unsupported input |
| T-05 | Extraction provenance | selected field navigates to original page/bounding box or source row |
| T-06 | Currency correctness | exact sums for refund/split/multi-currency cases, no float error or mixed-currency aggregate |
| T-07 | Reconciliation ambiguity | exact matches proposed, ambiguous ones flagged, no over-allocation or fabricated payment |
| T-08 | Duplicate and retry | repeated file import is idempotent, candidate false positives reviewable |
| T-09 | Chat groundedness | all numbers computed from approved records and answer contains source IDs |
| T-10 | Prompt injection | instructions inside imported PDF cannot edit database, call shell or disable verification |
| T-11 | Model absence/OOM | explicit and recoverable error, CPU fallback where supported, never fake answer |
| T-12 | Approval/audit | user correction persists, prior value auditable, no unapproved mutation |
| T-13 | Export | CSV/PDF totals exactly match database; reports identify period, currency, limitations |
| T-14 | Backup and restore | encrypted backup authenticated, wrong key rejected, restore succeeds in new profile |
| T-15 | Malformed/untrusted docs | parser limits, path traversal prevention, timeout, no plaintext leaks or crash loop |
| T-16 | UI interactions | keyboard nav, screen sizes, focus, scroll, every displayed action works |
| T-17 | Model manifest | local hash/license/version verified; no auto-download/network dependencies |
| T-18 | Native Windows packaging | clean install on Windows without developer toolchain and with internet disconnected |
| T-19 | Native Windows full workflow | import → OCR → categorize → reconcile → review → chat → export → backup/restore all succeed |
| T-20 | Restart under load | in-progress operation cancels or resumes truthfully; no duplicated transactions |

## Measurement report (fill with measured values, not estimates)
| Measurement | Debian actual | Windows actual | Hardware/environment |
|---|---|---|---|
| app install size | NOT RUN | NOT RUN | |
| model pack size | NOT RUN | NOT RUN | |
| cold start to UI | NOT RUN | NOT RUN | |
| model load CPU/GPU | NOT RUN | NOT RUN | |
| OCR time per sample page | NOT RUN | NOT RUN | |
| exact-field accuracy per field | NOT RUN | NOT RUN | |
| peak RAM / VRAM | NOT RUN | NOT RUN | |
| 20-document processing time | NOT RUN | NOT RUN | |
| chat answer latency | NOT RUN | NOT RUN | |
| external network connections | NOT RUN | NOT RUN | |

## Offline check
Inspect process tree, sockets/listeners, DNS attempts, firewall/packet capture with a deny-all-external rule. Loopback should be absent unless the bundled inference runtime uses authenticated localhost communication, in which case document port binding, token and process lifecycle. Also test launch with internet physically disabled and without preinstalled model runtimes. A passed check means observed local operation; it does not mean OS itself has no network activity.

## Mandatory evidence bundle
- Date/time, Git commit/hash, target OS/hardware, dependency lock hashes, installed model weights hashes.
- Actual build and test commands, exit codes and relevant redacted logs.
- Screenshots/video of installed Windows app: empty, import, OCR source, review, chat with source citations, report, backup restore.
- Screenshot of user-visible failures and recovery: wrong passphrase, unsupported PDF, model missing, worker failure.
- Exported report and independently calculated expected totals (redacted or appropriately licensed).
- Native Windows installer hash/size, SBOM and third-party license notices.
- Open issues list and explicit owner sign-off. Do not mark release completed before all gates.

## Failure policy
Any test failing fundamental privacy, record accuracy, restore reliability or release packaging blocks release. A test not performed is **NOT RUN**, never implicitly pass. Never weaken the test to make code look complete.
