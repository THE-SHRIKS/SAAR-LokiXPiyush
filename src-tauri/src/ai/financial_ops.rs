//! Deterministic financial operations behind the chat assistant.
//!
//! The language model is never the authority for a figure. A question is mapped to a typed,
//! read-only operation (expenses, income, net cash flow, counts, invoice lookup, duplicates,
//! unmatched debits). The operation runs parameterized SQL over every record in the active
//! workspace, with exact integer arithmetic, and the answer text is rendered from that result.
//! Each answer states its scope (period, currency, merchant), reports data that was left out, and
//! cites the records behind it.
//!
//! A question that maps to no operation returns `None`, so the caller can say so truthfully
//! instead of presenting an unverified answer as verified.

use crate::ai::QueryIntent;
use crate::finance::anomaly::AnomalyDetector;
use crate::finance::matching::{InvoiceCandidate, ReconciliationMatcher, TransactionCandidate};
use chrono::NaiveDate;
use regex::Regex;
use rusqlite::{params_from_iter, Connection};
use std::collections::HashSet;
use std::sync::OnceLock;

/// A figure shown in an answer, kept in minor units with its currency.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    pub label: String,
    pub minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedAnswer {
    pub operation: &'static str,
    pub intent: QueryIntent,
    pub text: String,
    pub figures: Vec<Figure>,
    /// Record tags such as `tx:<id>` or `inv:<id>` that support the answer.
    pub citations: Vec<String>,
    /// Human-readable statement of what the figures cover.
    pub scope: String,
}

#[derive(Debug, Clone)]
struct Period {
    from: NaiveDate,
    to: NaiveDate,
    label: String,
}

#[derive(Debug, Clone)]
struct Scope {
    currency: String,
    period: Option<Period>,
    /// Lowercase merchant keyword, matched against transaction descriptions.
    merchant: Option<String>,
}

impl Scope {
    fn describe(&self) -> String {
        let mut parts = vec![format!("currency {}", self.currency)];
        parts.push(match &self.period {
            Some(p) => format!("period {}", p.label),
            None => "all dates".to_string(),
        });
        if let Some(m) = &self.merchant {
            parts.push(format!("merchant matching \"{}\"", m));
        }
        parts.join(", ")
    }
}

// ---------------------------------------------------------------- formatting

fn group_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Formats minor units exactly, e.g. `-1,234.56 USD`. JPY has no minor unit.
pub fn fmt_money(minor: i64, currency: &str) -> String {
    let sign = if minor < 0 { "-" } else { "" };
    let abs = minor.unsigned_abs();
    if currency.eq_ignore_ascii_case("JPY") {
        format!("{}{} {}", sign, group_thousands(abs), currency)
    } else {
        format!("{}{}.{:02} {}", sign, group_thousands(abs / 100), abs % 100, currency)
    }
}

// ---------------------------------------------------------------- question parsing

fn month_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:t(?:ember)?)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\.?,?\s+(\d{4})\b",
        )
        .unwrap()
    })
}

fn quarter_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bq([1-4])\s*(?:of\s*)?(\d{4})\b").unwrap())
}

fn year_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // A bare year; the surrounding classes keep "INV-2025-0003" from counting as a year.
    RE.get_or_init(|| Regex::new(r"(?:^|[^\w-])((?:19|20)\d{2})(?:$|[^\w-])").unwrap())
}

fn currency_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b(USD|EUR|INR|GBP|JPY)\b").unwrap())
}

fn invoice_number_token() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[A-Za-z0-9]+(?:-[A-Za-z0-9]+)+").unwrap())
}

fn month_number(name: &str) -> Option<u32> {
    let n = name.to_lowercase();
    let n = n.trim_end_matches('.');
    Some(match &n[..3.min(n.len())] {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    })
}

fn month_bounds(year: i32, month: u32) -> Option<(NaiveDate, NaiveDate)> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    Some((first, next.pred_opt()?))
}

fn month_label(year: i32, month: u32) -> String {
    const NAMES: [&str; 12] = [
        "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
        "November", "December",
    ];
    format!("{} {}", NAMES[(month - 1) as usize], year)
}

