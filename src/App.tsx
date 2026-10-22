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
  GroundedChatResponse,
  CategoryProposal,
  SearchResult,
  NormalizedField,
  DocumentPreview,
  AuditEventRecord,
  ExportResult,
  formatMinorCurrency,
} from "./lib/types";
import { SaarApi } from "./lib/api";
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
  Sparkles,
  Search,
  Tag,
  Send,
  Download,
  Archive,
  FolderDown,
} from "lucide-react";

export const App: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [activeTab, setActiveTab] = useState<
    "documents" | "transactions" | "reconciliation" | "reports" | "chat" | "settings"
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

  // Grounded Chat state
  const [chatMessages, setChatMessages] = useState<
    Array<{
      sender: "user" | "assistant";
      text: string;
      response?: GroundedChatResponse;
      timestamp: string;
    }>
  >([
    {
      sender: "assistant",
      text: "SAAR Grounded Financial Assistant is online. All inference runs locally via Qwen2.5-3B on AVX2 CPU with deterministic greedy decoding and provenance citation enforcement. Ask any question about your local financial records.",
      timestamp: new Date().toLocaleTimeString(),
    },
  ]);
  const [chatInput, setChatInput] = useState<string>("");
  const [isChatLoading, setIsChatLoading] = useState<boolean>(false);

  // Search state
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState<boolean>(false);

  // Category Proposals state
  const [proposals, setProposals] = useState<Record<string, CategoryProposal>>({});
  const [classifyingTxId, setClassifyingTxId] = useState<string | null>(null);

  // Evidence Inspector Visual Preview & Bounding Box Highlighting state
  const [docPreview, setDocPreview] = useState<DocumentPreview | null>(null);
  const [activeField, setActiveField] = useState<NormalizedField | null>(null);

  // Audit Log & Correction Flow state
  const [auditLogs, setAuditLogs] = useState<AuditEventRecord[]>([]);
  const [correctingTx, setCorrectingTx] = useState<TransactionRecord | null>(null);
  const [correctionInput, setCorrectionInput] = useState<string>("");

  // Report Export state
  const [reportExports, setReportExports] = useState<ExportResult[]>([]);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [exportFeedback, setExportFeedback] = useState<string | null>(null);

  // Backup & Restore state
  const [backupPassphrase, setBackupPassphrase] = useState<string>("");
  const [isBackupLoading, setIsBackupLoading] = useState<boolean>(false);
  const [backupFeedback, setBackupFeedback] = useState<string | null>(null);
  const [restoreArchivePath, setRestoreArchivePath] = useState<string>("");
  const [restorePassphrase, setRestorePassphrase] = useState<string>("");
  const [isRestoreLoading, setIsRestoreLoading] = useState<boolean>(false);
  const [restoreFeedback, setRestoreFeedback] = useState<string | null>(null);

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
    SaarApi.getSystemStatus().then(setStatus).catch(console.error);
  }, []);

  const refreshData = async () => {
    try {
      const docs = await SaarApi.listDocuments();
      setDocuments(docs);
      const txs = await SaarApi.listTransactions();
      setTransactions(txs);
      const links = await SaarApi.listReconciliationLinks();
      setReconciliationLinks(links);
      const anoms = await SaarApi.listAnomalies();
      setAnomalies(anoms);
      const summary = await SaarApi.getFinancialSummary("USD");
      setFinancialSummary(summary);
      const logs = await SaarApi.listAuditLog();
      setAuditLogs(logs);
      const exports = await SaarApi.listReportExports();
      setReportExports(exports);
    } catch (e) {
      console.error("Refresh failed:", e);
    }
  };

  const handleExportReport = async (format: "pdf" | "csv") => {
    setIsExporting(true);
    setExportFeedback(null);
    try {
      const res = await SaarApi.exportReconciliationReport(format);
      setExportFeedback(
        `Export generated successfully: ${res.filePath} (${res.recordCount} records, Snapshot SHA-256: ${res.recordSnapshotHash.slice(0, 16)}...)`
      );
      const exports = await SaarApi.listReportExports();
      setReportExports(exports);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setExportFeedback(`Export error: ${msg}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleCreateBackup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!backupPassphrase) {
      setBackupFeedback("Backup passphrase cannot be empty.");
      return;
    }
    setIsBackupLoading(true);
    setBackupFeedback(null);
    try {
      const manifest = await SaarApi.createEncryptedBackup(backupPassphrase);
      setBackupFeedback(
        `Encrypted backup created successfully! Blobs: ${manifest.blobCount}, DB SHA: ${manifest.dbSha256.slice(0, 16)}..., Size: ${manifest.totalRawBytes} bytes`
      );
      setBackupPassphrase("");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setBackupFeedback(`Backup failed: ${msg}`);
    } finally {
      setIsBackupLoading(false);
    }
  };

  const handleRestoreBackup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!restoreArchivePath || !restorePassphrase) {
      setRestoreFeedback("Archive path and passphrase are required.");
      return;
    }
    setIsRestoreLoading(true);
    setRestoreFeedback(null);
    try {
      const manifest = await SaarApi.restoreEncryptedBackup(
        restoreArchivePath,
        restorePassphrase
      );
      setRestoreFeedback(
        `Workspace restored successfully! App: ${manifest.appVersion}, Blobs: ${manifest.blobCount}, DB SHA: ${manifest.dbSha256.slice(0, 16)}...`
      );
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setRestoreFeedback(`Restore failed: ${msg}`);
    } finally {
      setIsRestoreLoading(false);
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
      const ws = await SaarApi.createWorkspace(workspaceName, passphrase, "USD");
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
    await SaarApi.lockWorkspace();
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
      const res = await SaarApi.ingestFile(targetPath.trim());
      if (res.type === "success") {
        setImportMessage({
          text: `Successfully ingested "${res.fileName}" (${res.docType}) — ${res.recordsExtracted} items extracted.`,
          type: "success",
        });
        await refreshData();
        const details = await SaarApi.getDocumentDetails(res.documentId);
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
      const res = await SaarApi.runReconciliation();
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
      await SaarApi.approveReconciliationLink(linkId);
      setReviewMessage("Match link approved and audit trail recorded.");
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Approval failed: ${msg}`);
    }
  };

  const handleRejectLink = async (linkId: string) => {
    try {
      await SaarApi.rejectReconciliationLink(linkId, "User rejected match suggestion");
      setReviewMessage("Match link rejected and logged to audit trail.");
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Rejection failed: ${msg}`);
    }
  };

  const selectDocument = async (docId: string) => {
    try {
      const details = await SaarApi.getDocumentDetails(docId);
      setSelectedDocDetails(details);
      const preview = await SaarApi.getDocumentPreview(docId);
      setDocPreview(preview);
      setActiveField(null);
    } catch (err) {
      console.error("Failed to load details:", err);
    }
  };

  const handleSaveCorrection = async () => {
    if (!correctingTx || !correctionInput.trim()) return;
    try {
      await SaarApi.correctTransaction(correctingTx.id, correctionInput.trim());
      setReviewMessage(
        `Transaction ${correctingTx.id.substring(0, 8)} corrected to: "${correctionInput.trim()}"`
      );
      setCorrectingTx(null);
      setCorrectionInput("");
      await refreshData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Correction failed: ${msg}`);
    }
  };

  const handleSendMessage = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!chatInput.trim() || isChatLoading) return;

    const query = chatInput.trim();
    setChatInput("");
    const userMsg = {
      sender: "user" as const,
      text: query,
      timestamp: new Date().toLocaleTimeString(),
    };
    setChatMessages((prev) => [...prev, userMsg]);
    setIsChatLoading(true);

    try {
      const res = await SaarApi.chatGrounded(query);
      const assistantMsg = {
        sender: "assistant" as const,
        text: res.answer,
        response: res,
        timestamp: new Date().toLocaleTimeString(),
      };
      setChatMessages((prev) => [...prev, assistantMsg]);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setChatMessages((prev) => [
        ...prev,
        {
          sender: "assistant",
          text: `Local inference error: ${msg}`,
          timestamp: new Date().toLocaleTimeString(),
        },
      ]);
    } finally {
      setIsChatLoading(false);
    }
  };

  const handleSearch = async (term: string) => {
    setSearchQuery(term);
    if (!term.trim()) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    try {
      const results = await SaarApi.searchWorkspace(term.trim());
      setSearchResults(results);
    } catch (err) {
      console.error("Search error:", err);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelectSearchResult = (res: SearchResult) => {
    setSearchQuery("");
    setSearchResults([]);
    if (res.entityType === "DOCUMENT" || res.entityType === "INVOICE") {
      setActiveTab("documents");
      selectDocument(res.entityId);
    } else if (res.entityType === "TRANSACTION") {
      setActiveTab("transactions");
    }
  };

  const handleClassifyTransaction = async (txId: string) => {
    setClassifyingTxId(txId);
    try {
      const proposal = await SaarApi.classifyTransaction(txId);
      setProposals((prev) => ({ ...prev, [txId]: proposal }));
    } catch (err) {
      console.error("Classification error:", err);
    } finally {
      setClassifyingTxId(null);
    }
  };

  const handleApproveCategory = async (txId: string) => {
    const proposal = proposals[txId];
    if (!proposal) return;
    try {
      await SaarApi.approveCategory(proposal.id);
      setProposals((prev) => ({
        ...prev,
        [txId]: { ...proposal, status: "APPROVED" },
      }));
      await refreshData();
    } catch (err) {
      console.error("Approve category error:", err);
    }
  };

  return (
    <div className="app-container">
      {/* Editorial Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="brand-title">SAAR</div>
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
            className={`nav-item ${activeTab === "chat" ? "active" : ""}`}
            onClick={() => setActiveTab("chat")}
          >
            <Sparkles size={16} />
            <span>Grounded AI Chat</span>
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

          {workspace && (
            <div className="topbar-search-container">
              {isSearching ? (
                <RefreshCw size={14} className="topbar-search-icon spin" />
              ) : (
                <Search size={14} className="topbar-search-icon" />
              )}
              <input
                type="text"
                placeholder="FTS5 Fast Search across documents, transactions & invoices..."
                value={searchQuery}
                onChange={(e) => handleSearch(e.target.value)}
                className="topbar-search-input"
              />
              {searchResults.length > 0 && (
                <div className="search-dropdown-menu">
                  {searchResults.map((res, idx) => (
                    <div
                      key={idx}
                      className="search-dropdown-item"
                      onClick={() => handleSelectSearchResult(res)}
                    >
                      <div className="search-item-header">
                        <span className="search-item-type">{res.entityType}</span>
                        <span className="search-item-title">{res.title}</span>
                      </div>
                      <div
                        className="search-item-snippet"
                        dangerouslySetInnerHTML={{ __html: res.snippet }}
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

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

                        {/* Document Visual Preview Surface with Bounding Box Highlights */}
                        {docPreview && (
                          <div className="detail-section">
                            <h5>Document Preview & Coordinate Evidence</h5>
                            {docPreview.mimeType.startsWith("image/") && (
                              <div className="preview-container">
                                <div
                                  className="preview-image-wrapper"
                                  style={{
                                    position: "relative",
                                    overflow: "hidden",
                                    borderRadius: 6,
                                    border: "1px solid var(--border-subtle)",
                                  }}
                                >
                                  <img
                                    src={`data:${docPreview.mimeType};base64,${docPreview.dataBase64}`}
                                    alt="Document"
                                    style={{ width: "100%", display: "block" }}
                                  />
                                  {activeField &&
                                    (() => {
                                      try {
                                        const box = JSON.parse(activeField.boundingBoxJson);
                                        if (
                                          Array.isArray(box) &&
                                          box.length >= 4 &&
                                          typeof box[0] === "number"
                                        ) {
                                          return (
                                            <div
                                              className="bbox-overlay-highlight"
                                              style={{
                                                position: "absolute",
                                                left: `${box[0]}px`,
                                                top: `${box[1]}px`,
                                                width: `${Math.max(box[2] - box[0], 20)}px`,
                                                height: `${Math.max(box[3] - box[1], 15)}px`,
                                                border: "2px solid #f59e0b",
                                                backgroundColor: "rgba(245, 158, 11, 0.25)",
                                                pointerEvents: "none",
                                              }}
                                            />
                                          );
                                        }
                                      } catch {
                                        return null;
                                      }
                                    })()}
                                </div>
                              </div>
                            )}

                            {docPreview.mimeType === "application/pdf" && (
                              <div className="preview-container">
                                <object
                                  data={`data:application/pdf;base64,${docPreview.dataBase64}`}
                                  type="application/pdf"
                                  width="100%"
                                  height="240px"
                                  style={{
                                    borderRadius: 6,
                                    border: "1px solid var(--border-subtle)",
                                  }}
                                >
                                  <p className="text-small text-muted">
                                    Digital PDF Decrypted & Ready.
                                  </p>
                                </object>
                              </div>
                            )}

                            {docPreview.mimeType === "text/csv" && docPreview.textPreview && (
                              <div className="preview-container">
                                <pre
                                  className="csv-preview-text"
                                  style={{
                                    maxHeight: "180px",
                                    overflowY: "auto",
                                    fontSize: "11px",
                                    background: "var(--bg-primary)",
                                    padding: "8px",
                                    borderRadius: 6,
                                  }}
                                >
                                  {docPreview.textPreview}
                                </pre>
                              </div>
                            )}
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
                                <div
                                  key={idx}
                                  className={`field-item ${
                                    activeField?.fieldKey === f.fieldKey ? "active-field" : ""
                                  }`}
                                  onClick={() => setActiveField(f)}
                                  onMouseEnter={() => setActiveField(f)}
                                  style={{ cursor: "pointer" }}
                                >
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

                  {correctingTx && (
                    <div
                      className="correction-modal-card"
                      style={{
                        background: "var(--bg-tertiary)",
                        padding: "16px",
                        borderRadius: 8,
                        marginBottom: "16px",
                        border: "1px solid var(--accent-primary)",
                      }}
                    >
                      <div className="section-title">
                        <h5>
                          Manual Audit Correction for Transaction:{" "}
                          <span className="font-mono text-muted">
                            {correctingTx.id.substring(0, 8)}...
                          </span>
                        </h5>
                      </div>
                      <p className="text-small text-muted">
                        Mutations are recorded in the immutable append-only audit log with
                        before/after state captures.
                      </p>
                      <div style={{ display: "flex", gap: "8px", marginTop: "12px" }}>
                        <input
                          type="text"
                          value={correctionInput}
                          onChange={(e) => setCorrectionInput(e.target.value)}
                          className="input-field"
                          style={{ flex: 1 }}
                          placeholder="Enter corrected description..."
                          autoFocus
                        />
                        <button className="btn-primary" onClick={handleSaveCorrection}>
                          <Check size={14} />
                          <span>Save & Log Audit</span>
                        </button>
                        <button
                          className="btn-secondary"
                          onClick={() => {
                            setCorrectingTx(null);
                            setCorrectionInput("");
                          }}
                        >
                          <X size={14} />
                          <span>Cancel</span>
                        </button>
                      </div>
                    </div>
                  )}

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
                          <th>Category</th>
                          <th>Status</th>
                          <th>AI Action</th>
                        </tr>
                      </thead>
                      <tbody>
                        {transactions.map((tx) => {
                          const proposal = proposals[tx.id];
                          return (
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
                                {proposal ? (
                                  <span className="category-tag">
                                    <Tag size={12} />
                                    {proposal.suggestedCategory}
                                  </span>
                                ) : (
                                  <span className="text-muted text-small">Unclassified</span>
                                )}
                              </td>
                              <td>
                                <span
                                  className={`status-pill status-${tx.reviewState.toLowerCase()}`}
                                >
                                  {tx.reviewState}
                                </span>
                              </td>
                              <td>
                                <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                                  {proposal ? (
                                    proposal.status === "APPROVED" ? (
                                      <span className="badge-match badge-exact">Approved</span>
                                    ) : (
                                      <div className="category-proposal-box">
                                        <button
                                          className="btn-action btn-approve"
                                          onClick={() => handleApproveCategory(tx.id)}
                                        >
                                          <Check size={12} />
                                          <span>
                                            Approve ({Math.round(proposal.confidenceScore * 100)}%)
                                          </span>
                                        </button>
                                      </div>
                                    )
                                  ) : (
                                    <button
                                      className="btn-classify"
                                      onClick={() => handleClassifyTransaction(tx.id)}
                                      disabled={classifyingTxId === tx.id}
                                    >
                                      <Sparkles
                                        size={12}
                                        className={classifyingTxId === tx.id ? "spin" : ""}
                                      />
                                      <span>
                                        {classifyingTxId === tx.id
                                          ? "Classifying..."
                                          : "Auto-Classify"}
                                      </span>
                                    </button>
                                  )}
                                  <button
                                    className="btn-action"
                                    style={{
                                      background: "var(--bg-tertiary)",
                                      border: "1px solid var(--border-subtle)",
                                      color: "var(--text-secondary)",
                                    }}
                                    onClick={() => {
                                      setCorrectingTx(tx);
                                      setCorrectionInput(tx.description);
                                    }}
                                  >
                                    Edit
                                  </button>
                                </div>
                              </td>
                            </tr>
                          );
                        })}
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
                          <div
                            key={anom.id}
                            className={`anomaly-card severity-${anom.severity.toLowerCase()}`}
                          >
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
                                <span
                                  className={`badge-match badge-${link.matchType.toLowerCase()}`}
                                >
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

                  {/* Immutable Audit Trail Section */}
                  <div className="audit-section" style={{ marginTop: 28 }}>
                    <div className="section-title">
                      <ShieldCheck size={16} className="icon-green" />
                      <h5>Immutable Audit Trail ({auditLogs.length} events logged)</h5>
                    </div>
                    {auditLogs.length === 0 ? (
                      <p className="text-small text-muted" style={{ marginTop: 8 }}>
                        No mutations recorded in audit log yet. Actions like match approval,
                        rejection, description correction, or category classification generate
                        permanent cryptographic audit records.
                      </p>
                    ) : (
                      <table className="data-table" style={{ marginTop: 8, fontSize: 12 }}>
                        <thead>
                          <tr>
                            <th>Timestamp</th>
                            <th>Action</th>
                            <th>Target Entity</th>
                            <th>Audit Evidence Details</th>
                          </tr>
                        </thead>
                        <tbody>
                          {auditLogs.map((log) => (
                            <tr key={log.id}>
                              <td className="font-mono text-muted">{log.createdAt}</td>
                              <td>
                                <span className="badge-match badge-exact">{log.action}</span>
                              </td>
                              <td className="font-mono">
                                {log.entityType} ({log.entityId.substring(0, 8)}...)
                              </td>
                              <td
                                className="font-mono text-muted"
                                style={{
                                  maxWidth: 320,
                                  overflow: "hidden",
                                  textOverflow: "ellipsis",
                                  whiteSpace: "nowrap",
                                }}
                              >
                                {log.payloadJson}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    )}
                  </div>
                </div>
              )}

              {activeTab === "reports" && (
                <div className="reports-view">
                  <div className="panel-header">
                    <h4>Deterministic Financial Reports & Source-Traceable Exports</h4>
                    <span className="text-small text-muted">
                      Exact minor-unit ledger aggregates (Zero floating-point rounding)
                    </span>
                  </div>

                  <div style={{ display: "flex", gap: 10, marginTop: 12, marginBottom: 16 }}>
                    <button
                      className="btn-secondary"
                      onClick={() => handleExportReport("pdf")}
                      disabled={isExporting}
                    >
                      <Download size={14} />
                      <span>
                        {isExporting ? "Generating PDF..." : "Export Source-Traceable PDF"}
                      </span>
                    </button>
                    <button
                      className="btn-secondary"
                      onClick={() => handleExportReport("csv")}
                      disabled={isExporting}
                    >
                      <FileText size={14} />
                      <span>{isExporting ? "Generating CSV..." : "Export Reconciliation CSV"}</span>
                    </button>
                  </div>

                  {exportFeedback && (
                    <div className="alert-banner info" style={{ marginBottom: 16 }}>
                      <CheckCircle2 size={16} />
                      <span className="font-mono text-small">{exportFeedback}</span>
                    </div>
                  )}

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

                  {financialSummary && financialSummary.monthlyBreakdown.length > 0 && (
                    <div
                      className="report-chart-container"
                      style={{
                        backgroundColor: "var(--bg-secondary)",
                        border: "1px solid var(--border-subtle)",
                        borderRadius: 8,
                        padding: 16,
                        margin: "20px 0",
                      }}
                    >
                      <div className="section-title" style={{ marginBottom: 12 }}>
                        <BarChart3 size={16} />
                        <h5>Monthly Cash Flow Trends (Deterministic Integer Arithmetic)</h5>
                      </div>
                      <svg
                        width="100%"
                        height="160"
                        viewBox="0 0 600 160"
                        style={{ overflow: "visible" }}
                      >
                        {financialSummary.monthlyBreakdown.map((m, idx) => {
                          const maxAmount = Math.max(
                            ...financialSummary.monthlyBreakdown.map((x) =>
                              Math.max(x.totalIncomeMinor, x.totalExpenseMinor)
                            ),
                            1
                          );
                          const xOffset = 60 + idx * 140;
                          const incomeHeight = Math.max(
                            Math.round((m.totalIncomeMinor / maxAmount) * 110),
                            4
                          );
                          const expenseHeight = Math.max(
                            Math.round((m.totalExpenseMinor / maxAmount) * 110),
                            4
                          );
                          return (
                            <g key={m.monthKey}>
                              {/* Income Bar */}
                              <rect
                                x={xOffset}
                                y={130 - incomeHeight}
                                width="28"
                                height={incomeHeight}
                                fill="#10b981"
                                rx="3"
                              />
                              <text
                                x={xOffset + 14}
                                y={125 - incomeHeight}
                                textAnchor="middle"
                                fill="#34d399"
                                fontSize="10"
                                fontFamily="monospace"
                              >
                                {formatMinorCurrency(m.totalIncomeMinor, "USD")}
                              </text>
                              {/* Expense Bar */}
                              <rect
                                x={xOffset + 34}
                                y={130 - expenseHeight}
                                width="28"
                                height={expenseHeight}
                                fill="#ef4444"
                                rx="3"
                              />
                              <text
                                x={xOffset + 48}
                                y={125 - expenseHeight}
                                textAnchor="middle"
                                fill="#f87171"
                                fontSize="10"
                                fontFamily="monospace"
                              >
                                {formatMinorCurrency(m.totalExpenseMinor, "USD")}
                              </text>
                              {/* Month Label */}
                              <text
                                x={xOffset + 31}
                                y="150"
                                textAnchor="middle"
                                fill="#a0a6b5"
                                fontSize="11"
                                fontFamily="sans-serif"
                              >
                                {m.monthKey}
                              </text>
                            </g>
                          );
                        })}
                      </svg>
                    </div>
                  )}

                  <div className="breakdown-section">
                    <h5>Monthly Breakdown</h5>
                    {!financialSummary || financialSummary.monthlyBreakdown.length === 0 ? (
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

                  <div className="breakdown-section" style={{ marginTop: 24 }}>
                    <h5>Historical Report Exports & Provenance Snapshots</h5>
                    {reportExports.length === 0 ? (
                      <p className="text-muted text-small">No reports exported yet.</p>
                    ) : (
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>Exported At</th>
                            <th>Format</th>
                            <th>File Location</th>
                            <th>Size</th>
                            <th>Snapshot SHA-256</th>
                          </tr>
                        </thead>
                        <tbody>
                          {reportExports.map((exp) => (
                            <tr key={exp.exportId}>
                              <td className="font-mono text-muted text-small">{exp.createdAt}</td>
                              <td>
                                <span className="badge-match badge-exact">{exp.exportFormat}</span>
                              </td>
                              <td
                                className="font-mono text-small"
                                style={{
                                  maxWidth: 260,
                                  overflow: "hidden",
                                  textOverflow: "ellipsis",
                                  whiteSpace: "nowrap",
                                }}
                              >
                                {exp.filePath}
                              </td>
                              <td className="font-mono text-small">
                                {(exp.fileSizeBytes / 1024).toFixed(1)} KB
                              </td>
                              <td className="font-mono text-small text-muted">
                                {exp.recordSnapshotHash.slice(0, 16)}...
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    )}
                  </div>
                </div>
              )}

              {activeTab === "chat" && (
                <div className="chat-container">
                  <div className="chat-header">
                    <div>
                      <h4>Local Grounded Financial Intelligence</h4>
                      <span className="text-small text-muted">
                        Air-gapped Qwen2.5-3B • Fact-Packet Provenance Citations •
                        Anti-Hallucination Guard
                      </span>
                    </div>
                    <div className="chat-model-badge">
                      <Sparkles size={13} />
                      <span>Deterministic Temp 0.0 • AVX2 CPU</span>
                    </div>
                  </div>

                  <div className="chat-messages-area">
                    {chatMessages.map((msg, idx) => (
                      <div key={idx} className={`chat-message-row ${msg.sender}`}>
                        {msg.sender === "user" ? (
                          <div className="chat-bubble-user">{msg.text}</div>
                        ) : (
                          <div className="chat-bubble-assistant">
                            {msg.response && (
                              <div className="chat-intent-tag">{msg.response.intent}</div>
                            )}
                            <div>{msg.text}</div>

                            {msg.response?.citations && msg.response.citations.length > 0 && (
                              <div className="chat-citations-row">
                                <span className="text-small text-muted">Sources Cited:</span>
                                {msg.response.citations.map((c, cIdx) => (
                                  <span key={cIdx} className="citation-pill">
                                    [{c}]
                                  </span>
                                ))}
                              </div>
                            )}

                            {msg.response && (
                              <div
                                className={`verification-note ${
                                  msg.response.grounded ? "grounded" : "warning"
                                }`}
                              >
                                {msg.response.grounded ? (
                                  <CheckCircle2 size={13} />
                                ) : (
                                  <AlertCircle size={13} />
                                )}
                                <span>{msg.response.verificationNote}</span>
                              </div>
                            )}
                          </div>
                        )}
                        <span
                          className="text-small text-muted"
                          style={{
                            marginTop: 4,
                            alignSelf: msg.sender === "user" ? "flex-end" : "flex-start",
                          }}
                        >
                          {msg.timestamp}
                        </span>
                      </div>
                    ))}

                    {isChatLoading && (
                      <div className="chat-message-row assistant">
                        <div className="chat-bubble-assistant">
                          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                            <RefreshCw size={14} className="spin" />
                            <span>Synthesizing fact-packet answer on local CPU cores...</span>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="chat-suggestions">
                    <span
                      className="text-small text-muted"
                      style={{ alignSelf: "center", marginRight: 4 }}
                    >
                      Suggestions:
                    </span>
                    <button
                      className="suggestion-chip"
                      onClick={() =>
                        setChatInput("What was the invoice total for CloudHosting Corp?")
                      }
                    >
                      "Invoice total for CloudHosting Corp?"
                    </button>
                    <button
                      className="suggestion-chip"
                      onClick={() => setChatInput("What were my total expenses in September?")}
                    >
                      "Total expenses in September?"
                    </button>
                    <button
                      className="suggestion-chip"
                      onClick={() =>
                        setChatInput("Are there any unlinked transactions or duplicates?")
                      }
                    >
                      "Unlinked transactions or duplicates?"
                    </button>
                  </div>

                  <form onSubmit={handleSendMessage} className="chat-input-row">
                    <input
                      type="text"
                      value={chatInput}
                      onChange={(e) => setChatInput(e.target.value)}
                      placeholder="Ask questions about your financial records (e.g. totals, vendors, invoices)..."
                      className="chat-text-input"
                      disabled={isChatLoading}
                    />
                    <button
                      type="submit"
                      className="btn-primary"
                      disabled={isChatLoading || !chatInput.trim()}
                    >
                      <Send size={14} />
                      <span>Query AI</span>
                    </button>
                  </form>
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
                        <span className="spec-val">
                          v{status?.appVersion || "0.1.0"} ({status?.platform || "linux"})
                        </span>
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
                        <span className="spec-val">
                          {status?.llmModelId || "qwen-2.5-3b-instruct-q3_k_m"}
                        </span>
                      </div>
                    </div>

                    <div className="security-card" style={{ gridColumn: "1 / -1", marginTop: 12 }}>
                      <div className="card-title">
                        <Archive size={18} />
                        <span>Encrypted Offline Backup & Disaster Recovery (.saar)</span>
                      </div>
                      <p className="text-small text-muted" style={{ marginBottom: 14 }}>
                        Create an authenticated, tamper-evident AES-256-GCM archive containing the
                        SQLite database and encrypted blob store. Protected by Argon2id memory-hard
                        key derivation.
                      </p>

                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
                        {/* Create Backup */}
                        <form
                          onSubmit={handleCreateBackup}
                          style={{ display: "flex", flexDirection: "column", gap: 10 }}
                        >
                          <h6>Create New Encrypted Backup Archive</h6>
                          <input
                            type="password"
                            className="input-text"
                            placeholder="Enter backup passphrase..."
                            value={backupPassphrase}
                            onChange={(e) => setBackupPassphrase(e.target.value)}
                            disabled={isBackupLoading}
                          />
                          <button
                            type="submit"
                            className="btn-primary"
                            disabled={isBackupLoading || !backupPassphrase.trim()}
                          >
                            <Archive size={14} />
                            <span>
                              {isBackupLoading
                                ? "Encrypting Archive..."
                                : "Create Encrypted Backup"}
                            </span>
                          </button>
                          {backupFeedback && (
                            <span
                              className="text-small font-mono"
                              style={{
                                color: backupFeedback.includes("failed") ? "#ef4444" : "#10b981",
                                marginTop: 4,
                              }}
                            >
                              {backupFeedback}
                            </span>
                          )}
                        </form>

                        {/* Restore Backup */}
                        <form
                          onSubmit={handleRestoreBackup}
                          style={{ display: "flex", flexDirection: "column", gap: 10 }}
                        >
                          <h6>Restore from Backup Archive</h6>
                          <input
                            type="text"
                            className="input-text"
                            placeholder="Absolute path to .saar file..."
                            value={restoreArchivePath}
                            onChange={(e) => setRestoreArchivePath(e.target.value)}
                            disabled={isRestoreLoading}
                          />
                          <input
                            type="password"
                            className="input-text"
                            placeholder="Passphrase used during backup..."
                            value={restorePassphrase}
                            onChange={(e) => setRestorePassphrase(e.target.value)}
                            disabled={isRestoreLoading}
                          />
                          <button
                            type="submit"
                            className="btn-secondary"
                            disabled={
                              isRestoreLoading ||
                              !restoreArchivePath.trim() ||
                              !restorePassphrase.trim()
                            }
                          >
                            <FolderDown size={14} />
                            <span>
                              {isRestoreLoading ? "Decrypting & Restoring..." : "Restore Workspace"}
                            </span>
                          </button>
                          {restoreFeedback && (
                            <span
                              className="text-small font-mono"
                              style={{
                                color: restoreFeedback.includes("failed") ? "#ef4444" : "#10b981",
                                marginTop: 4,
                              }}
                            >
                              {restoreFeedback}
                            </span>
                          )}
                        </form>
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
