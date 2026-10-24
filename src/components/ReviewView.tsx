import React, { useState } from "react";
import {
  RefreshCw,
  AlertTriangle,
  GitCompare,
  Check,
  X,
  Tag,
  Sparkles,
  ShieldCheck,
  Edit2,
  Database,
} from "lucide-react";
import {
  CurrencyCode,
  ProposedLink,
  FinancialAnomaly,
  TransactionRecord,
  AuditEventRecord,
  CategoryProposal,
  formatMinorCurrency,
} from "../lib/types";

interface ReviewViewProps {
  reconciliationLinks: ProposedLink[];
  anomalies: FinancialAnomaly[];
  transactions: TransactionRecord[];
  auditLogs: AuditEventRecord[];
  proposals: Record<string, CategoryProposal>;
  isReconciling: boolean;
  reviewMessage: string | null;
  onRunReconciliation: () => Promise<void>;
  onApproveLink: (linkId: string) => Promise<void>;
  onRejectLink: (linkId: string) => Promise<void>;
  onClassifyTransaction: (txId: string) => Promise<void>;
  onApproveCategory: (txId: string) => Promise<void>;
  onSaveCorrection: (txId: string, newDesc: string) => Promise<void>;
  onSelectTransaction: (tx: TransactionRecord) => void;
  onSelectLink: (link: ProposedLink) => void;
}

const ANOMALY_LABELS: Record<string, string> = {
  DUPLICATE_INVOICE: "Duplicate invoice",
  POTENTIAL_DUPLICATE_PAYMENT: "Possible duplicate payment",
  LARGE_UNMATCHED: "Large payment without a document",
};

/** Readable name for an anomaly code; unknown codes are shown as sentence-case words. */
const anomalyLabel = (code: string): string =>
  ANOMALY_LABELS[code] ?? code.charAt(0) + code.slice(1).toLowerCase().replace(/_/g, " ");