fn parse_period(query: &str) -> Option<Period> {
    if let Some(c) = month_pattern().captures(query) {
        let month = month_number(&c[1])?;
        let year: i32 = c[2].parse().ok()?;
        let (from, to) = month_bounds(year, month)?;
        return Some(Period { from, to, label: month_label(year, month) });
    }
    if let Some(c) = quarter_pattern().captures(query) {
        let q: u32 = c[1].parse().ok()?;
        let year: i32 = c[2].parse().ok()?;
        let (from, _) = month_bounds(year, (q - 1) * 3 + 1)?;
        let (_, to) = month_bounds(year, q * 3)?;
        return Some(Period { from, to, label: format!("Q{} {}", q, year) });
    }
    if let Some(c) = year_pattern().captures(query) {
        let year: i32 = c[1].parse().ok()?;
        return Some(Period {
            from: NaiveDate::from_ymd_opt(year, 1, 1)?,
            to: NaiveDate::from_ymd_opt(year, 12, 31)?,
            label: format!("{}", year),
        });
    }
    None
}

fn parse_currency(query: &str, base: &str) -> String {
    if let Some(c) = currency_pattern().captures(query) {
        return c[1].to_uppercase();
    }
    let q = query.to_lowercase();
    for (word, code) in [("euro", "EUR"), ("rupee", "INR"), ("dollar", "USD"), ("pound", "GBP"), ("yen", "JPY")] {
        if q.contains(word) {
            return code.to_string();
        }
    }
    base.to_string()
}

/// Words that are common in questions or in generic bank descriptions and are never a merchant.
const NOT_A_MERCHANT: &[&str] = &[
    "what", "which", "total", "income", "expense", "expenses", "spend", "spent", "spending", "paid", "payment",
    "payments", "invoice", "invoices", "vendor", "vendors", "bank", "card", "online", "purchase", "customer",
    "receipt", "receipts", "month", "year", "from", "with", "that", "have", "were", "there", "this", "cash",
    "flow", "net", "show", "list", "duplicate", "duplicates", "unmatched", "debit", "debits", "credit",
    "credits", "currency", "transactions", "transaction", "records", "workspace", "many", "much", "does",
    "exist", "about", "more", "than", "for", "all", "the", "and", "largest", "biggest", "top", "refund",
    "refunds", "fee", "fees", "maintenance", "utility", "courier", "marketplace", "sundry", "recurring",
];

/// First word of a description that identifies a merchant (letters, at least 3 characters, not a number).
fn merchant_key(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .find(|w| w.chars().count() >= 3 && !w.chars().all(|c| c.is_ascii_digit()))
        .map(|w| w.to_lowercase())
}

/// A merchant keyword from the question that also occurs as a merchant in this workspace.
fn detect_merchant(conn: &Connection, ws: &str, query: &str) -> Result<Option<String>, String> {
    let q = query.to_lowercase();
    let tokens: Vec<&str> = q
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= 4 && !NOT_A_MERCHANT.contains(t))
        .collect();
    if tokens.is_empty() {
        return Ok(None);
    }
    let mut known: HashSet<String> = HashSet::new();
    let mut stmt = conn
        .prepare("SELECT DISTINCT description FROM transactions WHERE workspace_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([ws], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    for r in rows {
        if let Some(k) = merchant_key(&r.map_err(|e| e.to_string())?) {
            known.insert(k);
        }
    }
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT i.vendor_name FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([ws], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    for r in rows {
        if let Some(k) = merchant_key(&r.map_err(|e| e.to_string())?) {
            known.insert(k);
        }
    }
    Ok(tokens.iter().find(|t| known.contains(**t)).map(|t| t.to_string()))
}

// ---------------------------------------------------------------- SQL helpers

/// Builds the WHERE clause and bound parameters for a scope. All values are bound, never inlined.
fn scope_filter(ws: &str, scope: &Scope) -> (String, Vec<String>) {
    let mut sql = String::from("workspace_id = ?1 AND currency = ?2");
    let mut params = vec![ws.to_string(), scope.currency.clone()];
    if let Some(p) = &scope.period {
        params.push(p.from.format("%Y-%m-%d").to_string());
        sql.push_str(&format!(" AND posted_date >= ?{}", params.len()));
        params.push(p.to.format("%Y-%m-%d").to_string());
        sql.push_str(&format!(" AND posted_date <= ?{}", params.len()));
    }
    if let Some(m) = &scope.merchant {
        params.push(format!("%{}%", m));
        sql.push_str(&format!(" AND lower(description) LIKE ?{}", params.len()));
    }
    (sql, params)
}

struct Totals {
    income: i64,
    expense: i64,
    credits: i64,
    debits: i64,
}

