"""Generates synthetic invoices, a scanned receipt, a corrupted PDF, and a matching bank statement.

All names, amounts and numbers are invented. Totals are computed here in integer minor units,
so the expectations in bills_expected.json do not depend on SAAR. Run from the repository root:

    python tests/fixtures/golden/make_bills.py

Outputs (deterministic):
    tests/fixtures/golden/bills/*.pdf          30 digital invoices
    tests/fixtures/golden/bills/scan_receipt.png  rendered scan (OCR input)
    tests/fixtures/golden/bills/corrupted_invoice.pdf  truncated file (must fail safely)
    tests/fixtures/golden/bank_bills.csv       bank statement with payments for some invoices
    tests/fixtures/golden/bills_expected.json  expected totals and match outcomes
"""

import csv
import json
import os
from datetime import date, timedelta

import pymupdf

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "bills")
os.makedirs(OUT, exist_ok=True)

VENDORS = [
    ("NORTHWIND CLOUD LTD", 18),   # GST-registered: 18% tax
    ("BLUE HARBOR STATIONERY", 0),  # tax-exempt
    ("PINE & OAK CONSULTING", 18),
    ("HELIX DATA SYSTEMS", 0),
    ("MERIDIAN LOGISTICS CO", 5),
]
ITEMS = [
    ("Compute seats", 120_00),
    ("Object storage GB", 20),
    ("Premium support", 99_50),
    ("A4 paper ream", 6_25),
    ("Consulting hour", 150_00),
    ("Freight pallet", 45_75),
]


def fmt(minor):
    return f"{minor // 100}.{minor % 100:02d}"


def tax_of(subtotal_minor, pct):
    # Integer half-up rounding, no floats.
    return (subtotal_minor * pct + 50) // 100


invoices = []  # dicts with number, vendor, date, subtotal, tax, total, pdf
start = date(2026, 6, 1)
for i in range(30):
    vendor, pct = VENDORS[i % len(VENDORS)]
    issued = start + timedelta(days=i * 4)
    number = f"INV-{2026}-{i + 1:04d}"
    lines = []
    subtotal = 0
    for k in range(1 + (i % 3)):
        name, unit = ITEMS[(i + k) % len(ITEMS)]
        qty = 1 + ((i * 7 + k * 3) % 9)
        amount = unit * qty
        lines.append((name, qty, unit, amount))
        subtotal += amount
    tax = tax_of(subtotal, pct)
    total = subtotal + tax
    fname = f"invoice_{i + 1:02d}.pdf"

    doc = pymupdf.open()
    page = doc.new_page()
    y = 60
    for text, size in [
        (vendor, 16),
        (f"Invoice Number: {number}", 11),
        (f"Invoice Date: {issued.isoformat()}", 11),
        ("Bill To: THE SHRIKS Operations", 11),
        ("Description                       Qty        Amount", 11),
    ]:
        page.insert_text((50, y), text, fontsize=size)
        y += size + 8
    for name, qty, unit, amount in lines:
        page.insert_text((50, y), f"{name:<30} {qty:>3}   {fmt(amount):>10}", fontsize=11)
        y += 19
    for text in [
        f"Subtotal                                  {fmt(subtotal):>10}",
        f"Tax {pct}%                                    {fmt(tax):>10}",
        f"Total Due USD                              {fmt(total):>10}",
    ]:
        page.insert_text((50, y), text, fontsize=11)
        y += 19
    doc.save(os.path.join(OUT, fname))
    doc.close()

    invoices.append({
        "file": fname,
        "invoice_number": number,
        "vendor": vendor,
        "issued": issued.isoformat(),
        "subtotal_minor": subtotal,
        "tax_minor": tax,
        "total_minor": total,
    })

# Scanned receipt: an invoice page rendered to an image, to exercise OCR.
scan_src = invoices[0]
doc = pymupdf.open(os.path.join(OUT, scan_src["file"]))
pix = doc[0].get_pixmap(dpi=150)
pix.save(os.path.join(OUT, "scan_receipt.png"))
doc.close()

# Corrupted file: the first bytes of a real PDF, truncated.
with open(os.path.join(OUT, invoices[1]["file"]), "rb") as fh:
    head = fh.read(400)
with open(os.path.join(OUT, "corrupted_invoice.pdf"), "wb") as fh:
    fh.write(head)

# Bank statement. Payments:
#   invoices 1-20: paid in full on time (matched)
#   invoice 21:    paid twice (a potential duplicate payment)
#   invoice 22:    paid with a short amount (partial, must not be matched exactly)
#   invoices 23-30: unpaid (invoice with no bank transaction)
#   one debit with no invoice (bank transaction with no invoice)
bank = []
for inv in invoices[:20]:
    d = date.fromisoformat(inv["issued"]) + timedelta(days=9)
    bank.append((d, f"{inv['vendor']} {inv['invoice_number']}", -inv["total_minor"]))
inv21 = invoices[20]
for days in (9, 14):
    d = date.fromisoformat(inv21["issued"]) + timedelta(days=days)
    bank.append((d, f"{inv21['vendor']} PAYMENT", -inv21["total_minor"]))
inv22 = invoices[21]
bank.append((date.fromisoformat(inv22["issued"]) + timedelta(days=9),
             f"{inv22['vendor']} PAYMENT", -(inv22["total_minor"] - 1000)))
bank.append((date(2026, 9, 30), "UNKNOWN CARD MERCHANT", -4_200))

bank.sort(key=lambda r: r[0])
with open(os.path.join(HERE, "bank_bills.csv"), "w", newline="", encoding="utf-8") as fh:
    w = csv.writer(fh)
    w.writerow(["Date", "Description", "Debit", "Credit", "Currency", "Balance"])
    for d, desc, minor in bank:
        w.writerow([d.isoformat(), desc, fmt(-minor), "", "USD", ""])

expected = {
    "synthetic": True,
    "invoice_count": len(invoices),
    "invoices": invoices,
    "invoice_total_sum_minor": sum(i["total_minor"] for i in invoices),
    "bank_debit_count": len(bank),
    "bank_debit_sum_minor": sum(-m for _, _, m in bank),
    "matched_invoices": [i["invoice_number"] for i in invoices[:20]],
    "potential_duplicate_payment_invoice": inv21["invoice_number"],
    "partial_payment_invoice": inv22["invoice_number"],
    "unpaid_invoices": [i["invoice_number"] for i in invoices[22:]],
    "unmatched_bank_debit": {"description": "UNKNOWN CARD MERCHANT", "amount_minor": 4200},
    "corrupted_file_must_fail": "corrupted_invoice.pdf",
    "scan_file_for_ocr": "scan_receipt.png",
}
with open(os.path.join(HERE, "bills_expected.json"), "w", encoding="utf-8") as fh:
    json.dump(expected, fh, indent=2)
    fh.write("\n")

print(f"wrote {len(invoices)} invoices, {len(bank)} bank rows")
