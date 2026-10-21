import React, { useEffect, useState } from "react";
import {
  SystemStatus,
  WorkspaceInfo,
  DocumentRecord,
  TransactionRecord,
  DocumentDetails,
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
} from "lucide-react";

export const App: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [activeTab, setActiveTab] = useState<"documents" | "transactions" | "settings">(
    "documents"
  );
  const [documents, setDocuments] = useState<DocumentRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [selectedDocDetails, setSelectedDocDetails] = useState<DocumentDetails | null>(null);

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
        // Automatically select newly ingested document
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
            <span>Ledger Transactions</span>
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
                              {formatMinorCurrency(
                                selectedDocDetails.invoiceTotalMinor,
                                "USD" // display currency
                              )}
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
                              <span className="status-pill status-unreviewed">
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
