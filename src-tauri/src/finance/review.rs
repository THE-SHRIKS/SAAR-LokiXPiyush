use crate::db::Database;
use rusqlite::{params, Result};
use serde_json::json;

pub struct ReviewService;

impl ReviewService {
    /// Approves a proposed reconciliation link between an invoice and a bank transaction.
    /// Sets both entities to 'APPROVED' and records an immutable audit log entry.
    pub fn approve_link(db: &mut Database, workspace_id: &str, link_id: &str) -> Result<()> {
        let (inv_id, tx_id, amount): (String, String, i64) = db
            .conn_ref()
            .query_row(
                "SELECT invoice_id, transaction_id, matched_amount_minor FROM invoice_payment_links WHERE id = ?1 AND workspace_id = ?2;",
                params![link_id, workspace_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;

        let conn = db.connection_mut();
        let tx = conn.transaction()?;

        tx.execute(
            "UPDATE invoice_payment_links SET status = 'APPROVED' WHERE id = ?1;",
            params![link_id],
        )?;

        tx.execute(
            "UPDATE transactions SET matched_invoice_id = ?1, review_state = 'APPROVED' WHERE id = ?2;",
            params![inv_id, tx_id],
        )?;

        tx.execute(
            "UPDATE invoices SET review_state = 'APPROVED' WHERE id = ?1;",
            params![inv_id],
        )?;

        tx.commit()?;

        let audit_details = json!({
            "link_id": link_id,
            "invoice_id": inv_id,
            "transaction_id": tx_id,
            "matched_amount_minor": amount,
            "decision": "APPROVED"
        })
        .to_string();

        db.record_audit_event(
            workspace_id,
            "invoice_payment_link",
            link_id,
            "APPROVE_LINK",
            &audit_details,
        )?;

        Ok(())
    }

    /// Rejects a proposed reconciliation link.
    pub fn reject_link(db: &Database, workspace_id: &str, link_id: &str) -> Result<()> {
        db.conn_ref().execute(
            "UPDATE invoice_payment_links SET status = 'REJECTED' WHERE id = ?1 AND workspace_id = ?2;",
            params![link_id, workspace_id],
        )?;

        let audit_details = json!({
            "link_id": link_id,
            "decision": "REJECTED"
        })
        .to_string();

        db.record_audit_event(
            workspace_id,
            "invoice_payment_link",
            link_id,
            "REJECT_LINK",
            &audit_details,
        )?;

        Ok(())
    }

    /// User correction of a transaction description or reference.
    /// Captures prior values and persists full audit trail.
    pub fn correct_transaction(
        db: &Database,
        workspace_id: &str,
        tx_id: &str,
        new_description: &str,
    ) -> Result<()> {
        let prior_desc: String = db.conn_ref().query_row(
            "SELECT description FROM transactions WHERE id = ?1 AND workspace_id = ?2;",
            params![tx_id, workspace_id],
            |r| r.get(0),
        )?;

        db.conn_ref().execute(
            "UPDATE transactions SET description = ?1, review_state = 'CORRECTED' WHERE id = ?2 AND workspace_id = ?3;",
            params![new_description, tx_id, workspace_id],
        )?;

        let audit_details = json!({
            "field": "description",
            "prior_value": prior_desc,
            "new_value": new_description,
        })
        .to_string();

        db.record_audit_event(
            workspace_id,
            "transaction",
            tx_id,
            "CORRECT_TRANSACTION",
            &audit_details,
        )?;

        Ok(())
    }

    /// User correction of an invoice vendor or invoice number.
    pub fn correct_invoice(
        db: &Database,
        workspace_id: &str,
        invoice_id: &str,
        new_vendor: &str,
        new_number: Option<&str>,
    ) -> Result<()> {
        let (prior_vendor, prior_number): (String, Option<String>) = db.conn_ref().query_row(
            "SELECT vendor_name, invoice_number FROM invoices WHERE id = ?1;",
            params![invoice_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        db.conn_ref().execute(
            "UPDATE invoices SET vendor_name = ?1, invoice_number = ?2, review_state = 'CORRECTED' WHERE id = ?3;",
            params![new_vendor, new_number, invoice_id],
        )?;

        let audit_details = json!({
            "prior_vendor": prior_vendor,
            "new_vendor": new_vendor,
            "prior_number": prior_number,
            "new_number": new_number,
        })
        .to_string();

        db.record_audit_event(
            workspace_id,
            "invoice",
            invoice_id,
            "CORRECT_INVOICE",
            &audit_details,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approve_link_updates_state_and_audit() {
        let mut db = Database::open_in_memory().unwrap();
        let ws_id = "ws-review-test";
        db.create_workspace(ws_id, "Review Test", "USD", &[0u8; 16], &[1u8; 32])
            .unwrap();

        // Insert document, invoice, and transaction
        db.conn_ref().execute(
            "INSERT INTO documents (id, workspace_id, file_name, file_size_bytes, sha256, imported_at)
             VALUES ('doc-rev', 'ws-review-test', 'inv.pdf', 100, 'hash', datetime('now'));",
            [],
        ).unwrap();

        db.conn_ref().execute(
            "INSERT INTO invoices (id, document_id, vendor_name, invoice_number, total_minor, review_state)
             VALUES ('inv-rev', 'doc-rev', 'CloudHosting', 'INV-1', 12000, 'UNREVIEWED');",
            [],
        ).unwrap();

        db.conn_ref().execute(
            "INSERT INTO transactions (id, workspace_id, posted_date, description, amount_minor, review_state)
             VALUES ('tx-rev', 'ws-review-test', '2026-09-02', 'CloudHosting', -12000, 'UNREVIEWED');",
            [],
        ).unwrap();

        db.conn_ref().execute(
            "INSERT INTO invoice_payment_links (id, workspace_id, invoice_id, transaction_id, matched_amount_minor, confidence_score, match_type, explanation, status, created_at)
             VALUES ('link-1', 'ws-review-test', 'inv-rev', 'tx-rev', 12000, 1.0, 'EXACT', 'Exact match', 'PROPOSED', datetime('now'));",
            [],
        ).unwrap();

        // Execute approve
        ReviewService::approve_link(&mut db, ws_id, "link-1").unwrap();

        // Verify status is approved on all entities
        let link_status: String = db
            .conn_ref()
            .query_row(
                "SELECT status FROM invoice_payment_links WHERE id = 'link-1';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(link_status, "APPROVED");

        let tx_state: String = db
            .conn_ref()
            .query_row(
                "SELECT review_state FROM transactions WHERE id = 'tx-rev';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tx_state, "APPROVED");

        let inv_state: String = db
            .conn_ref()
            .query_row(
                "SELECT review_state FROM invoices WHERE id = 'inv-rev';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(inv_state, "APPROVED");

        // Verify audit trail
        let audit_count: i64 = db
            .conn_ref()
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'APPROVE_LINK';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 1);
    }
}
