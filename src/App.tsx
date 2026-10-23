import React, { useEffect, useState, useMemo, useCallback } from "react";
import {
  SystemStatus,
  WorkspaceInfo,
  WorkspaceSummary,
  DocumentRecord,
  TransactionRecord,
  DocumentDetails,
  ProposedLink,
  FinancialAnomaly,
  FinancialSummary,
  CategoryProposal,
  SearchResult,
  DocumentPreview,
  AuditEventRecord,
  ExportResult,
  ChatMessage,
  ChatMessageRecord,
  ChatThread,
  ChatThreadRecord,
} from "./lib/types";
import { SaarApi } from "./lib/api";
import { Sidebar, WorkspaceTab } from "./components/Sidebar";
import { HeaderBar } from "./components/HeaderBar";
import { UnlockScreen } from "./components/UnlockScreen";
import { ChatWorkspace } from "./components/ChatWorkspace";
import { EvidenceInspector } from "./components/EvidenceInspector";
import { DocumentsView } from "./components/DocumentsView";
import { ReviewView } from "./components/ReviewView";
import { ReportsView } from "./components/ReportsView";
import { SecurityView } from "./components/SecurityView";

export const App: React.FC = () => {
  // Theme state: "dark" (#111844 canvas) or "light" (#EAE0CF warm linen canvas)
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    return (localStorage.getItem("saar_theme") as "dark" | "light") || "dark";
  });

  const handleToggleTheme = () => {
    setTheme((prev) => {
      const next = prev === "dark" ? "light" : "dark";
      localStorage.setItem("saar_theme", next);
      return next;
    });
  };

  // Navigation & Core State
  const [activeTab, setActiveTab] = useState<WorkspaceTab>("chat");
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);

  // Authentication state
  const [workspaces, setWorkspaces] = useState<WorkspaceSummary[]>([]);
  const [authError, setAuthError] = useState<string | null>(null);
  const [isAuthLoading, setIsAuthLoading] = useState(false);

  // Multi-Chat Threads State (threads and messages are persisted in the encrypted workspace)
  const [threads, setThreads] = useState<ChatThread[]>([]);
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null);
  const [isChatLoading, setIsChatLoading] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);

  // Workspace Records Data
  const [documents, setDocuments] = useState<DocumentRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [reconciliationLinks, setReconciliationLinks] = useState<ProposedLink[]>([]);
  const [anomalies, setAnomalies] = useState<FinancialAnomaly[]>([]);
  const [financialSummary, setFinancialSummary] = useState<FinancialSummary | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditEventRecord[]>([]);
  const [reportExports, setReportExports] = useState<ExportResult[]>([]);
  const [proposals, setProposals] = useState<Record<string, CategoryProposal>>({});

  // Contextual Evidence Inspector State
  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [selectedDocDetails, setSelectedDocDetails] = useState<DocumentDetails | null>(null);
  const [docPreview, setDocPreview] = useState<DocumentPreview | null>(null);
  const [selectedTx, setSelectedTx] = useState<TransactionRecord | null>(null);
  const [selectedLink, setSelectedLink] = useState<ProposedLink | null>(null);

  // Global FTS5 Search State
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  // Operational Action States
  const [isImporting, setIsImporting] = useState(false);
  const [importMessage, setImportMessage] = useState<{
    text: string;
    type: "success" | "error" | "info";
  } | null>(null);
  const [isReconciling, setIsReconciling] = useState(false);
  const [reviewMessage, setReviewMessage] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [exportFeedback, setExportFeedback] = useState<string | null>(null);

  // Backup & Restore State
  const [backupPassphrase, setBackupPassphrase] = useState("");
  const [isBackupLoading, setIsBackupLoading] = useState(false);
  const [backupFeedback, setBackupFeedback] = useState<string | null>(null);
  const [restoreArchivePath, setRestoreArchivePath] = useState("");
  const [restorePassphrase, setRestorePassphrase] = useState("");
  const [isRestoreLoading, setIsRestoreLoading] = useState(false);
  const [restoreFeedback, setRestoreFeedback] = useState<string | null>(null);

  // Check system status and the workspaces that can be unlocked on mount
  useEffect(() => {
    SaarApi.getSystemStatus().then(setStatus).catch(console.error);
    SaarApi.listWorkspaces().then(setWorkspaces).catch(console.error);
  }, []);

  // Keyboard shortcut Ctrl+N for new chat, Ctrl+K for search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "n") {
        e.preventDefault();
        handleNewChat();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [workspace]);

  const activeThread = useMemo(() => {
    return threads.find((t) => t.id === activeThreadId) || null;
  }, [threads, activeThreadId]);

  // Converts persisted thread records into the shape the sidebar renders.
  const toChatThread = (record: ChatThreadRecord, messages: ChatMessage[] = []): ChatThread => ({
    id: record.id,
    title: record.title,
    pinned: record.pinned,
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
    messages,
  });

  const toChatMessage = (record: ChatMessageRecord): ChatMessage => ({
    id: record.id,
    sender: record.sender,
    text: record.text,
    response: record.response ?? undefined,
    timestamp: new Date(record.createdAt).toLocaleTimeString(),
  });

  const loadThreadMessages = useCallback(async (threadId: string) => {
    const records = await SaarApi.listChatMessages(threadId);
    const messages = records.map(toChatMessage);
    setThreads((prev) => prev.map((t) => (t.id === threadId ? { ...t, messages } : t)));
  }, []);

  const loadThreads = useCallback(async () => {
    const records = await SaarApi.listChatThreads();
    const loaded = records.map((r) => toChatThread(r));
    setThreads(loaded);
    return loaded;
  }, []);

  // Load the persisted conversation list whenever a workspace is unlocked.
  useEffect(() => {
    if (!workspace) return;
    loadThreads()
      .then((loaded) => {
        const first = loaded[0];
        if (first) {
          setActiveThreadId(first.id);
          return loadThreadMessages(first.id);
        }
        setActiveThreadId(null);
      })
      .catch((err) => console.error("Failed to load conversations:", err));
  }, [workspace, loadThreads, loadThreadMessages]);

  const refreshWorkspaceData = async (baseCurrency: string = workspace?.baseCurrency ?? "USD") => {
    try {
      const [docs, txs, links, anoms, summary, logs, exports] = await Promise.all([
        SaarApi.listDocuments(),
        SaarApi.listTransactions(),
        SaarApi.listReconciliationLinks(),
        SaarApi.listAnomalies(),
        SaarApi.getFinancialSummary(baseCurrency),
        SaarApi.listAuditLog(),
        SaarApi.listReportExports(),
      ]);
      setDocuments(docs);
      setTransactions(txs);
      setReconciliationLinks(links);
      setAnomalies(anoms);
      setFinancialSummary(summary);
      setAuditLogs(logs);
      setReportExports(exports);
    } catch (err) {
      console.error("Refresh workspace data error:", err);
    }
  };

  // Auth Handler
  // Unlocks an existing workspace, or creates the first one when none exists yet.
  const handleUnlock = async (
    mode:
      | { kind: "unlock"; workspaceId: string }
      | { kind: "create"; displayName: string; baseCurrency: string },
    passphrase: string,
  ) => {
    setIsAuthLoading(true);
    setAuthError(null);
    try {
      const ws =
        mode.kind === "unlock"
          ? await SaarApi.unlockWorkspace(mode.workspaceId, passphrase)
          : await SaarApi.createWorkspace(mode.displayName, passphrase, mode.baseCurrency);
      setWorkspace(ws);
      setWorkspaces(await SaarApi.listWorkspaces());
      await refreshWorkspaceData(ws.baseCurrency);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setAuthError(`Authentication failure: ${msg}`);
    } finally {
      setIsAuthLoading(false);
    }
  };

  const handleLockWorkspace = async () => {
    await SaarApi.lockWorkspace();
    setWorkspace(null);
    setThreads([]);
    setActiveThreadId(null);
    setChatError(null);
    setDocuments([]);
    setTransactions([]);
    setReconciliationLinks([]);
    setAnomalies([]);
    setFinancialSummary(null);
    setSelectedDocDetails(null);
    setDocPreview(null);
    setSelectedTx(null);
    setSelectedLink(null);
    setIsInspectorOpen(false);
  };

  // Multi-Chat Thread Management
  const reportError = (err: unknown, setter: (msg: string) => void, prefix: string) => {
    const msg = err instanceof Error ? err.message : String(err);
    setter(`${prefix}: ${msg}`);
  };

  const handleNewChat = async () => {
    setActiveTab("chat");
    setChatError(null);
    try {
      const record = await SaarApi.createChatThread();
      setThreads((prev) => [toChatThread(record), ...prev]);
      setActiveThreadId(record.id);
    } catch (err) {
      reportError(err, setChatError, "Could not start a conversation");
    }
  };

  const handleSelectThread = async (threadId: string) => {
    setActiveThreadId(threadId);
    setChatError(null);
    try {
      await loadThreadMessages(threadId);
    } catch (err) {
      reportError(err, setChatError, "Could not load this conversation");
    }
  };

  const handleDeleteThread = async (threadId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await SaarApi.deleteChatThread(threadId);
    } catch (err) {
      reportError(err, setChatError, "Could not delete this conversation");
      return;
    }
    const remaining = threads.filter((t) => t.id !== threadId);
    setThreads(remaining);
    if (activeThreadId === threadId) {
      const next = remaining[0];
      setActiveThreadId(next ? next.id : null);
      if (next) {
        await loadThreadMessages(next.id);
      }
    }
  };

  const handleTogglePinThread = async (threadId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const current = threads.find((t) => t.id === threadId);
    if (!current) return;
    try {
      await SaarApi.setChatThreadPinned(threadId, !current.pinned);
      const records = await SaarApi.listChatThreads();
      setThreads((prev) =>
        records.map((r) => ({
          ...toChatThread(r),
          messages: prev.find((t) => t.id === r.id)?.messages ?? [],
        })),
      );
    } catch (err) {
      reportError(err, setChatError, "Could not update this conversation");
    }
  };

  // Chat Query Handler with Real Local AI & Provenance Tracking
  const handleSendMessage = async (query: string) => {
    setChatError(null);
    setIsChatLoading(true);
    let threadId: string | null = activeThreadId;
    try {
      // Start a persisted conversation on the first question if none is open.
      if (!threadId) {
        const record = await SaarApi.createChatThread();
        threadId = record.id;
        setThreads((prev) => [toChatThread(record), ...prev]);
        setActiveThreadId(threadId);
      }

      // The answer is generated and stored by the backend; the reloaded transcript is authoritative.
      await SaarApi.chatGrounded(threadId, query);
      await loadThreadMessages(threadId);
      const records = await SaarApi.listChatThreads();
      setThreads((prev) =>
        records.map((r) => ({
          ...toChatThread(r),
          messages: prev.find((t) => t.id === r.id)?.messages ?? [],
        })),
      );
    } catch (err: unknown) {
      reportError(err, setChatError, "The local model could not answer");
      if (threadId) {
        await loadThreadMessages(threadId).catch(() => undefined);
      }
    } finally {
      setIsChatLoading(false);
    }
  };

  // Document Inspection
  const loadDocumentDetails = async (docId: string) => {
    try {
      const details = await SaarApi.getDocumentDetails(docId);
      setSelectedDocDetails(details);
      const preview = await SaarApi.getDocumentPreview(docId);
      setDocPreview(preview);
      setSelectedTx(null);
      setSelectedLink(null);
      setIsInspectorOpen(true);
    } catch (err) {
      console.error("Failed to load document evidence:", err);
    }
  };

  const handleSelectCitation = async (citation: string) => {
    const cleaned = citation.replace(/^\[|\]$/g, "");
    const parts = cleaned.split(":");
    const type = parts[0];
    const id = parts[1] || parts[0];

    if (type === "inv" || type === "doc") {
      await loadDocumentDetails(id);
    } else if (type === "tx") {
      const tx = transactions.find((t) => t.id === id);
      if (tx) {
        setSelectedTx(tx);
        setSelectedDocDetails(null);
        setDocPreview(null);
        setSelectedLink(null);
        setIsInspectorOpen(true);
      }
    }
  };

  // Ingestion Handler
  const handleIngestFile = async (filePath: string) => {
    if (!filePath.trim()) return;
    setIsImporting(true);
    setImportMessage({ text: `Ingesting document from ${filePath}...`, type: "info" });
    try {
      const res = await SaarApi.ingestFile(filePath.trim());
      if (res.type === "success") {
        setImportMessage({
          text: `Successfully ingested "${res.fileName}" (${res.docType}) — ${res.recordsExtracted} records extracted.`,
          type: "success",
        });
        await refreshWorkspaceData();
        await loadDocumentDetails(res.documentId);
      } else {
        setImportMessage({
          text: `Deduplication: "${res.fileName}" already stored in encrypted workspace (SHA-256: ${res.sha256.substring(0, 16)}...).`,
          type: "info",
        });
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setImportMessage({ text: `Ingestion failed: ${msg}`, type: "error" });
    } finally {
      setIsImporting(false);
    }
  };

  // Reconciliation Handlers
  const handleRunReconciliation = async () => {
    setIsReconciling(true);
    setReviewMessage("Executing deterministic matching rules & anomaly detection...");
    try {
      const res = await SaarApi.runReconciliation();
      setReconciliationLinks(res.links);
      setAnomalies(res.anomalies);
      setReviewMessage(
        `Reconciliation complete: ${res.links.length} proposed links found, ${res.anomalies.length} anomalies flagged.`
      );
      await refreshWorkspaceData();
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
      setReviewMessage("Match link approved and audit trail logged.");
      await refreshWorkspaceData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Approval failed: ${msg}`);
    }
  };

  const handleRejectLink = async (linkId: string) => {
    try {
      await SaarApi.rejectReconciliationLink(linkId);
      setReviewMessage("Match link rejected and logged to audit trail.");
      await refreshWorkspaceData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Rejection failed: ${msg}`);
    }
  };

  // Classification & Correction Handlers
  const handleClassifyTransaction = async (txId: string) => {
    try {
      const proposal = await SaarApi.classifyTransaction(txId);
      setProposals((prev) => ({ ...prev, [txId]: proposal }));
    } catch (err) {
      console.error("Classification error:", err);
    }
  };

  const handleApproveCategory = async (txId: string) => {
    const prop = proposals[txId];
    if (!prop) return;
    try {
      await SaarApi.approveCategory(prop.id);
      setProposals((prev) => ({
        ...prev,
        [txId]: { ...prop, status: "APPROVED" },
      }));
      await refreshWorkspaceData();
    } catch (err) {
      console.error("Approve category error:", err);
    }
  };

  const handleSaveCorrection = async (txId: string, newDesc: string) => {
    try {
      await SaarApi.correctTransaction(txId, newDesc);
      setReviewMessage(`Transaction description updated and audited: "${newDesc}"`);
      await refreshWorkspaceData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setReviewMessage(`Correction error: ${msg}`);
    }
  };

  // Export & Backup Handlers
  const handleExportReport = async (format: "pdf" | "csv") => {
    setIsExporting(true);
    setExportFeedback(null);
    try {
      const res = await SaarApi.exportReconciliationReport(format);
      setExportFeedback(
        `Generated: ${res.filePath} (${res.recordCount} records, Snapshot SHA-256: ${res.recordSnapshotHash.slice(0, 16)}...)`
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
    if (!backupPassphrase) return;
    setIsBackupLoading(true);
    setBackupFeedback(null);
    try {
      const manifest = await SaarApi.createEncryptedBackup(backupPassphrase);
      setBackupFeedback(
        `Encrypted archive created! Blobs: ${manifest.blobCount}, DB SHA: ${manifest.dbSha256.slice(0, 16)}..., Size: ${(manifest.totalRawBytes / 1024).toFixed(0)} KB`
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
    if (!restoreArchivePath || !restorePassphrase) return;
    setIsRestoreLoading(true);
    setRestoreFeedback(null);
    try {
      const manifest = await SaarApi.restoreEncryptedBackup(restoreArchivePath, restorePassphrase);
      setRestoreFeedback(
        `Restored archive successfully! Workspace: ${manifest.workspaceName}, Blobs: ${manifest.blobCount}`
      );
      await refreshWorkspaceData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setRestoreFeedback(`Restore failed: ${msg}`);
    } finally {
      setIsRestoreLoading(false);
    }
  };

  // Global Search Handler
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
      loadDocumentDetails(res.entityId);
    } else if (res.entityType === "TRANSACTION") {
      const tx = transactions.find((t) => t.id === res.entityId);
      if (tx) {
        setSelectedTx(tx);
        setSelectedDocDetails(null);
        setDocPreview(null);
        setSelectedLink(null);
        setIsInspectorOpen(true);
      }
    }
  };

  // Starter questions are derived from records that actually exist in this workspace.
  // Nothing is shown for an empty category, and the model answers every question itself.
  const chatSuggestions = useMemo(() => {
    const suggestions: string[] = [];
    if (documents.length > 0) {
      suggestions.push(`Summarize the ${documents.length} imported documents`);
    }
    if (anomalies.length > 0) {
      suggestions.push(
        `Explain the ${anomalies.length} flagged ${anomalies.length === 1 ? "anomaly" : "anomalies"}`,
      );
    }
    if (reconciliationLinks.length > 0) {
      suggestions.push(`Which ${reconciliationLinks.length} proposed matches need review, and why?`);
    }
    if (transactions.length > 0) {
      suggestions.push("What were my largest expenses in the recorded period?");
    }
    if (financialSummary) {
      suggestions.push(`What is my net savings in ${workspace?.baseCurrency ?? "USD"}?`);
    }
    return suggestions.slice(0, 4);
  }, [documents, anomalies, reconciliationLinks, transactions, financialSummary, workspace]);

  // Header Title
  const activeTabTitle = useMemo(() => {
    if (activeTab === "chat") {
      return activeThread?.title || "Conversation Workspace";
    }
    const titles: Record<WorkspaceTab, string> = {
      chat: "Conversation",
      documents: "Document Archive & Ingestion",
      review: "Reconciliation & Ledger Postings",
      reports: "Deterministic Financial Reports",
      security: "Vault Security & Cryptography",
    };
    return titles[activeTab];
  }, [activeTab, activeThread]);

  const hasInspectorContent = Boolean(selectedDocDetails || selectedTx || selectedLink);

  return (
    <div className="saar-app-root" data-theme={theme}>
      {!workspace ? (
        <UnlockScreen
          status={status}
          workspaces={workspaces}
          onUnlock={handleUnlock}
          isLoading={isAuthLoading}
          authError={authError}
          theme={theme}
          onToggleTheme={handleToggleTheme}
        />
      ) : (
        <div className="claude-desktop-shell">
          {/* 1. Left Sidebar Navigation (260px) */}
          <Sidebar
            workspace={workspace}
            activeTab={activeTab}
            onSelectTab={setActiveTab}
            threads={threads}
            activeThreadId={activeThreadId}
            onSelectThread={handleSelectThread}
            onNewChat={handleNewChat}
            onDeleteThread={handleDeleteThread}
            onTogglePinThread={handleTogglePinThread}
            pendingReviewCount={reconciliationLinks.length}
            documentCount={documents.length}
            onLockWorkspace={handleLockWorkspace}
            theme={theme}
            onToggleTheme={handleToggleTheme}
          />

          {/* 2. Main Workstation Viewport */}
          <div className="claude-main-viewport">
            {/* Top Header Bar */}
            <HeaderBar
              workspace={workspace}
              activeTabTitle={activeTabTitle}
              searchQuery={searchQuery}
              onSearchChange={handleSearch}
              searchResults={searchResults}
              isSearching={isSearching}
              onSelectSearchResult={handleSelectSearchResult}
              isInspectorOpen={isInspectorOpen}
              onToggleInspector={() => setIsInspectorOpen(!isInspectorOpen)}
              hasInspectorContent={hasInspectorContent}
              theme={theme}
              onToggleTheme={handleToggleTheme}
            />

            {/* Split Viewport: Central Stage + Contextual Evidence Inspector Drawer */}
            <div className="claude-split-body">
              <main className="claude-central-stage">
                {activeTab === "chat" && (
                  <ChatWorkspace
                    messages={activeThread?.messages || []}
                    onSendMessage={handleSendMessage}
                    isLoading={isChatLoading}
                    errorMessage={chatError}
                    suggestions={chatSuggestions}
                    onSelectCitation={handleSelectCitation}
                    onNavigateToTab={(tab) => {
                      if (tab === "documents") setActiveTab("documents");
                      else if (tab === "review") setActiveTab("review");
                      else if (tab === "reports") setActiveTab("reports");
                    }}
                    onIngestFile={handleIngestFile}
                  />
                )}

                {activeTab === "documents" && (
                  <DocumentsView
                    documents={documents}
                    selectedDocumentId={selectedDocDetails?.document.id || null}
                    onSelectDocument={loadDocumentDetails}
                    onIngestFile={handleIngestFile}
                    isImporting={isImporting}
                    importMessage={importMessage}
                  />
                )}

                {activeTab === "review" && (
                  <ReviewView
                    reconciliationLinks={reconciliationLinks}
                    anomalies={anomalies}
                    transactions={transactions}
                    auditLogs={auditLogs}
                    proposals={proposals}
                    isReconciling={isReconciling}
                    reviewMessage={reviewMessage}
                    onRunReconciliation={handleRunReconciliation}
                    onApproveLink={handleApproveLink}
                    onRejectLink={handleRejectLink}
                    onClassifyTransaction={handleClassifyTransaction}
                    onApproveCategory={handleApproveCategory}
                    onSaveCorrection={handleSaveCorrection}
                    onSelectTransaction={(tx) => {
                      setSelectedTx(tx);
                      setSelectedDocDetails(null);
                      setDocPreview(null);
                      setSelectedLink(null);
                      setIsInspectorOpen(true);
                    }}
                    onSelectLink={(link) => {
                      setSelectedLink(link);
                      if (link.invoiceId) {
                        loadDocumentDetails(link.invoiceId);
                      }
                      const matchedTx = transactions.find((t) => t.id === link.transactionId);
                      if (matchedTx) setSelectedTx(matchedTx);
                      setIsInspectorOpen(true);
                    }}
                  />
                )}

                {activeTab === "reports" && (
                  <ReportsView
                    financialSummary={financialSummary}
                    reportExports={reportExports}
                    isExporting={isExporting}
                    exportFeedback={exportFeedback}
                    onExportReport={handleExportReport}
                  />
                )}

                {activeTab === "security" && (
                  <SecurityView
                    status={status}
                    backupPassphrase={backupPassphrase}
                    setBackupPassphrase={setBackupPassphrase}
                    isBackupLoading={isBackupLoading}
                    backupFeedback={backupFeedback}
                    onCreateBackup={handleCreateBackup}
                    restoreArchivePath={restoreArchivePath}
                    setRestoreArchivePath={setRestoreArchivePath}
                    restorePassphrase={restorePassphrase}
                    setRestorePassphrase={setRestorePassphrase}
                    isRestoreLoading={isRestoreLoading}
                    restoreFeedback={restoreFeedback}
                    onRestoreBackup={handleRestoreBackup}
                  />
                )}
              </main>

              {/* 3. Right Contextual Evidence Drawer */}
              <EvidenceInspector
                isOpen={isInspectorOpen}
                onClose={() => setIsInspectorOpen(false)}
                documentDetails={selectedDocDetails}
                documentPreview={docPreview}
                selectedTransaction={selectedTx}
                selectedLink={selectedLink}
                onApproveLink={handleApproveLink}
                onRejectLink={handleRejectLink}
                onStartCorrection={(_tx) => {
                  setActiveTab("review");
                }}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