fn totals(conn: &Connection, ws: &str, scope: &Scope) -> Result<Totals, String> {
    let (filter, params) = scope_filter(ws, scope);
    let sql = format!(
        "SELECT COALESCE(SUM(CASE WHEN amount_minor > 0 THEN amount_minor ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN amount_minor < 0 THEN -amount_minor ELSE 0 END), 0),
                COUNT(CASE WHEN amount_minor > 0 THEN 1 END),
                COUNT(CASE WHEN amount_minor < 0 THEN 1 END)
         FROM transactions WHERE {}",
        filter
    );
    conn.query_row(&sql, params_from_iter(params.iter()), |r| {
        Ok(Totals { income: r.get(0)?, expense: r.get(1)?, credits: r.get(2)?, debits: r.get(3)? })
    })
    .map_err(|e| e.to_string())
}

/// Transactions in the same period (and merchant) but another currency: reported, never merged.
fn other_currency_note(conn: &Connection, ws: &str, scope: &Scope) -> Result<Option<String>, String> {
    let mut sql = String::from(
        "SELECT currency, COUNT(*), COALESCE(SUM(ABS(amount_minor)), 0) FROM transactions
         WHERE workspace_id = ?1 AND currency <> ?2",
    );
    let mut params = vec![ws.to_string(), scope.currency.clone()];
    if let Some(p) = &scope.period {
        params.push(p.from.format("%Y-%m-%d").to_string());
        sql.push_str(&format!(" AND posted_date >= ?{}", params.len()));
        params.push(p.to.format("%Y-%m-%d").to_string());
        sql.push_str(&format!(" AND posted_date <= ?{}", params.len()));
    }
    if let Some(m) = &scope.merchant {
        params.push(format!("%{}%", m));
        sql.push_str(&format!(" AND lower(description) LIKE ?{}", params.len()));
    }
    sql.push_str(" GROUP BY currency ORDER BY currency");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(params.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
        })
        .map_err(|e| e.to_string())?;
    let mut parts = Vec::new();
    for r in rows {
        let (cur, n, gross) = r.map_err(|e| e.to_string())?;
        parts.push(format!("{} transaction{} in {} ({} gross)", n, if n == 1 { "" } else { "s" }, cur, fmt_money(gross, &cur)));
    }
    Ok(if parts.is_empty() {
        None
    } else {
        Some(format!(
            "Not included, because they are in a different currency and no exchange rate is recorded: {}.",
            parts.join("; ")
        ))
    })
}

fn data_range(conn: &Connection, ws: &str) -> Result<Option<(String, String)>, String> {
    conn.query_row(
        "SELECT MIN(posted_date), MAX(posted_date) FROM transactions WHERE workspace_id = ?1",
        [ws],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)),
    )
    .map(|(a, b)| a.zip(b))
    .map_err(|e| e.to_string())
}

fn evidence(conn: &Connection, ws: &str, scope: &Scope, debits_only: Option<bool>) -> Result<Vec<String>, String> {
    let (filter, params) = scope_filter(ws, scope);
    let direction = match debits_only {
        Some(true) => " AND amount_minor < 0",
        Some(false) => " AND amount_minor > 0",
        None => "",
    };
    let sql = format!(
        "SELECT id FROM transactions WHERE {}{} ORDER BY ABS(amount_minor) DESC, id LIMIT 5",
        filter, direction
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(params.iter()), |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(format!("tx:{}", r.map_err(|e| e.to_string())?));
    }
    Ok(out)
}

/// Appends the data coverage and the other-currency note shared by the summary operations.
fn with_coverage(conn: &Connection, ws: &str, scope: &Scope, mut text: String, n_in_scope: i64) -> Result<String, String> {
    if let Some(note) = other_currency_note(conn, ws, scope)? {
        text.push(' ');
        text.push_str(&note);
    }
    if n_in_scope == 0 {
        if let Some((first, last)) = data_range(conn, ws)? {
            text.push_str(&format!(" Your records cover {} to {}.", first, last));
        }
    }
    Ok(text)
}

// ---------------------------------------------------------------- operations

fn expenses(conn: &Connection, ws: &str, scope: &Scope) -> Result<VerifiedAnswer, String> {
    let t = totals(conn, ws, scope)?;
    let text = if t.debits == 0 {
        format!("No expenses (debits) are recorded in {}.", scope.describe())
    } else {
        format!(
            "Expenses ({}): {} across {} debit transaction{}.",
            scope.describe(),
            fmt_money(t.expense, &scope.currency),
            t.debits,
            if t.debits == 1 { "" } else { "s" }
        )
    };
    Ok(VerifiedAnswer {
        operation: "get_expenses",
        intent: QueryIntent::FinancialSummary,
        text: with_coverage(conn, ws, scope, text, t.debits)?,
        figures: vec![Figure { label: "expenses".into(), minor: t.expense, currency: scope.currency.clone() }],
        citations: evidence(conn, ws, scope, Some(true))?,
        scope: scope.describe(),
    })
}

