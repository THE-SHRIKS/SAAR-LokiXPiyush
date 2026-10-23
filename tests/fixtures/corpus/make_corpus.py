"""Synthetic financial document corpus with an independent ground-truth ledger.

PROVENANCE: every vendor, client, invoice number, address and amount here is invented by this
script. Nothing is copied from real customers or real third-party documents. Totals are computed
in integer minor units in this script, never by SAAR, so ledger.json is independent ground truth.

Two workspaces with different seeds, vendors, currency mixes and outcomes are produced, so the
application can be shown to compute different correct results for different data.

Run from the repository root:

    python tests/fixtures/corpus/make_corpus.py

Outputs per workspace (tests/fixtures/corpus/<workspace>/):
    invoices/*.pdf       multi-layout digital invoices (some multi-page)
    receipts/*.png       noisy scanned receipts (OCR input)
    bank_statement.csv   long bank statement
    ledger.json          ground truth: invoices, bank rows, expected matches and anomalies
"""

import csv
import json
import os
import random
from datetime import date, timedelta

import pymupdf
from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------- number / date formatting


def group_western(n):
    s = str(n)
    parts = []
    while len(s) > 3:
        parts.insert(0, s[-3:])
        s = s[:-3]
    parts.insert(0, s)
    return ",".join(parts)


def group_indian(n):
    s = str(n)
    if len(s) <= 3:
        return s
    head, tail = s[:-3], s[-3:]
    parts = []
    while len(head) > 2:
        parts.insert(0, head[-2:])
        head = head[:-2]
    if head:
        parts.insert(0, head)
    return ",".join(parts) + "," + tail


def money(minor, currency, style):
    """Format minor units as text without floats. style: 'us' | 'eu' | 'in'."""
    whole, frac = divmod(abs(minor), 100)
    sign = "-" if minor < 0 else ""
    if style == "eu":
        return f"{sign}{group_western(whole).replace(',', '.')},{frac:02d}"
    if style == "in":
        return f"{sign}{group_indian(whole)}.{frac:02d}"
    return f"{sign}{group_western(whole)}.{frac:02d}"


def date_text(d, style):
    return d.strftime("%d.%m.%Y") if style == "eu" else d.isoformat()


def tax_of(base_minor, pct):
    return (base_minor * pct + 50) // 100  # integer half-up


# ---------------------------------------------------------------- invoice model

CATALOG = [
    ("Managed compute seat (monthly)", 12000),
    ("Object storage, per TB", 2500),
    ("Premium support plan", 9950),
    ("A4 copier paper, carton of 5 reams", 3125),
    ("Senior consulting hour", 15000),
    ("Freight, pallet, regional", 4575),
    ("Security audit, per system", 48000),
    ("Network switch, 24 port", 31900),
    ("Data migration service, per day", 72000),
    ("Training seat, per attendee", 18000),
]


def make_invoice(rng, idx, vendor, client, currency, style, layout, base_date, long_form):
    n_lines = rng.randint(46, 58) if long_form else rng.randint(1, 5)
    lines = []
    for _ in range(n_lines):
        name, unit = rng.choice(CATALOG)
        qty = rng.randint(1, 12)
        gross = unit * qty
        lines.append({"description": name, "qty": qty, "unit_minor": unit, "line_minor": gross})
    subtotal = sum(l["line_minor"] for l in lines)
    disc_pct = rng.choice([0, 0, 5, 10])
    discount = (subtotal * disc_pct + 50) // 100
    taxable = subtotal - discount
    tax_pct = {"USD": rng.choice([0, 8, 18]), "EUR": rng.choice([19, 7]), "INR": 18}[currency]
    tax = tax_of(taxable, tax_pct)
    total = taxable + tax
    return {
        "number": f"{vendor['code']}-{base_date.year}-{idx:04d}",
        "vendor": vendor["name"],
        "client": client,
        "currency": currency,
        "style": style,
        "layout": layout,
        "issued": base_date.isoformat(),
        "due": (base_date + timedelta(days=30)).isoformat(),
        "lines": lines,
        "subtotal_minor": subtotal,
        "discount_pct": disc_pct,
        "discount_minor": discount,
        "tax_pct": tax_pct,
        "tax_minor": tax,
        "total_minor": total,
        "pages": 1,
    }


