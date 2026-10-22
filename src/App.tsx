import React, { useEffect, useState } from "react";
import {
  SystemStatus,
  WorkspaceInfo,
  DocumentRecord,
  TransactionRecord,
  DocumentDetails,
  ProposedLink,
  FinancialAnomaly,
  FinancialSummary,
  formatMinorCurrency,
} from "./lib/types";
import { OpenDeckApi } from "./lib/api";
import {
  ShieldCheck,
  FileText,
  Database,
  Lock,
  Unlock,
  HardDrive,
  Cpu,
  UploadCloud,
  CheckCircle2,
  AlertCircle,
  Clock,
  ChevronRight,
  GitCompare,
  BarChart3,
  Check,
  X,
  AlertTriangle,
  RefreshCw,
} from "lucide-react";

export const App: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [activeTab, setActiveTab] = useState<
    "documents" | "transactions" | "reconciliation" | "reports" | "settings"
  >("documents");
  const [documents, setDocuments] = useState<DocumentRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [selectedDocDetails, setSelectedDocDetails] = useState<DocumentDetails | null>(null);

  // Reconciliation & Review state
  const [reconciliationLinks, setReconciliationLinks] = useState<ProposedLink[]>([]);
  const [anomalies, setAnomalies] = useState<FinancialAnomaly[]>([]);
  const [financialSummary, setFinancialSummary] = useState<FinancialSummary | null>(null);
  const [isReconciling, setIsReconciling] = useState<boolean>(false);
  const [reviewMessage, setReviewMessage] = useState<string | null>(null);

  // Authentication & Workspace creation state
  const [passphrase, setPassphrase] = useState<string>("");
  const [workspaceName, setWorkspaceName] = useState<string>("THE SHRIKS Primary");
  const [authError, setAuthError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Ingestion state
  const [importPath, setImportPath] = useState<string>("");
  const [importMessage, setImportMessage] = useState<{
    text: string;
    type: "success" | "error" | "info";
  } | null>(null);
  const [isImporting, setIsImporting] = useState<boolean>(false);

  useEffect(() => {
    OpenDeckApi.getSystemStatus().then(setStatus).catch(console.error);
  }, []);

  const refreshData = async () => {
    try {
      const docs = await OpenDeckApi.listDocuments();
      setDocuments(docs);
      const txs = await OpenDeckApi.listTransactions();
      setTransactions(txs);
      const links = await OpenDeckApi.listReconciliationLinks();
      setReconciliationLinks(links);
      const anoms = await OpenDeckApi.listAnomalies();
      setAnomalies(anoms);
      const summary = await OpenDeckApi.getFinancialSummary("USD");
      setFinancialSummary(summary);
    } catch (e) {
      console.error("Refresh failed:", e);
    }
  };

  const handleUnlockOrCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!passphrase) {
      setAuthError("Passphrase cannot be empty.");
      return;
    }
    setAuthError(null);
    setIsLoading(true);

    try {
      const ws = await OpenDeckApi.createWorkspace(workspaceName, passphrase, "USD");
      setWorkspace(ws);
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setAuthError(`Authentication error: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLock = async () => {
    await OpenDeckApi.lockWorkspace();
    setWorkspace(null);
    setDocuments([]);
    setTransactions([]);
    setReconciliationLinks([]);
    setAnomalies([]);
    setFinancialSummary(null);
    setSelectedDocDetails(null);
    setPassphrase("");
  };

  const handleIngest = async (targetPath: string) => {
    if (!targetPath.trim()) return;
    setIsImporting(true);
    setImportMessage({ text: `Ingesting document from ${targetPath}...`, type: "info" });

    try {
      const res = await OpenDeckApi.ingestFile(targetPath.trim());
      if (res.type === "success") {
        setImportMessage({
          text: `Successfully ingested "${res.fileName}" (${res.docType}) — ${res.recordsExtracted} items extracted.`,
          type: "success",
        });
        await refreshData();
        const details = await OpenDeckApi.getDocumentDetails(res.documentId);
        setSelectedDocDetails(details);
      } else {
        setImportMessage({
          text: `Deduplication: "${res.fileName}" already exists in workspace (SHA-256: ${res.sha256.substring(0, 12)}...).`,
          type: "info",
        });
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setImportMessage({ text: `Ingestion failed: ${msg}`, type: "error" });
    } finally {
      setIsImporting(false);
      setImportPath("");
    }
  };

  const handleRunReconciliation = async () => {
    setIsReconciling(true);
    setReviewMessage("Running deterministic matching rules & anomaly detection...");
    try {
      const res = await OpenDeckApi.runReconciliation();
      setReconciliationLinks(res.links);
      setAnomalies(res.anomalies);
      setReviewMessage(
        `Reconciliation complete: ${res.links.length} proposed links found, ${res.anomalies.length} anomalies flagged.`
      );
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Reconciliation failed: ${msg}`);
    } finally {
      setIsReconciling(false);
    }
  };

  const handleApproveLink = async (linkId: string) => {
    try {
      await OpenDeckApi.approveReconciliationLink(linkId);
      setReviewMessage("Match link approved and audit trail recorded.");
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Approval failed: ${msg}`);
    }
  };

  const handleRejectLink = async (linkId: string) => {
    try {
      await OpenDeckApi.rejectReconciliationLink(linkId, "User rejected match suggestion");
      setReviewMessage("Match link rejected and logged to audit trail.");
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Rejection failed: ${msg}`);
    }
  };

  const selectDocument = async (docId: string) => {
    try {
      const details = await OpenDeckApi.getDocumentDetails(docId);
      setSelectedDocDetails(details);
    } catch (err) {
      console.error("Failed to load details:", err);
    }
  };

  return (
    <div className="app-container">
      {/* Editorial Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="brand-title">OpenDeck</div>
          <div className="brand-badge">Air-Gapped Desktop</div>
        </div>

        <nav className="sidebar-nav">
          <div
            className={`nav-item ${activeTab === "documents" ? "active" : ""}`}
            onClick={() => setActiveTab("documents")}
          >
            <FileText size={16} />
            <span>Documents & Ingestion</span>
          </div>

          <div
            className={`nav-item ${activeTab === "transactions" ? "active" : ""}`}
            onClick={() => setActiveTab("transactions")}
          >
            <Database size={16} />
            <span>Ledger Postings</span>
          </div>

          <div
            className={`nav-item ${activeTab === "reconciliation" ? "active" : ""}`}
            onClick={() => setActiveTab("reconciliation")}
          >
            <GitCompare size={16} />
            <span>Reconciliation & Inbox</span>
            {anomalies.length > 0 && <span className="nav-badge">{anomalies.length}</span>}
          </div>

          <div
            className={`nav-item ${activeTab === "reports" ? "active" : ""}`}
            onClick={() => setActiveTab("reports")}
          >
            <BarChart3 size={16} />
            <span>Financial Reports</span>
          </div>

          <div
            className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            <ShieldCheck size={16} />
            <span>Security & Runtime</span>
          </div>
        </nav>

        <div className="sidebar-footer">
          <div className="status-indicator">
            <span className="dot-green"></span>
            <span>Zero Network • Strict Offline</span>
          </div>
          <div className="hardware-info">Intel Core i3 • AVX2 CPU</div>
        </div>
      </aside>

      {/* Main Content Pane */}
      <main className="main-content">
        <header className="topbar">
          <div className="workspace-status">
            {workspace ? (
              <div className="unlocked-badge">
                <Unlock size={14} className="icon-green" />
                <span>{workspace.displayName}</span>
                <span className="pill">AES-256-GCM Encrypted</span>
              </div>
            ) : (
              <div className="locked-badge">
                <Lock size={14} className="icon-amber" />
                <span>Workspace Locked</span>
              </div>
            )}
          </div>

          <div className="topbar-actions">
            {workspace && (
              <button className="btn-secondary" onClick={handleLock}>
                <Lock size={14} />
                <span>Lock Workspace</span>
              </button>
            )}
          </div>
        </header>

        <div className="content-body">
          {!workspace ? (
            /* Master Passphrase Unlock Surface */
            <div className="auth-card">
              <div className="auth-header">
                <h2>Unlock Workspace</h2>
                <p>
                  Keys are derived on device via Argon2id (64 MiB, 3 iterations). Zero cloud
                  telemetry.
                </p>
              </div>

              <form onSubmit={handleUnlockOrCreate} className="auth-form">
                <div className="form-group">
                  <label>Workspace Title</label>
                  <input
                    type="text"
                    value={workspaceName}
                    onChange={(e) => setWorkspaceName(e.target.value)}
                    placeholder="Enter workspace name"
                    className="input-field"
                  />
                </div>

                <div className="form-group">
                  <label>Master Passphrase</label>
                  <input
                    type="password"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="Enter master passphrase"
                    className="input-field"
                    autoFocus
                  />
                </div>

                {authError && <div className="error-banner">{authError}</div>}

                <button type="submit" className="btn-primary" disabled={isLoading}>
                  {isLoading ? "Deriving Argon2id Keys..." : "Initialize / Unlock"}
                </button>
              </form>
            </div>
          ) : (
            /* Authenticated Workspace Content */
            <div>
              {activeTab === "documents" && (
                <div className="documents-view">
                  {/* File Ingestion Bar */}
                  <div className="import-card">
                    <div className="import-header">
                      <div className="import-title">
                        <UploadCloud size={18} />
                        <h3>Secure Document Ingestion</h3>
                      </div>
                      <span className="import-desc">
                        PDF, PNG, JPG, or Bank CSV (max 50 MB, 50 pages)
                      </span>
                    </div>

                    <div className="import-input-row">
                      <input
                        type="text"
                        value={importPath}
                        onChange={(e) => setImportPath(e.target.value)}
                        placeholder="Enter absolute file path to ingest..."
                        className="input-field flex-grow"
                        disabled={isImporting}
                      />
                      <button
                        className="btn-primary"
                        onClick={() => handleIngest(importPath)}
                        disabled={isImporting || !importPath.trim()}
                      >
                        {isImporting ? "Processing..." : "Ingest File"}
                      </button>
                    </div>

                    {/* Quick Fixture Buttons for instant manual testing */}
                    <div className="quick-fixtures">
                      <span className="quick-fixtures-label">Sample Fixtures:</span>
                      <button
                        className="btn-tag"
                        onClick={() =>
                          handleIngest(
                            "/home/theshriksloki/OpenDeck/tests/fixtures/invoices/gst_b2b_valid.pdf"
                          )
                        }
                        disabled={isImporting}
                      >
                        GST Invoice (PDF)
                      </button>
                      <button
                        className="btn-tag"
                        onClick={() =>
                          handleIngest(
                            "/home/theshriksloki/OpenDeck/tests/fixtures/receipts/thermal_receipt_scanned.png"
                          )
                        }
                        disabled={isImporting}
                      >
                        Thermal Receipt (OCR)
                      </button>
                      <button
                        className="btn-tag"
                        onClick={() =>
                          handleIngest(
                            "/home/theshriksloki/OpenDeck/tests/fixtures/statements/bank_us_sample.csv"
                          )
                        }
                        disabled={isImporting}
                      >
                        US Bank Statement (CSV)
                      </button>
                      <button
                        className="btn-tag"
                        onClick={() =>
                          handleIngest(
                            "/home/theshriksloki/OpenDeck/tests/fixtures/statements/bank_hdfc_sample.csv"
                          )
                        }
                        disabled={isImporting}
                      >
                        HDFC Statement (CSV)
                      </button>
                    </div>

                    {importMessage && (
                      <div className={`message-banner message-${importMessage.type}`}>
                        {importMessage.type === "success" && <CheckCircle2 size={16} />}
                        {importMessage.type === "error" && <AlertCircle size={16} />}
                        {importMessage.type === "info" && <Clock size={16} />}
                        <span>{importMessage.text}</span>
                      </div>
                    )}
                  </div>

                  {/* Documents Layout: Master Table + Detail Inspector */}
                  <div className="workspace-split">
                    <div className="table-panel">
                      <div className="panel-header">
                        <h4>Ingested Documents ({documents.length})</h4>
                      </div>

                      {documents.length === 0 ? (
                        <div className="empty-state">
                          <FileText size={36} className="text-muted" />
                          <p>No documents imported yet.</p>
                          <span className="text-small">
                            Ingest a PDF invoice, scanned receipt, or bank CSV above.
                          </span>
                        </div>
                      ) : (
                        <table className="data-table">
                          <thead>
                            <tr>
                              <th>File Name</th>
                              <th>Type</th>
                              <th>Size</th>
                              <th>Pages</th>
                              <th>Status</th>
                              <th>Action</th>
                            </tr>
                          </thead>
                          <tbody>
                            {documents.map((doc) => (
                              <tr
                                key={doc.id}
                                className={
                                  selectedDocDetails?.document.id === doc.id ? "selected-row" : ""
                                }
                                onClick={() => selectDocument(doc.id)}
                              >
                                <td className="font-medium">{doc.fileName}</td>
                                <td>
                                  <span className={`badge-type badge-${doc.docType.toLowerCase()}`}>
                                    {doc.docType}
                                  </span>
                                </td>
                                <td>{(doc.fileSizeBytes / 1024).toFixed(1)} KB</td>
                                <td>{doc.pageCount}</td>
                                <td>
                                  <span className="status-pill status-ready">{doc.status}</span>
                                </td>
                                <td>
                                  <ChevronRight size={16} className="text-muted" />
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      )}
                    </div>

                    {/* Evidence Inspector Side Panel */}
                    {selectedDocDetails && (
                      <div className="detail-panel">
                        <div className="detail-header">
                          <h4>Evidence Inspector</h4>
                          <span className="text-small text-muted">
                            {selectedDocDetails.document.fileName}
                          </span>
                        </div>

                        {selectedDocDetails.invoiceTotalMinor !== undefined && (
                          <div className="detail-summary-card">
                            <div className="summary-label">Parsed Total</div>
                            <div className="summary-amount">
                              {formatMinorCurrency(selectedDocDetails.invoiceTotalMinor, "USD")}
                            </div>
                          </div>
                        )}

                        <div className="detail-section">
                          <h5>Extracted Normalized Fields</h5>
                          {selectedDocDetails.fields.length === 0 ? (
                            <p className="text-small text-muted">
                              No key-value fields (standard transaction statement).
                            </p>
                          ) : (
                            <div className="field-list">
                              {selectedDocDetails.fields.map((f, idx) => (
                                <div key={idx} className="field-item">
                                  <div className="field-meta">
                                    <span className="field-key">{f.fieldKey}</span>
                                    <span className="field-confidence">
                                      {(f.confidence * 100).toFixed(0)}% conf • p.{f.pageNum}
                                    </span>
                                  </div>
                                  <div className="field-val">{f.normalizedValue}</div>
                                  <div className="field-box text-small text-muted">
                                    Box: {f.boundingBoxJson}
                                  </div>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>

                        {selectedDocDetails.lineItems.length > 0 && (
                          <div className="detail-section">
                            <h5>Line Items ({selectedDocDetails.lineItems.length})</h5>
                            <ul className="line-item-list">
                              {selectedDocDetails.lineItems.map((item, idx) => (
                                <li key={idx} className="line-item">
                                  <span>{item.description}</span>
                                  <span className="font-mono">
                                    {formatMinorCurrency(item.amountMinor, "USD")}
                                  </span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {activeTab === "transactions" && (
                <div className="transactions-view">
                  <div className="panel-header">
                    <h4>Normalized Ledger Postings ({transactions.length})</h4>
                    <span className="text-small text-muted">
                      Deterministic integer minor-unit sums
                    </span>
                  </div>

                  {transactions.length === 0 ? (
                    <div className="empty-state">
                      <Database size={36} className="text-muted" />
                      <p>No transactions imported yet.</p>
                      <span className="text-small">
                        Import a bank statement CSV to populate normalized transactions.
                      </span>
                    </div>
                  ) : (
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>Description</th>
                          <th>Reference</th>
                          <th>Amount</th>
                          <th>Currency</th>
                          <th>Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {transactions.map((tx) => (
                          <tr key={tx.id}>
                            <td className="font-mono">{tx.postedDate}</td>
                            <td>{tx.description}</td>
                            <td className="font-mono text-muted">{tx.externalRef || "—"}</td>
                            <td
                              className={`font-mono font-medium ${
                                tx.amountMinor < 0 ? "amount-debit" : "amount-credit"
                              }`}
                            >
                              {formatMinorCurrency(tx.amountMinor, tx.currency)}
                            </td>
                            <td>
                              <span className="pill-currency">{tx.currency}</span>
                            </td>
                            <td>
                              <span className={`status-pill status-${tx.reviewState.toLowerCase()}`}>
                                {tx.reviewState}
                              </span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}

              {activeTab === "reconciliation" && (
                <div className="reconciliation-view">
                  <div className="reconcile-toolbar">
                    <div className="toolbar-info">
                      <h4>Deterministic Reconciliation Inbox</h4>
                      <span className="text-small text-muted">
                        Automated two-tier matching engine with explainability and anomaly guards
                      </span>
                    </div>
                    <button
                      className="btn-primary"
                      onClick={handleRunReconciliation}
                      disabled={isReconciling}
                    >
                      <RefreshCw size={14} className={isReconciling ? "spin" : ""} />
                      <span>{isReconciling ? "Matching..." : "Run Reconciliation"}</span>
                    </button>
                  </div>

                  {reviewMessage && (
                    <div className="message-banner message-info">
                      <Clock size={16} />
                      <span>{reviewMessage}</span>
                    </div>
                  )}

                  {/* Anomaly Alerts Section */}
                  {anomalies.length > 0 && (
                    <div className="anomaly-section">
                      <div className="section-title">
                        <AlertTriangle size={16} className="icon-amber" />
                        <h5>Detected Financial Anomalies ({anomalies.length})</h5>
                      </div>
                      <div className="anomaly-cards">
                        {anomalies.map((anom) => (
                          <div key={anom.id} className={`anomaly-card severity-${anom.severity.toLowerCase()}`}>
                            <div className="anomaly-meta">
                              <span className="badge-severity">{anom.severity}</span>
                              <span className="font-mono text-small">{anom.anomalyType}</span>
                            </div>
                            <div className="anomaly-desc">{anom.description}</div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Proposed Matches Inbox */}
                  <div className="links-section">
                    <h5>Proposed Matches ({reconciliationLinks.length})</h5>
                    {reconciliationLinks.length === 0 ? (
                      <div className="empty-state">
                        <GitCompare size={36} className="text-muted" />
                        <p>No match proposals currently in inbox.</p>
                        <span className="text-small">
                          Click "Run Reconciliation" to match invoices against ledger transactions.
                        </span>
                      </div>
                    ) : (
                      <div className="link-cards">
                        {reconciliationLinks.map((link) => (
                          <div key={link.id} className="link-card">
                            <div className="link-card-header">
                              <div className="match-badges">
                                <span className={`badge-match badge-${link.matchType.toLowerCase()}`}>
                                  {link.matchType}
                                </span>
                                <span className="badge-score">
                                  {(link.confidenceScore * 100).toFixed(0)}% Confidence
                                </span>
                                <span className={`status-pill status-${link.status.toLowerCase()}`}>
                                  {link.status}
                                </span>
                              </div>
                              <div className="match-amount font-mono">
                                {formatMinorCurrency(link.matchedAmountMinor, "USD")}
                              </div>
                            </div>

                            <p className="link-explanation">{link.explanation}</p>

                            <div className="link-actions">
                              {link.status === "PROPOSED" ? (
                                <>
                                  <button
                                    className="btn-action btn-approve"
                                    onClick={() => handleApproveLink(link.id)}
                                  >
                                    <Check size={14} />
                                    <span>Approve Match</span>
                                  </button>
                                  <button
                                    className="btn-action btn-reject"
                                    onClick={() => handleRejectLink(link.id)}
                                  >
                                    <X size={14} />
                                    <span>Reject</span>
                                  </button>
                                </>
                              ) : (
                                <span className="reviewed-label">
                                  Decided: {link.status} (Audit Recorded)
                                </span>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {activeTab === "reports" && (
                <div className="reports-view">
                  <div className="panel-header">
                    <h4>Deterministic Financial Reports</h4>
                    <span className="text-small text-muted">
                      Exact minor-unit ledger aggregates (Zero floating-point rounding)
                    </span>
                  </div>

                  {financialSummary && (
                    <div className="summary-cards">
                      <div className="report-card">
                        <div className="report-card-label">Total Inflow / Income</div>
                        <div className="report-card-val amount-credit">
                          {formatMinorCurrency(financialSummary.totalIncomeMinor, "USD")}
                        </div>
                      </div>

                      <div className="report-card">
                        <div className="report-card-label">Total Outflow / Expenses</div>
                        <div className="report-card-val amount-debit">
                          {formatMinorCurrency(financialSummary.totalExpenseMinor, "USD")}
                        </div>
                      </div>

                      <div className="report-card">
                        <div className="report-card-label">Net Savings / Cash Flow</div>
                        <div
                          className={`report-card-val ${
                            financialSummary.netSavingsMinor >= 0 ? "amount-credit" : "amount-debit"
                          }`}
                        >
                          {formatMinorCurrency(financialSummary.netSavingsMinor, "USD")}
                        </div>
                      </div>

                      {financialSummary.momExpenseChangePercent !== undefined && (
                        <div className="report-card">
                          <div className="report-card-label">MoM Expense Trend</div>
                          <div className="report-card-val font-mono">
                            {financialSummary.momExpenseChangePercent > 0 ? "+" : ""}
                            {financialSummary.momExpenseChangePercent.toFixed(1)}%
                          </div>
                        </div>
                      )}
                    </div>
                  )}

                  <div className="breakdown-section">
                    <h5>Monthly Breakdown</h5>
                    {(!financialSummary || financialSummary.monthlyBreakdown.length === 0) ? (
                      <p className="text-muted text-small">No monthly records to display.</p>
                    ) : (
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>Month</th>
                            <th>Transactions</th>
                            <th>Total Income</th>
                            <th>Total Expenses</th>
                            <th>Net Cash Flow</th>
                          </tr>
                        </thead>
                        <tbody>
                          {financialSummary.monthlyBreakdown.map((m) => (
                            <tr key={m.monthKey}>
                              <td className="font-mono">{m.monthKey}</td>
                              <td>{m.transactionCount}</td>
                              <td className="font-mono amount-credit">
                                {formatMinorCurrency(m.totalIncomeMinor, "USD")}
                              </td>
                              <td className="font-mono amount-debit">
                                {formatMinorCurrency(m.totalExpenseMinor, "USD")}
                              </td>
                              <td
                                className={`font-mono font-medium ${
                                  m.netCashFlowMinor >= 0 ? "amount-credit" : "amount-debit"
                                }`}
                              >
                                {formatMinorCurrency(m.netCashFlowMinor, "USD")}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    )}
                  </div>
                </div>
              )}

              {activeTab === "settings" && (
                <div className="settings-view">
                  <div className="panel-header">
                    <h4>Security Boundary & Offline Runtime Status</h4>
                  </div>

                  <div className="security-grid">
                    <div className="security-card">
                      <div className="card-title">
                        <HardDrive size={18} />
                        <span>Storage & Keys</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Database</span>
                        <span className="spec-val">SQLite 3 + WAL Journaling</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Key Derivation</span>
                        <span className="spec-val">Argon2id (64 MiB RAM, 3 iter)</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Blob Encryption</span>
                        <span className="spec-val">AES-256-GCM Authenticated</span>
                      </div>
                    </div>

                    <div className="security-card">
                      <div className="card-title">
                        <ShieldCheck size={18} />
                        <span>Air-Gap Network Boundary</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">CSP Rule</span>
                        <span className="spec-val">connect-src 'none'</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Telemetry</span>
                        <span className="spec-val">Permanently Disabled</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Updater</span>
                        <span className="spec-val">No Remote Calls</span>
                      </div>
                    </div>

                    <div className="security-card">
                      <div className="card-title">
                        <Cpu size={18} />
                        <span>Local Document Inference</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">App Version</span>
                        <span className="spec-val">v{status?.appVersion || "0.1.0"} ({status?.platform || "linux"})</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">OCR Engine</span>
                        <span className="spec-val">RapidOCR (ONNXRuntime CPU AVX2)</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Digital Parser</span>
                        <span className="spec-val">PyMuPDF / pdfplumber</span>
                      </div>
                      <div className="spec-row">
                        <span className="spec-label">Local LLM</span>
                        <span className="spec-val">{status?.llmModelId || "qwen-2.5-3b-instruct-q3_k_m"}</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