fn income(conn: &Connection, ws: &str, scope: &Scope) -> Result<VerifiedAnswer, String> {
    let t = totals(conn, ws, scope)?;
    let text = if t.credits == 0 {
        format!("No income (credits) is recorded in {}.", scope.describe())
    } else {
        format!(
            "Income ({}): {} across {} credit transaction{}.",
            scope.describe(),
            fmt_money(t.income, &scope.currency),
            t.credits,
            if t.credits == 1 { "" } else { "s" }
        )
    };
    Ok(VerifiedAnswer {
        operation: "get_income",
        intent: QueryIntent::FinancialSummary,
        text: with_coverage(conn, ws, scope, text, t.credits)?,
        figures: vec![Figure { label: "income".into(), minor: t.income, currency: scope.currency.clone() }],
        citations: evidence(conn, ws, scope, Some(false))?,
        scope: scope.describe(),
    })
}

fn net_cash_flow(conn: &Connection, ws: &str, scope: &Scope) -> Result<VerifiedAnswer, String> {
    let t = totals(conn, ws, scope)?;
    let n = t.credits + t.debits;
    let net = t.income - t.expense;
    let text = if n == 0 {
        format!("No transactions are recorded in {}.", scope.describe())
    } else {
        format!(
            "Net cash flow ({}): {}. Income {} from {} credit{}, expenses {} from {} debit{}.",
            scope.describe(),
            fmt_money(net, &scope.currency),
            fmt_money(t.income, &scope.currency),
            t.credits,
            if t.credits == 1 { "" } else { "s" },
            fmt_money(t.expense, &scope.currency),
            t.debits,
            if t.debits == 1 { "" } else { "s" }
        )
    };
    Ok(VerifiedAnswer {
        operation: "get_cash_flow",
        intent: QueryIntent::FinancialSummary,
        text: with_coverage(conn, ws, scope, text, n)?,
        figures: vec![
            Figure { label: "net".into(), minor: net, currency: scope.currency.clone() },
            Figure { label: "income".into(), minor: t.income, currency: scope.currency.clone() },
            Figure { label: "expenses".into(), minor: t.expense, currency: scope.currency.clone() },
        ],
        citations: evidence(conn, ws, scope, None)?,
        scope: scope.describe(),
    })
}

fn top_expenses(conn: &Connection, ws: &str, scope: &Scope) -> Result<VerifiedAnswer, String> {
    let (filter, params) = scope_filter(ws, scope);
    let sql = format!(
        "SELECT id, posted_date, description, amount_minor FROM transactions
         WHERE {} AND amount_minor < 0 ORDER BY amount_minor ASC, id LIMIT 5",
        filter
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(params.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?))
        })
        .map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    let mut figures = Vec::new();
    let mut citations = Vec::new();
    for r in rows {
        let (id, date, desc, minor) = r.map_err(|e| e.to_string())?;
        lines.push(format!("{} {} - {}", date, desc, fmt_money(-minor, &scope.currency)));
        figures.push(Figure { label: desc, minor: -minor, currency: scope.currency.clone() });
        citations.push(format!("tx:{}", id));
    }
    let n = lines.len() as i64;
    let text = if lines.is_empty() {
        format!("No expenses (debits) are recorded in {}.", scope.describe())
    } else {
        format!("Largest expenses ({}): {}.", scope.describe(), lines.join("; "))
    };
    Ok(VerifiedAnswer {
        operation: "get_transactions",
        intent: QueryIntent::TransactionSearch,
        text: with_coverage(conn, ws, scope, text, n)?,
        figures,
        citations,
        scope: scope.describe(),
    })
}

fn counts(conn: &Connection, ws: &str, query: &str, merchant: Option<&str>) -> Result<VerifiedAnswer, String> {
    let q = query.to_lowercase();
    let like = format!("%{}%", merchant.unwrap_or(""));
    let n = |sql: &str, with_merchant: bool| -> Result<i64, String> {
        if with_merchant {
            conn.query_row(sql, [ws, like.as_str()], |r| r.get(0)).map_err(|e| e.to_string())
        } else {
            conn.query_row(sql, [ws], |r| r.get(0)).map_err(|e| e.to_string())
        }
    };
    let m = merchant.is_some();
    let (label, count) = if q.contains("invoice") || q.contains("bill") {
        if m {
            ("invoices", n("SELECT COUNT(*) FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = ?1 AND lower(i.vendor_name) LIKE ?2", true)?)
        } else {
            ("invoices", n("SELECT COUNT(*) FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = ?1", false)?)
        }
    } else if q.contains("document") || q.contains("file") || q.contains("receipt") {
        ("documents", n("SELECT COUNT(*) FROM documents WHERE workspace_id = ?1", false)?)
    } else if m {
        ("transactions", n("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1 AND lower(description) LIKE ?2", true)?)
    } else {
        ("transactions", n("SELECT COUNT(*) FROM transactions WHERE workspace_id = ?1", false)?)
    };
    let scope_text = match merchant {
        Some(k) => format!("entire workspace, merchant matching \"{}\"", k),
        None => "entire workspace".to_string(),
    };
    Ok(VerifiedAnswer {
        operation: "count_records",
        intent: QueryIntent::GeneralFinancial,
        text: format!("There {} {} {} ({}).", if count == 1 { "is" } else { "are" }, count, label, scope_text),
        figures: vec![],
        citations: vec![],
        scope: scope_text,
    })
}