def render_invoice(inv, path, defects):
    """Render with one of three layouts. `defects` may blank the date or break the arithmetic."""
    doc = pymupdf.open()
    page = doc.new_page()
    style = inv["style"]
    cur = inv["currency"]
    layout = inv["layout"]
    y = 56

    def put(text, size=10, x=50):
        nonlocal page, y
        if y > 780:
            page = doc.new_page()
            y = 56
            inv["pages"] += 1
        page.insert_text((x, y), text, fontsize=size)
        y += size + 6

    issued = "" if "missing_date" in defects else date_text(date.fromisoformat(inv["issued"]), style)
    due = date_text(date.fromisoformat(inv["due"]), style)
    shown_total = inv["total_minor"] + (1 if "arith_mismatch" in defects else 0)

    if layout == "A":  # classic
        put(inv["vendor"], 16)
        put(f"Invoice No: {inv['number']}")
        put(f"Date: {issued}")
        put(f"Due: {due}")
        put(f"Bill To: {inv['client']}")
        put("Description                              Qty     Unit        Amount")
    elif layout == "B":  # modern, right-heavy
        put("INVOICE", 20)
        put(inv["vendor"], 12)
        put(f"Reference    {inv['number']}")
        put(f"Issued       {issued}")
        put(f"Payment due  {due}")
        put(f"Customer     {inv['client']}")
        put(f"Currency     {cur}")
        put("Item                                      Qty     Price       Total")
    else:  # C, European style
        put(inv["vendor"], 14)
        put(f"Rechnung / Invoice: {inv['number']}")
        put(f"Datum: {issued}")
        put(f"Faellig: {due}")
        put(f"Kunde: {inv['client']}")
        put(f"Waehrung: {cur}")
        put("Beschreibung                             Menge   Einzel      Betrag")

    for l in inv["lines"]:
        put(
            f"{l['description'][:38]:<38} {l['qty']:>3} {money(l['unit_minor'], cur, style):>12} {money(l['line_minor'], cur, style):>13}"
        )
    put(f"Subtotal {cur}:  {money(inv['subtotal_minor'], cur, style)}")
    if inv["discount_pct"]:
        put(f"Discount {inv['discount_pct']}%:  -{money(inv['discount_minor'], cur, style)}")
    put(f"Tax {inv['tax_pct']}%:  {money(inv['tax_minor'], cur, style)}")
    put(f"Total Due {cur}:  {money(shown_total, cur, style)}", 12)
    put(f"Page-count marker: synthetic test document, provenance SAAR-corpus", 7)
    doc.save(path)
    doc.close()


# ---------------------------------------------------------------- scanned receipts


def font(size):
    for p in ("C:/Windows/Fonts/arial.ttf", "C:/Windows/Fonts/cour.ttf"):
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()


