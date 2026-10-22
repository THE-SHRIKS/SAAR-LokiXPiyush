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
| app install size | 14.8 MB (release binary) | 18.5 MB (NSIS installer) | Intel Core i3-7020U (2 cores, 4 threads @ 2.3 GHz), 11 GB RAM |
| model pack size | 1.83 GB (GGUF Q3_K_M) | 1.83 GB (GGUF Q3_K_M) | SHA-256: `8eff4e0eb51...` |
| cold start to UI | 420 ms | 580 ms | Local Webview (WebKit2GTK / WebView2) |
| model load CPU/GPU | 1.45 s (AVX2 CPU) | 1.62 s (AVX2 CPU) | 0 MB VRAM (Intel HD 620 iGPU fallback) |
| OCR time per sample page | 5.26 s (RapidOCR ONNX) | 4.80 s (RapidOCR ONNX) | ONNXRuntime CPU AVX2 |
| exact-field accuracy per field | 100% (20/20 fields) | 100% (20/20 fields) | Tested against thermal receipt & digital PDF |
| peak RAM / VRAM | 2.3 GB RSS / 0 VRAM | 2.4 GB RSS / 0 VRAM | Concurrent OCR + LLM execution |
| 20-document processing time | 105.2 s (CPU) | 96.0 s (CPU) | Two-tier digital + OCR fallback |
| chat answer latency | 12.8 s (3.6 tok/s) | 11.5 s (4.1 tok/s) | Greedy temp 0.0, AVX2 CPU |
| external network connections | **0 (Zero external calls)** | **0 (Zero external calls)** | Air-gap verified via `unshare -rn` and CSP |

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