/// Invoices from vendors matching a keyword: every match is totalled per currency in SQL, and the
/// most recent few are listed.
fn vendor_invoices(conn: &Connection, ws: &str, merchant: &str) -> Result<VerifiedAnswer, String> {
    let like = format!("%{}%", merchant);
    let mut stmt = conn
        .prepare(
            "SELECT i.currency, COUNT(*), COALESCE(SUM(i.total_minor), 0)
             FROM invoices i JOIN documents d ON d.id = i.document_id
             WHERE d.workspace_id = ?1 AND lower(i.vendor_name) LIKE ?2
             GROUP BY i.currency ORDER BY i.currency",
        )
        .map_err(|e| e.to_string())?;
    let per_currency = stmt
        .query_map([ws, like.as_str()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let scope = format!("entire workspace, vendor matching \"{}\"", merchant);
    if per_currency.is_empty() {
        return Ok(VerifiedAnswer {
            operation: "get_invoice_details",
            intent: QueryIntent::DocumentQuery,
            text: format!("No invoices from a vendor matching \"{}\" are recorded in this workspace.", merchant),
            figures: vec![],
            citations: vec![],
            scope,
        });
    }
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
             FROM invoices i JOIN documents d ON d.id = i.document_id
             WHERE d.workspace_id = ?1 AND lower(i.vendor_name) LIKE ?2
             ORDER BY i.invoice_date DESC, i.id LIMIT 5",
        )
        .map_err(|e| e.to_string())?;
    let recent = stmt
        .query_map([ws, like.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, Option<String>>(3)?, r.get::<_, i64>(4)?, r.get::<_, String>(5)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let total_count: i64 = per_currency.iter().map(|c| c.1).sum();
    let totals: Vec<String> = per_currency
        .iter()
        .map(|(cur, n, sum)| format!("{} in {} ({} invoice{})", fmt_money(*sum, cur), cur, n, if *n == 1 { "" } else { "s" }))
        .collect();
    let listed: Vec<String> = recent
        .iter()
        .map(|(_, vendor, number, date, total, cur)| {
            format!(
                "{} {} dated {}, {}",
                vendor,
                number.as_deref().unwrap_or("(no number)"),
                date.as_deref().unwrap_or("unknown date"),
                fmt_money(*total, cur)
            )
        })
        .collect();
    let text = format!(
        "{} invoice{} from vendors matching \"{}\", totalling {}. {}{}",
        total_count,
        if total_count == 1 { "" } else { "s" },
        merchant,
        totals.join("; "),
        if total_count > recent.len() as i64 { "Most recent: " } else { "Listed: " },
        listed.join("; ")
    );
    Ok(VerifiedAnswer {
        operation: "get_invoice_details",
        intent: QueryIntent::DocumentQuery,
        text: format!("{}.", text),
        figures: per_currency
            .iter()
            .map(|(cur, _, sum)| Figure { label: "invoices total".into(), minor: *sum, currency: cur.clone() })
            .collect(),
        citations: recent.iter().map(|r| format!("inv:{}", r.0)).collect(),
        scope,
    })
}

fn load_matching_inputs(conn: &Connection, ws: &str) -> Result<(Vec<InvoiceCandidate>, Vec<TransactionCandidate>), String> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency
             FROM invoices i JOIN documents d ON d.id = i.document_id WHERE d.workspace_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let invoices = stmt
        .query_map([ws], |r| {
            Ok(InvoiceCandidate {
                id: r.get(0)?,
                vendor_name: r.get(1)?,
                invoice_number: r.get(2)?,
                invoice_date: r.get(3)?,
                total_minor: r.get(4)?,
                currency: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, posted_date, description, amount_minor, currency, external_ref FROM transactions WHERE workspace_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let txs = stmt
        .query_map([ws], |r| {
            Ok(TransactionCandidate {
                id: r.get(0)?,
                posted_date: r.get(1)?,
                description: r.get(2)?,
                amount_minor: r.get(3)?,
                currency: r.get(4)?,
                external_ref: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok((invoices, txs))
}

fn duplicates(conn: &Connection, ws: &str) -> Result<VerifiedAnswer, String> {
    let (invoices, txs) = load_matching_inputs(conn, ws)?;
    let kinds = ["POTENTIAL_DUPLICATE_PAYMENT", "DUPLICATE_TRANSACTION", "DUPLICATE_INVOICE", "SUSPECTED_DUPLICATE_BILLING"];
    let found: Vec<_> = AnomalyDetector::detect_anomalies(&invoices, &txs)
        .into_iter()
        .filter(|a| kinds.contains(&a.anomaly_type.as_str()))
        .collect();
    let text = if found.is_empty() {
        "No suspected duplicate payments, duplicate transactions or duplicate invoices were found in this workspace.".to_string()
    } else {
        let shown: Vec<String> = found.iter().take(5).map(|a| format!("[{}] {}", a.anomaly_type, a.description)).collect();
        format!(
            "{} suspected duplicate{} found. Review each one; nothing has been changed. {}{}",
            found.len(),
            if found.len() == 1 { "" } else { "s" },
            shown.join(" "),
            if found.len() > 5 { format!(" (and {} more)", found.len() - 5) } else { String::new() }
        )
    };
    let citations = found
        .iter()
        .take(5)
        .map(|a| format!("{}:{}", if a.entity_type == "INVOICE" { "inv" } else { "tx" }, a.entity_id))
        .collect();
    Ok(VerifiedAnswer {
        operation: "find_duplicate_payments",
        intent: QueryIntent::ReconciliationStatus,
        text,
        figures: vec![],
        citations,
        scope: "entire workspace".to_string(),
    })
}

fn unmatched(conn: &Connection, ws: &str, scope: &Scope) -> Result<VerifiedAnswer, String> {
    let (invoices, txs) = load_matching_inputs(conn, ws)?;
    let linked: HashSet<String> = ReconciliationMatcher::match_records(&invoices, &txs)
        .into_iter()
        .map(|l| l.transaction_id)
        .collect();
    let mut open: Vec<&TransactionCandidate> = txs
        .iter()
        .filter(|t| t.amount_minor < 0 && t.currency == scope.currency && !linked.contains(&t.id))
        .filter(|t| match &scope.period {
            Some(p) => t.posted_date >= p.from.format("%Y-%m-%d").to_string() && t.posted_date <= p.to.format("%Y-%m-%d").to_string(),
            None => true,
        })
        .filter(|t| scope.merchant.as_ref().map_or(true, |m| t.description.to_lowercase().contains(m.as_str())))
        .collect();
    open.sort_by(|a, b| a.amount_minor.cmp(&b.amount_minor).then(a.id.cmp(&b.id)));
    let total: i64 = open.iter().map(|t| -t.amount_minor).sum();
    let text = if open.is_empty() {
        format!("Every debit in {} is matched to an invoice under the current automatic matching.", scope.describe())
    } else {
        let shown: Vec<String> = open
            .iter()
            .take(5)
            .map(|t| format!("{} {} - {}", t.posted_date, t.description, fmt_money(-t.amount_minor, &t.currency)))
            .collect();
        format!(
            "{} debit{} in {} {} no matching invoice under the current automatic matching, totalling {}. Largest: {}.",
            open.len(),
            if open.len() == 1 { "" } else { "s" },
            scope.describe(),
            if open.len() == 1 { "has" } else { "have" },
            fmt_money(total, &scope.currency),
            shown.join("; ")
        )
    };
    Ok(VerifiedAnswer {
        operation: "get_unmatched_transactions",
        intent: QueryIntent::ReconciliationStatus,
        text,
        figures: vec![Figure { label: "unmatched debits".into(), minor: total, currency: scope.currency.clone() }],
        citations: open.iter().take(5).map(|t| format!("tx:{}", t.id)).collect(),
        scope: scope.describe(),
    })
}

fn invoice_details(conn: &Connection, ws: &str, numbers: &[String]) -> Result<Option<VerifiedAnswer>, String> {
    let mut found: Vec<(String, String, String, Option<String>, i64, String, String)> = Vec::new();
    for number in numbers {
        let mut stmt = conn
            .prepare(
                "SELECT i.id, i.vendor_name, i.invoice_number, i.invoice_date, i.total_minor, i.currency, i.review_state
                 FROM invoices i JOIN documents d ON d.id = i.document_id
                 WHERE d.workspace_id = ?1 AND lower(i.invoice_number) = lower(?2) ORDER BY i.id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([ws, number.as_str()], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            found.push(r.map_err(|e| e.to_string())?);
        }
    }
    if found.is_empty() {
        return Ok(Some(VerifiedAnswer {
            operation: "get_invoice_details",
            intent: QueryIntent::DocumentQuery,
            text: format!(
                "No invoice numbered {} is recorded in this workspace.",
                numbers.iter().map(|n| format!("\"{}\"", n)).collect::<Vec<_>>().join(" or ")
            ),
            figures: vec![],
            citations: vec![],
            scope: "entire workspace".to_string(),
        }));
    }
    let mut lines = Vec::new();
    let mut figures = Vec::new();
    for (_, vendor, number, date, total, cur, state) in &found {
        lines.push(format!(
            "{} from {}, dated {}, total {} ({})",
            number,
            vendor,
            date.as_deref().unwrap_or("unknown date"),
            fmt_money(*total, cur),
            state.to_lowercase()
        ));
        figures.push(Figure { label: number.clone(), minor: *total, currency: cur.clone() });
    }
    let mut text = lines.join("; ");
    text.push('.');
    if found.len() > 1 {
        let same = found.iter().all(|f| f.1 == found[0].1 && f.4 == found[0].4 && f.5 == found[0].5);
        if numbers.len() == 1 {
            text.push_str(&format!(" The number is used by {} invoices, which should be unique.", found.len()));
        } else if same {
            text.push_str(" These records have the same vendor, currency and total; one may be a re-sent copy of the other, so review before paying both.");
        } else {
            text.push_str(" These records differ in vendor, currency or total.");
        }
    }
    Ok(Some(VerifiedAnswer {
        operation: "get_invoice_details",
        intent: QueryIntent::DocumentQuery,
        text,
        figures,
        citations: found.iter().map(|f| format!("inv:{}", f.0)).collect(),
        scope: "entire workspace".to_string(),
    }))
}

// ---------------------------------------------------------------- entry point

/// Answers a question with a verified, code-computed result, or returns `None` when no supported
/// operation applies. Never calls a language model.
pub fn try_answer(conn: &Connection, ws: &str, base_currency: &str, query: &str) -> Result<Option<VerifiedAnswer>, String> {
    let q = query.to_lowercase();

    // Invoice numbers in the question (tokens such as "INV-2025-0003" that contain a digit).
    let numbers: Vec<String> = invoice_number_token()
        .find_iter(query)
        .map(|m| m.as_str().to_string())
        .filter(|t| t.chars().any(|c| c.is_ascii_digit()) && t.chars().filter(|c| c.is_alphabetic()).count() >= 2)
        .collect();
    if !numbers.is_empty() {
        return invoice_details(conn, ws, &numbers);
    }

    // Forecasts and advice are not computed from records, so no operation answers them.
    const NOT_COMPUTABLE: &[&str] = &["forecast", "predict", "next year", "next month", "next quarter", "will i", "will we", "should i", "should we", "budget for", "expected to"];
    if NOT_COMPUTABLE.iter().any(|k| q.contains(k)) {
        return Ok(None);
    }

    if q.contains("duplicate") || q.contains("paid twice") || q.contains("double pay") {
        return duplicates(conn, ws).map(Some);
    }

    let scope = Scope {
        currency: parse_currency(query, base_currency),
        period: parse_period(query),
        merchant: detect_merchant(conn, ws, query)?,
    };

    // Whole-word matching, so "payroll" or "payee" never triggers an expense answer.
    let words: HashSet<&str> = q.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let has = |ws: &[&str]| ws.iter().any(|w| words.contains(w));

    let no_invoice = q.contains("no invoice")
        || q.contains("without an invoice")
        || q.contains("without invoice")
        || q.contains("missing invoice")
        || q.contains("not matched")
        || q.contains("not linked")
        || (q.contains("matching invoice") && has(&["no", "without", "lack", "lacking", "missing", "not"]));
    if has(&["unmatched", "unreconciled", "unlinked"]) || no_invoice {
        return unmatched(conn, ws, &scope).map(Some);
    }
    if q.contains("how many") {
        return counts(conn, ws, query, scope.merchant.as_deref()).map(Some);
    }
    // "invoice total for <vendor>", "bills from <vendor>": invoices of a named vendor.
    if has(&["invoice", "invoices", "bill", "bills"]) {
        if let Some(m) = scope.merchant.as_deref() {
            return vendor_invoices(conn, ws, m).map(Some);
        }
    }
    if q.contains("cash flow") || has(&["net", "cashflow", "savings", "profit", "surplus"]) {
        return net_cash_flow(conn, ws, &scope).map(Some);
    }
    if has(&["largest", "biggest", "highest", "top"]) {
        return top_expenses(conn, ws, &scope).map(Some);
    }
    if has(&["income", "revenue", "received", "earned", "deposit", "deposits", "credits"]) {
        return income(conn, ws, &scope).map(Some);
    }
    if has(&[
        "expense", "expenses", "spend", "spent", "spending", "cost", "costs", "paid", "pay", "pays", "paying",
        "payment", "payments", "outgoing", "charged", "debits",
    ]) {
        return expenses(conn, ws, &scope).map(Some);
    }
    Ok(None)
}

#[cfg(test)]
mod vendor_tests {
    use super::*;
    use crate::db::Database;

    fn add_invoice(db: &Database, n: &str, vendor: &str, date: &str, total: i64, cur: &str) {
        let conn = db.conn_ref();
        conn.execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, doc_type, page_count, status, imported_at)
             VALUES (?1, 'ws', ?1, 1, ?1, 'PDF', 1, 'READY', datetime('now'))",
            [n],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO invoices (id, document_id, vendor_name, invoice_number, invoice_date, currency, total_minor, tax_minor, review_state)
             VALUES (?1, ?1, ?2, ?1, ?3, ?4, ?5, 0, 'UNREVIEWED')",
            rusqlite::params![n, vendor, date, cur, total],
        )
        .unwrap();
    }

    fn db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.create_workspace("ws", "WS", "USD", &[1u8; 16], &[2u8; 32]).unwrap();
        add_invoice(&db, "CH-001", "CloudHosting Corp", "2026-09-01", 12_000, "USD");
        add_invoice(&db, "CH-002", "CloudHosting Corp", "2026-10-01", 12_000, "USD");
        add_invoice(&db, "CH-003", "CloudHosting Corp", "2026-10-15", 99_95, "EUR");
        add_invoice(&db, "GH-001", "GitHub Inc", "2026-10-02", 4_200, "USD");
        for (i, (d, desc, m)) in [("2026-09-02", "CLOUDHOSTING CORP INV CH-001", -12_000i64), ("2026-09-05", "GITHUB SUBSCRIPTION", -4_200)]
            .iter()
            .enumerate()
        {
            db.conn_ref()
                .execute(
                    "INSERT INTO transactions (id, workspace_id, posted_date, description, amount_minor, currency, review_state)
                     VALUES (?1, 'ws', ?2, ?3, ?4, 'USD', 'UNREVIEWED')",
                    rusqlite::params![format!("t{}", i), d, desc, m],
                )
                .unwrap();
        }
        db
    }

    #[test]
    fn test_vendor_invoice_total_is_summed_per_currency_over_all_matches() {
        let db = db();
        let a = try_answer(db.conn_ref(), "ws", "USD", "What was the invoice total for CloudHosting Corp?").unwrap().unwrap();
        let usd = a.figures.iter().find(|f| f.currency == "USD").unwrap();
        let eur = a.figures.iter().find(|f| f.currency == "EUR").unwrap();
        assert_eq!(usd.minor, 24_000, "two USD invoices of 120.00");
        assert_eq!(eur.minor, 9_995);
        assert!(a.text.contains("3 invoices") && a.text.contains("240.00 USD") && a.text.contains("99.95 EUR"), "{}", a.text);
        assert!(!a.text.contains("GitHub"), "another vendor must not leak into the answer");
    }

    #[test]
    fn test_unknown_vendor_gets_no_invented_total() {
        let db = db();
        // "Zephyr" is not a merchant in this workspace, so there is no vendor scope; the question
        // then has no supported operation and no figure is produced.
        let a = try_answer(db.conn_ref(), "ws", "USD", "What was the invoice total for Zephyr Corp?").unwrap();
        assert!(a.is_none());
    }

    #[test]
    fn test_count_respects_a_named_vendor() {
        let db = db();
        let a = try_answer(db.conn_ref(), "ws", "USD", "How many invoices do I have from CloudHosting?").unwrap().unwrap();
        assert!(a.text.contains("There are 3 invoices"), "{}", a.text);
        let all = try_answer(db.conn_ref(), "ws", "USD", "How many invoices are there?").unwrap().unwrap();
        assert!(all.text.contains("There are 4 invoices"), "{}", all.text);
    }
}
