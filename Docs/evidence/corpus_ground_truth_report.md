# Corpus run against ground truth

## workspace_alpha


invoices compared: 32 of 32
- receipt OK receipt_01_good.png ("good")
- receipt OK receipt_02_fair.png ("fair")
- receipt OK receipt_03_poor.png ("poor")
- receipt OK receipt_04_good.png ("good")
- receipt OK receipt_05_fair.png ("fair")
- receipt OK receipt_06_poor.png ("poor")
- bank EUR OK: 12 rows, income 0, expense 22738282
- bank INR OK: 11 rows, income 0, expense 17706239
- bank USD OK: 144 rows, income 6391654, expense 21387653

reconciliation: 28 links, 59 anomalies
- paid-in-full matched correctly: 20/20
- duplicate payment HDS-2025-0019 flagged: true
- duplicate payment MLC-2025-0030 flagged: true
- duplicate payment POC-2025-0008 flagged: true
- short_payment_invoices MLC-2025-0010 treated as exact match: false
- short_payment_invoices NWC-2025-0021 treated as exact match: false
- partial_payment_invoices BHS-2025-0017 treated as exact match: false
- partial_payment_invoices NWC-2025-0006 treated as exact match: false
- partial_payment_invoices POC-2025-0028 treated as exact match: false
- re-sent invoice flagged: true
- reused invoice number flagged: true

mismatches in workspace_alpha: 0

## workspace_beta


invoices compared: 15 of 15
- receipt OK receipt_01_good.png ("good")
- receipt OK receipt_02_fair.png ("fair")
- receipt OK receipt_03_poor.png ("poor")
- receipt OK receipt_04_good.png ("good")
- bank INR OK: 8 rows, income 0, expense 11354293
- bank USD OK: 64 rows, income 3273100, expense 12428528

reconciliation: 13 links, 30 anomalies
- paid-in-full matched correctly: 10/10
- duplicate payment RPW-2025-0008 flagged: true
- short_payment_invoices OTS-2025-0010 treated as exact match: false
- partial_payment_invoices OTS-2025-0006 treated as exact match: false
- re-sent invoice flagged: true
- reused invoice number flagged: true

mismatches in workspace_beta: 0