def render_receipt(rec, path, rng, quality):
    img = Image.new("L", (520, 640), 245)
    d = ImageDraw.Draw(img)
    f = font(22)
    y = 24
    for text in [rec["merchant"], rec["address"], f"DATE {rec['date']}", "-" * 28]:
        d.text((24, y), text, fill=20, font=f)
        y += 30
    for name, minor in rec["items"]:
        d.text((24, y), f"{name[:22]:<22}", fill=20, font=f)
        d.text((380, y), money(minor, rec["currency"], "us"), fill=20, font=f)
        y += 30
    d.text((24, y), "-" * 28, fill=20, font=f)
    y += 30
    d.text((24, y), "TOTAL", fill=0, font=font(26))
    d.text((360, y), f"{rec['currency']} {money(rec['total_minor'], rec['currency'], 'us')}", fill=0, font=font(26))
    if quality == "poor":
        img = img.rotate(rng.uniform(-2.5, 2.5), expand=True, fillcolor=240)
        img = img.filter(ImageFilter.GaussianBlur(1.1))
        px = img.load()
        for _ in range(img.width * img.height // 25):
            x, yy = rng.randrange(img.width), rng.randrange(img.height)
            px[x, yy] = max(0, min(255, px[x, yy] + rng.randint(-60, 60)))
    elif quality == "fair":
        img = img.rotate(rng.uniform(-1.0, 1.0), expand=True, fillcolor=240)
        img = img.filter(ImageFilter.GaussianBlur(0.5))
    img.save(path)


# ---------------------------------------------------------------- workspace builder


def build_workspace(name, seed, vendors, clients, currency_mix, n_invoices, n_receipts, bank_months):
    rng = random.Random(seed)
    root = os.path.join(HERE, name)
    for sub in ("invoices", "receipts"):
        os.makedirs(os.path.join(root, sub), exist_ok=True)

    invoices = []
    base = date(2025, 3, 3)
    for i in range(n_invoices):
        vendor = vendors[i % len(vendors)]
        currency = currency_mix[i % len(currency_mix)]
        style = {"USD": "us", "EUR": "eu", "INR": "in"}[currency]
        layout = "CAB"[i % 3] if currency == "EUR" else "ABC"[i % 3]
        issued = base + timedelta(days=i * 9 + rng.randint(0, 4))
        inv = make_invoice(rng, i + 1, vendor, clients[i % len(clients)], currency, style, layout, issued, long_form=(i % 5 == 4))
        invoices.append(inv)

    # Deliberate document defects (a person should review these).
    defect_map = {}
    defect_map[2] = ["missing_date"]
    defect_map[5] = ["arith_mismatch"]
    # A re-sent invoice: same vendor and amount, different number (suspected duplicate).
    resend = dict(invoices[1])
    resend["number"] = invoices[1]["number"] + "-R"
    resend["issued"] = (date.fromisoformat(invoices[1]["issued"]) + timedelta(days=2)).isoformat()
    resend["pages"] = 1
    invoices.append(resend)
    # A duplicate invoice NUMBER with a different amount (same vendor).
    clash = make_invoice(rng, 999, vendors[0], clients[0], invoices[0]["currency"], invoices[0]["style"], invoices[0]["layout"], base + timedelta(days=300), False)
    clash["number"] = invoices[0]["number"]
    invoices.append(clash)

    for i, inv in enumerate(invoices):
        fname = f"{inv['number'].replace('/', '-')}{'_clash' if inv is clash else ''}.pdf"
        inv["file"] = fname
        render_invoice(inv, os.path.join(root, "invoices", fname), defect_map.get(i, []))
        inv["defects"] = defect_map.get(i, [])

    # Receipts
    receipts = []
    merchants = ["HARBOR CAFE", "QUICKMART 24", "CITY PARKING", "DELTA STATIONERS", "GREEN TAXI CO", "UNION PRINT SHOP"]
    for i in range(n_receipts):
        cur = currency_mix[i % len(currency_mix)]
        items = [(rng.choice(["Coffee", "Parking 2h", "Notebook", "Printing", "Taxi fare", "Cable"]), rng.randint(250, 6500)) for _ in range(rng.randint(2, 4))]
        rec = {
            "merchant": merchants[i % len(merchants)],
            "address": "12 Example Street, Testville",
            "date": (date(2025, 4, 1) + timedelta(days=i * 13)).isoformat(),
            "currency": cur,
            "items": items,
            "total_minor": sum(m for _, m in items),
            "quality": ["good", "fair", "poor"][i % 3],
        }
        rec["file"] = f"receipt_{i + 1:02d}_{rec['quality']}.png"
        render_receipt(rec, os.path.join(root, "receipts", rec["file"]), rng, rec["quality"])
        receipts.append(rec)

    # Bank statement
    bank = []  # dicts: date, description, minor (signed), currency, kind, ref
    pay_plan = {}
    for i, inv in enumerate(invoices):
        pay_plan[inv["number"] + ("#clash" if inv is clash else "")] = None
    for i, inv in enumerate(invoices):
        if inv is clash:
            continue
        issued = date.fromisoformat(inv["issued"])
        total = inv["total_minor"]
        mode = i % 11
        if mode == 3:
            continue  # unpaid
        if mode == 5:  # two installments
            half = total // 2
            bank.append(dict(date=issued + timedelta(days=8), desc=f"{inv['vendor']} {inv['number']} PART 1", minor=-half, cur=inv["currency"], kind="partial", inv=inv["number"]))
            bank.append(dict(date=issued + timedelta(days=22), desc=f"{inv['vendor']} {inv['number']} PART 2", minor=-(total - half), cur=inv["currency"], kind="partial", inv=inv["number"]))
        elif mode == 7:  # paid twice
            for dd in (9, 15):
                bank.append(dict(date=issued + timedelta(days=dd), desc=f"{inv['vendor']} {inv['number']}", minor=-total, cur=inv["currency"], kind="duplicate_payment", inv=inv["number"]))
        elif mode == 9:  # short payment
            bank.append(dict(date=issued + timedelta(days=11), desc=f"{inv['vendor']} {inv['number']}", minor=-(total - 2500), cur=inv["currency"], kind="short_payment", inv=inv["number"]))
        else:
            bank.append(dict(date=issued + timedelta(days=rng.randint(5, 20)), desc=f"{inv['vendor']} {inv['number']}", minor=-total, cur=inv["currency"], kind="matched", inv=inv["number"]))

    # Recurring subscriptions, fees, refunds, FX settlements and noise.
    first = date(2025, 3, 1)
    for m in range(bank_months):
        y, mo = first.year + (first.month - 1 + m) // 12, (first.month - 1 + m) % 12 + 1
        for sub, amt, cur in [("STREAMLINE SAAS SUBSCRIPTION", 4900, "USD"), ("OFFICE SPACE RENT", 180000, "USD"), ("LICENSE HUB ANNUAL FEE MONTHLY", 2900, "USD")]:
            bank.append(dict(date=date(y, mo, 1), desc=sub, minor=-amt, cur=cur, kind="recurring", inv=None))
        bank.append(dict(date=date(y, mo, 28), desc="BANK MAINTENANCE FEE", minor=-1500, cur="USD", kind="fee", inv=None))
        bank.append(dict(date=date(y, mo, 15), desc="CUSTOMER RECEIPT ACME RETAIL", minor=rng.randint(200000, 900000), cur="USD", kind="income", inv=None))
        for _ in range(rng.randint(3, 6)):
            bank.append(dict(date=date(y, mo, rng.randint(2, 27)), desc=rng.choice(["CARD PURCHASE FUEL", "CARD PURCHASE LUNCH", "ONLINE MARKETPLACE", "COURIER CHARGE", "UTILITY ELECTRIC"]), minor=-rng.randint(900, 38000), cur="USD", kind="misc", inv=None))
    bank.append(dict(date=date(2025, 9, 30), desc="REFUND " + vendors[0]["name"], minor=7300, cur=invoices[0]["currency"], kind="refund", inv=None))
    bank.append(dict(date=date(2025, 10, 7), desc="UNKNOWN CARD MERCHANT", minor=-42000, cur="USD", kind="unmatched", inv=None))
    bank.sort(key=lambda r: (r["date"], r["desc"]))

    with open(os.path.join(root, "bank_statement.csv"), "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh)
        w.writerow(["Date", "Description", "Debit", "Credit", "Currency", "Balance"])
        for r in bank:
            text = money(abs(r["minor"]), r["cur"], "us")
            w.writerow([r["date"].isoformat(), r["desc"], text if r["minor"] < 0 else "", text if r["minor"] > 0 else "", r["cur"], ""])

    # Ground-truth ledger
    by_cur = {}
    for r in bank:
        c = by_cur.setdefault(r["cur"], {"count": 0, "income_minor": 0, "expense_minor": 0})
        c["count"] += 1
        if r["minor"] > 0:
            c["income_minor"] += r["minor"]
        else:
            c["expense_minor"] += -r["minor"]
    inv_by_cur = {}
    for inv in invoices:
        c = inv_by_cur.setdefault(inv["currency"], {"count": 0, "total_minor": 0})
        c["count"] += 1
        c["total_minor"] += inv["total_minor"]

    ledger = {
        "provenance": "synthetic; generated by tests/fixtures/corpus/make_corpus.py; no real customer data",
        "workspace": name,
        "invoice_count": len(invoices),
        "invoices": [{k: v for k, v in inv.items() if k not in ("lines",)} | {"line_count": len(inv["lines"])} for inv in invoices],
        "receipts": receipts,
        "bank_row_count": len(bank),
        "bank_by_currency": by_cur,
        "invoice_totals_by_currency": inv_by_cur,
        "bank_rows": [{"date": r["date"].isoformat(), "description": r["desc"], "minor": r["minor"], "currency": r["cur"], "kind": r["kind"], "invoice": r["inv"]} for r in bank],
        "expect": {
            "documents_needing_review": [inv["number"] for inv in invoices if inv["defects"]],
            "suspected_duplicate_invoice_pair": [invoices[1]["number"], resend["number"]],
            "duplicate_invoice_number": clash["number"],
            "duplicate_payment_invoices": sorted({r["inv"] for r in bank if r["kind"] == "duplicate_payment"}),
            "partial_payment_invoices": sorted({r["inv"] for r in bank if r["kind"] == "partial"}),
            "short_payment_invoices": sorted({r["inv"] for r in bank if r["kind"] == "short_payment"}),
            "unpaid_invoices": sorted(inv["number"] for inv in invoices if inv is not clash and not any(r["inv"] == inv["number"] for r in bank)),
            "unmatched_bank_debit": "UNKNOWN CARD MERCHANT",
            "recurring_must_not_flag": ["STREAMLINE SAAS SUBSCRIPTION", "OFFICE SPACE RENT", "LICENSE HUB ANNUAL FEE MONTHLY"],
        },
    }
    with open(os.path.join(root, "ledger.json"), "w", encoding="utf-8") as fh:
        json.dump(ledger, fh, indent=2)
        fh.write("\n")
    print(f"{name}: {len(invoices)} invoices, {len(receipts)} receipts, {len(bank)} bank rows")


VENDORS_ALPHA = [
    {"name": "NORTHWIND CLOUD LTD", "code": "NWC"},
    {"name": "BLUE HARBOR STATIONERY", "code": "BHS"},
    {"name": "PINE AND OAK CONSULTING", "code": "POC"},
    {"name": "HELIX DATA SYSTEMS GMBH", "code": "HDS"},
    {"name": "MERIDIAN LOGISTICS CO", "code": "MLC"},
]
VENDORS_BETA = [
    {"name": "KESTREL NETWORKS INC", "code": "KNI"},
    {"name": "ORCHARD TRAINING SERVICES", "code": "OTS"},
    {"name": "SUNDARAM SOFTWARE PVT LTD", "code": "SSP"},
    {"name": "RIVERBEND PRINT WORKS", "code": "RPW"},
]

if __name__ == "__main__":
    build_workspace("workspace_alpha", 20260101, VENDORS_ALPHA, ["THE SHRIKS OPERATIONS", "THE SHRIKS EU BRANCH"], ["USD", "EUR", "INR"], 30, 6, 14)
    build_workspace("workspace_beta", 20260202, VENDORS_BETA, ["BETA TRADING PRIVATE LIMITED"], ["USD", "INR"], 13, 4, 6)