export const ReviewView: React.FC<ReviewViewProps> = ({
  reconciliationLinks,
  anomalies,
  transactions,
  auditLogs,
  proposals,
  isReconciling,
  reviewMessage,
  onRunReconciliation,
  onApproveLink,
  onRejectLink,
  onClassifyTransaction,
  onApproveCategory,
  onSaveCorrection,
  onSelectTransaction,
  onSelectLink,
}) => {
  const [subTab, setSubTab] = useState<"reconciliation" | "ledger" | "audit">("reconciliation");
  const [editingTx, setEditingTx] = useState<TransactionRecord | null>(null);
  const [editDescription, setEditDescription] = useState("");
  const [classifyingTxId, setClassifyingTxId] = useState<string | null>(null);

  const handleStartEdit = (tx: TransactionRecord) => {
    setEditingTx(tx);
    setEditDescription(tx.description);
  };

  const handleCommitEdit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingTx || !editDescription.trim()) return;
    await onSaveCorrection(editingTx.id, editDescription.trim());
    setEditingTx(null);
    setEditDescription("");
  };

  const handleTriggerClassify = async (txId: string) => {
    setClassifyingTxId(txId);
    try {
      await onClassifyTransaction(txId);
    } finally {
      setClassifyingTxId(null);
    }
  };

  return (
    <div className="review-workspace">
      {/* Sub-Navigation Header */}
      <div className="review-subnav-bar">
        <div className="subnav-tabs">
          <button
            className={`subnav-tab ${subTab === "reconciliation" ? "active" : ""}`}
            onClick={() => setSubTab("reconciliation")}
          >
            <GitCompare size={14} />
            <span>Matches</span>
            {reconciliationLinks.length > 0 && (
              <span className="tab-pill-count">{reconciliationLinks.length}</span>
            )}
          </button>

          <button
            className={`subnav-tab ${subTab === "ledger" ? "active" : ""}`}
            onClick={() => setSubTab("ledger")}
          >
            <Database size={14} />
            <span>Transactions</span>
            <span className="tab-pill-count">{transactions.length}</span>
          </button>

          <button
            className={`subnav-tab ${subTab === "audit" ? "active" : ""}`}
            onClick={() => setSubTab("audit")}
          >
            <ShieldCheck size={14} />
            <span>Activity log</span>
            <span className="tab-pill-count">{auditLogs.length}</span>
          </button>
        </div>

        {subTab === "reconciliation" && (
          <button
            className="btn-primary-medium"
            onClick={onRunReconciliation}
            disabled={isReconciling}
          >
            <RefreshCw size={13} className={isReconciling ? "spin" : ""} />
            <span>{isReconciling ? "Matching..." : "Run Reconciliation"}</span>
          </button>
        )}
      </div>

      {reviewMessage && (
        <div className="review-notification-banner">
          <span>{reviewMessage}</span>
        </div>
      )}

      {/* Mode 1: Reconciliation Inbox */}
      {subTab === "reconciliation" && (
        <div className="reconciliation-content">
          {/* Anomaly Alerts */}
          {anomalies.length > 0 && (
            <div className="anomalies-container">
              <div className="section-head-simple">
                <AlertTriangle size={14} className="icon-amber" />
                <h5>Detected Financial Anomalies ({anomalies.length})</h5>
              </div>

              <div className="anomalies-list">
                {anomalies.map((anom) => (
                  <div
                    key={anom.id}
                    className={`anomaly-card severity-${anom.severity.toLowerCase()}`}
                  >
                    <div className="anomaly-header">
                      <span className={`badge-severity ${anom.severity.toLowerCase()}`}>
                        {anom.severity}
                      </span>
                      <span className="anomaly-type-code">{anomalyLabel(anom.anomalyType)}</span>
                    </div>
                    <p className="anomaly-description">{anom.description}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Proposed Match Candidates */}
          <div className="matches-container">
            <div className="section-head-simple">
              <GitCompare size={14} className="icon-cream" />
              <h5>Candidate Link Proposals ({reconciliationLinks.length})</h5>
            </div>

            {reconciliationLinks.length === 0 ? (
              <div className="empty-panel-state">
                <p>No matches to review</p>
                <span>
                  Run reconciliation to match invoices with bank rows.
                </span>
              </div>
            ) : (
              <div className="matches-grid">
                {reconciliationLinks.map((link) => (
                  <div
                    key={link.id}
                    className="match-proposal-card"
                    onClick={() => onSelectLink(link)}
                  >
                    <div className="match-card-top">
                      <div className="match-badge-group">
                        <span className={`match-type-pill ${link.matchType.toLowerCase()}`}>
                          {link.matchType}
                        </span>
                        <span className="confidence-pill">
                          {(link.confidenceScore * 100).toFixed(0)}% Confidence
                        </span>
                      </div>
                      <div className="match-amount font-mono">
                        {formatMinorCurrency(
                          link.matchedAmountMinor,
                          (transactions.find((t) => t.id === link.transactionId)?.currency ?? "USD") as CurrencyCode,
                        )}
                      </div>
                    </div>

                    <p className="match-explanation">{link.explanation}</p>

                    <div className="match-card-actions">
                      {link.status === "PROPOSED" ? (
                        <>
                          <button
                            className="btn-approve"
                            onClick={(e) => {
                              e.stopPropagation();
                              onApproveLink(link.id);
                            }}
                          >
                            <Check size={12} />
                            <span>Approve Match</span>
                          </button>
                          <button
                            className="btn-reject"
                            onClick={(e) => {
                              e.stopPropagation();
                              onRejectLink(link.id);
                            }}
                          >
                            <X size={12} />
                            <span>Reject</span>
                          </button>
                        </>
                      ) : (
                        <span className="match-recorded-state">Audit Logged: {link.status}</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Mode 2: Ledger Postings */}
      {subTab === "ledger" && (
        <div className="ledger-content">
          {editingTx && (
            <div className="inline-edit-modal-bar">
              <span className="edit-modal-title">
                Manual Audit Correction: <code>{editingTx.id.substring(0, 10)}...</code>
              </span>
              <form onSubmit={handleCommitEdit} className="edit-form-row">
                <input
                  type="text"
                  className="input-text edit-input"
                  value={editDescription}
                  onChange={(e) => setEditDescription(e.target.value)}
                  placeholder="Enter corrected description..."
                  autoFocus
                />
                <button type="submit" className="btn-primary-small">
                  <Check size={12} />
                  <span>Save Correction</span>
                </button>
                <button
                  type="button"
                  className="btn-secondary-small"
                  onClick={() => setEditingTx(null)}
                >
                  <X size={12} />
                  <span>Cancel</span>
                </button>
              </form>
            </div>
          )}

          <div className="ledger-table-card">
            <table className="app-data-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Description</th>
                  <th>Reference</th>
                  <th className="num-col">Amount</th>
                  <th>Category</th>
                  <th>Review State</th>
                  <th className="action-col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {transactions.map((tx) => {
                  const proposal = proposals[tx.id];
                  return (
                    <tr key={tx.id} className="table-row" onClick={() => onSelectTransaction(tx)}>
                      <td className="font-mono text-xs">{tx.postedDate}</td>
                      <td className="tx-desc-cell">{tx.description}</td>
                      <td className="font-mono text-muted text-xs">{tx.externalRef || "—"}</td>
                      <td
                        className={`num-col font-mono ${
                          tx.amountMinor < 0 ? "amount-debit" : "amount-credit"
                        }`}
                      >
                        {formatMinorCurrency(tx.amountMinor, tx.currency)}
                      </td>
                      <td>
                        {proposal ? (
                          <div className="category-tag-wrap">
                            <span className="category-name">
                              <Tag size={10} />
                              {proposal.suggestedCategory}
                            </span>
                            {proposal.status === "PROPOSED" ? (
                              <button
                                className="btn-approve-cat"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  onApproveCategory(tx.id);
                                }}
                                title="Approve category"
                              >
                                Approve ({(proposal.confidenceScore * 100).toFixed(0)}%)
                              </button>
                            ) : (
                              <span className="cat-approved-check">Approved</span>
                            )}
                          </div>
                        ) : (
                          <button
                            className="btn-auto-classify"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleTriggerClassify(tx.id);
                            }}
                            disabled={classifyingTxId === tx.id}
                          >
                            <Sparkles
                              size={11}
                              className={classifyingTxId === tx.id ? "spin" : ""}
                            />
                            <span>
                              {classifyingTxId === tx.id ? "Classifying..." : "Auto-Classify"}
                            </span>
                          </button>
                        )}
                      </td>
                      <td>
                        <span className={`status-pill ${tx.reviewState.toLowerCase()}`}>
                          {tx.reviewState}
                        </span>
                      </td>
                      <td className="action-col">
                        <button
                          className="btn-edit-action"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleStartEdit(tx);
                          }}
                          title="Correct description in immutable audit log"
                        >
                          <Edit2 size={12} />
                          <span>Edit</span>
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Mode 3: Immutable Audit Trail */}
      {subTab === "audit" && (
        <div className="audit-content">
          <div className="audit-table-card">
            <div className="table-card-head">
              <span className="table-head-title">
                Immutable SQLite Audit Trail ({auditLogs.length} events)
              </span>
              <span className="table-head-subtitle">Record of approvals, rejections and other changes</span>
            </div>

            <table className="app-data-table">
              <thead>
                <tr>
                  <th>Timestamp</th>
                  <th>Action</th>
                  <th>Target Entity</th>
                  <th>Details</th>
                </tr>
              </thead>
              <tbody>
                {auditLogs.map((log) => (
                  <tr key={log.id} className="table-row">
                    <td className="font-mono text-muted text-xs">{log.createdAt}</td>
                    <td>
                      <span className="audit-action-badge">{log.action}</span>
                    </td>
                    <td className="font-mono text-xs">
                      {log.entityType} ({log.entityId.substring(0, 10)}...)
                    </td>
                    <td className="font-mono text-muted text-xs payload-cell">{log.payloadJson}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
};
