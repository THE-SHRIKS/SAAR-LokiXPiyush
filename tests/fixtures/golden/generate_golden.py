"""Generates the synthetic golden financial dataset and its independently computed expectations.

Everything here is synthetic. Expected values are computed from the generated rows with plain
integer arithmetic in this script, never from SAAR's code, so the expectations stay independent
of the application under test. Run from the repository root:

    python tests/fixtures/golden/generate_golden.py

Outputs (deterministic, committed):
    tests/fixtures/golden/bank_golden.csv
    tests/fixtures/golden/expected.json
"""

import csv
import json
import os
from datetime import date, timedelta

HERE = os.path.dirname(os.path.abspath(__file__))
BASE_CURRENCY = "USD"

# Currency minor-unit exponents (ISO 4217): JPY has none.
EXPONENT = {"USD": 2, "EUR": 2, "INR": 2, "GBP": 2, "JPY": 0}

rows = []  # (date_iso, description, signed_minor, currency)


def add(d, description, amount_str, currency):
    """amount_str is a decimal string as written in the CSV; minor units are computed exactly."""
    neg = amount_str.startswith("-")
    digits = amount_str.lstrip("-")
    exp = EXPONENT[currency]
    if exp == 0:
        minor = int(digits)
    else:
        whole, _, frac = digits.partition(".")
        frac = (frac + "0" * exp)[:exp]
        minor = int(whole) * (10 ** exp) + int(frac or "0")
    rows.append((d.isoformat(), description, -minor if neg else minor, currency))


# 1. Monthly subscription, 24 months: legitimate repeats, must NOT be flagged as duplicates.
start = date(2024, 1, 1)
for m in range(24):
    d = date(2024 + (m // 12), 1 + (m % 12), 1)
    add(d, "CLOUDHOST SUBSCRIPTION", "-120.00", "USD")

# 2. Two distinct 742.81 debits to the same merchant, 12 days apart: must be flagged for review.
add(date(2026, 9, 13), "NORTHWIND CLOUD LTD, INV 2026-0912", "-742.81", "USD")
add(date(2026, 9, 25), "NORTHWIND CLOUD LTD", "-742.81", "USD")

# 3. Refund and partial refund to the same merchant.
add(date(2026, 9, 18), "NORTHWIND CLOUD LTD REFUND", "50.00", "USD")
add(date(2026, 10, 2), "NORTHWIND CLOUD LTD PARTIAL REFUND", "12.34", "USD")

# 4. Other currencies in a USD workspace: must never be silently merged into USD totals.
add(date(2026, 9, 21), "ACME CONSULTING GMBH", "-1200.00", "EUR")
add(date(2026, 8, 3), "EU SOFTWARE REFUND", "99.99", "EUR")
add(date(2026, 9, 9), "TATA CONSULTANCY", "-15000.50", "INR")
add(date(2026, 9, 10), "SHINKANSEN TICKET", "-5000", "JPY")

# 5. Edge values: zero, very large, descriptions with commas and quotes.
add(date(2026, 6, 30), "ZERO VALUE TEST", "0.00", "USD")
add(date(2026, 6, 30), "LARGE ANNUAL CONTRACT", "-99999999999.99", "USD")
add(date(2026, 6, 1), 'Wire fee, "international" transfer', "-25.00", "USD")

# 6. Volume: more than 50 transactions, spread over months and years.
seed = 7
for i in range(70):
    seed = (seed * 1103515245 + 12345) % (2 ** 31)
    d = date(2025, 1, 1) + timedelta(days=seed % 700)
    cents = 100 + (seed % 90000)
    sign = "-" if (seed // 7) % 3 else ""
    amount = f"{sign}{cents // 100}.{cents % 100:02d}"
    add(d, f"VENDOR {i % 9:02d} SUNDRY", amount, "USD")

# Stable order: by date, then description.
rows.sort(key=lambda r: (r[0], r[1]))

csv_path = os.path.join(HERE, "bank_golden.csv")
with open(csv_path, "w", newline="", encoding="utf-8") as fh:
    w = csv.writer(fh)
    w.writerow(["Date", "Description", "Debit", "Credit", "Currency", "Balance"])
    running = 0
    for d, desc, minor, cur in rows:
        exp = EXPONENT[cur]
        if exp == 0:
            text = f"{abs(minor)}"
        else:
            text = f"{abs(minor) // 10 ** exp}.{abs(minor) % 10 ** exp:0{exp}d}"
        debit = text if minor < 0 else ""
        credit = text if minor > 0 else ""
        w.writerow([d, desc, debit, credit, cur, ""])

# Expected results, computed from `rows` only.
by_currency = {}
for _, _, minor, cur in rows:
    c = by_currency.setdefault(cur, {"count": 0, "income_minor": 0, "expense_minor": 0})
    c["count"] += 1
    if minor > 0:
        c["income_minor"] += minor
    elif minor < 0:
        c["expense_minor"] += -minor
for c in by_currency.values():
    c["net_minor"] = c["income_minor"] - c["expense_minor"]

usd = [r for r in rows if r[3] == BASE_CURRENCY]
monthly = {}
for d, _, minor, _ in usd:
    key = d[:7]
    m = monthly.setdefault(key, {"income_minor": 0, "expense_minor": 0, "count": 0})
    m["count"] += 1
    if minor > 0:
        m["income_minor"] += minor
    elif minor < 0:
        m["expense_minor"] += -minor
for m in monthly.values():
    m["net_minor"] = m["income_minor"] - m["expense_minor"]

def merchant(desc):
    word = next((w for w in desc.replace(",", " ").split() if len(w) >= 3 and not w.isdigit()), "")
    return word.lower()


potential_duplicates = []
subscription_pairs = []
debits = [r for r in usd if r[2] < 0]
for i, (d1, desc1, m1, _) in enumerate(debits):
    for d2, desc2, m2, _ in debits[i + 1:]:
        if m1 != m2 or merchant(desc1) != merchant(desc2):
            continue
        gap = abs((date.fromisoformat(d1) - date.fromisoformat(d2)).days)
        if 1 <= gap <= 14:
            potential_duplicates.append({"merchant": merchant(desc1), "amount_minor": -m1,
                                         "currency": BASE_CURRENCY, "dates": [d1, d2], "gap_days": gap})
        elif 25 <= gap <= 35:
            subscription_pairs.append({"merchant": merchant(desc1), "amount_minor": -m1,
                                       "currency": BASE_CURRENCY, "dates": [d1, d2], "gap_days": gap})

expected = {
    "dataset": "bank_golden.csv",
    "synthetic": True,
    "base_currency": BASE_CURRENCY,
    "transaction_count": len(rows),
    "by_currency": by_currency,
    "usd_monthly": dict(sorted(monthly.items())),
    "potential_duplicate_payments": potential_duplicates,
    "monthly_subscription_pairs_not_flagged": subscription_pairs,
    "largest_usd_expense_minor": max(-r[2] for r in usd if r[2] < 0),
    "malformed_rows_expected_to_be_rejected": 0,
}
with open(os.path.join(HERE, "expected.json"), "w", encoding="utf-8") as fh:
    json.dump(expected, fh, indent=2, sort_keys=True)
    fh.write("\n")

print(f"wrote {len(rows)} rows to {csv_path}")
