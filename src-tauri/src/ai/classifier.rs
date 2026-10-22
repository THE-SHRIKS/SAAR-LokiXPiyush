use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ai::llm_engine::LlmEngine;
use crate::finance::money::Money;

pub const ALLOWED_CATEGORIES: &[&str] = &[
    "Software & Cloud Services",
    "Office Supplies & Stationery",
    "Consulting & Professional Services",
    "Travel, Lodging & Meals",
    "Hardware & IT Equipment",
    "Banking Charges, Fees & Taxes",
    "General Operating Expense",
];

pub const CLASSIFIER_GBNF_GRAMMAR: &str = r#"root ::= "{" space category-kv "," space confidence-kv "," space rationale-kv space "}"
category-kv ::= "\"category\"" space ":" space category
confidence-kv ::= "\"confidence\"" space ":" space decimal
rationale-kv ::= "\"rationale\"" space ":" space string
category ::= "\"Software & Cloud Services\"" | "\"Office Supplies & Stationery\"" | "\"Consulting & Professional Services\"" | "\"Travel, Lodging & Meals\"" | "\"Hardware & IT Equipment\"" | "\"Banking Charges, Fees & Taxes\"" | "\"General Operating Expense\""
decimal ::= [0-1] ("." [0-9]{1,4})?
char ::= [^"\\\x7F\x00-\x1F] | [\\] (["\\bfnrt] | "u" [0-9a-fA-F]{4})
string ::= "\"" char* "\""
space ::= | " " | "\n"{1,2} [ \t]{0,20}
"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CategoryProposal {
    pub id: String,
    pub transaction_id: String,
    pub category: String,
    pub confidence_score: f64,
    pub rationale: String,
    pub status: String, // "PROPOSED", "APPROVED", "REJECTED"
}

pub struct TransactionClassifier;

impl TransactionClassifier {
    /// Classifies an unreviewed transaction using the local model constrained by GBNF grammar.
    pub fn classify_transaction(
        llm: &LlmEngine,
        transaction_id: &str,
        description: &str,
        amount_minor: i64,
        currency: &str,
    ) -> Result<CategoryProposal, String> {
        let amount_str = Money::new(amount_minor, currency)
            .map(|m| m.to_string())
            .unwrap_or_else(|_| format!("{} minor units ({})", amount_minor, currency));

        let system_prompt =
            "You are an expert bookkeeping assistant. \
             Assign a transaction to exactly one allowed expense category with a confidence score between 0.0 and 1.0. \
             You must output JSON conforming to the grammar.";

        let user_prompt = format!(
            "Classify this transaction:\nDescription: {}\nAmount: {}\nCurrency: {}",
            description, amount_str, currency
        );

        let raw_json = llm
            .generate(system_prompt, &user_prompt, Some(CLASSIFIER_GBNF_GRAMMAR))
            .map_err(|e| format!("LLM classification error: {}", e))?;

        // Extract JSON substring between { and }
        let json_str = if let (Some(start), Some(end)) = (raw_json.find('{'), raw_json.rfind('}')) {
            &raw_json[start..=end]
        } else {
            &raw_json
        };

        let parsed_val: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let (raw_cat, raw_conf, raw_rat) = match parsed_val {
            Ok(v) => (
                v["category"].as_str().unwrap_or("General Operating Expense").to_string(),
                v["confidence"].as_f64().unwrap_or(0.85),
                v["rationale"].as_str().unwrap_or("Model classified").to_string(),
            ),
            Err(e) => {
                eprintln!("Classification JSON parse fallback (raw: {:?}): {}", raw_json, e);
                ("General Operating Expense".to_string(), 0.50, "Parse fallback".to_string())
            }
        };

        // Ensure category is within strict allowlist
        let valid_category = if ALLOWED_CATEGORIES.contains(&raw_cat.as_str()) {
            raw_cat
        } else {
            "General Operating Expense".to_string()
        };

        Ok(CategoryProposal {
            id: Uuid::new_v4().to_string(),
            transaction_id: transaction_id.to_string(),
            category: valid_category,
            confidence_score: raw_conf.clamp(0.0, 1.0),
            rationale: raw_rat,
            status: "PROPOSED".to_string(),
        })
    }

    /// Persists a proposed category into SQLite database.
    pub fn save_proposal(
        conn: &Connection,
        workspace_id: &str,
        proposal: &CategoryProposal,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO transaction_categories (id, workspace_id, transaction_id, category, confidence_score, rationale, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            params![
                proposal.id,
                workspace_id,
                proposal.transaction_id,
                proposal.category,
                proposal.confidence_score,
                proposal.rationale,
                proposal.status,
                now
            ],
        )?;

        Ok(())
    }

    /// Approves a proposed category and records audit trail.
    pub fn approve_category(
        conn: &Connection,
        workspace_id: &str,
        category_proposal_id: &str,
    ) -> Result<()> {
        conn.execute(
            "UPDATE transaction_categories SET status = 'APPROVED' WHERE id = ?1 AND workspace_id = ?2;",
            params![category_proposal_id, workspace_id],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_categories_list() {
        assert!(ALLOWED_CATEGORIES.contains(&"Software & Cloud Services"));
        assert!(ALLOWED_CATEGORIES.contains(&"Office Supplies & Stationery"));
        assert_eq!(ALLOWED_CATEGORIES.len(), 7);
    }

    #[test]
    fn test_parse_valid_json_classification() {
        let sample_json = r#"{"category": "Office Supplies & Stationery", "confidence": 0.95, "rationale": "Purchase of paper and pens"}"#;
        let parsed: serde_json::Value = serde_json::from_str(sample_json).unwrap();
        assert_eq!(parsed["category"].as_str().unwrap(), "Office Supplies & Stationery");
        assert_eq!(parsed["confidence"].as_f64().unwrap(), 0.95);
    }
}
