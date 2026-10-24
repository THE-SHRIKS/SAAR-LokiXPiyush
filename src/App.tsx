import React, { useEffect, useState, useMemo, useCallback, useRef } from "react";
import { baseName, isSupportedImport, listenForDroppedFiles, pickImportFiles } from "./lib/files";
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
  InvoiceCurrencySummary,
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
import { ErrorBoundary } from "./components/ErrorBoundary";

export const App: React.FC = () => {
  // Theme state: "dark" or "light"
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

  // The sidebar can be closed; the choice is remembered.
  const [sidebarCollapsed, setSidebarCollapsed] = useState<boolean>(() => {
    try {
      return localStorage.getItem("saar_sidebar") === "closed";
    } catch {
      return false;
    }
  });
  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => {
      try {
        localStorage.setItem("saar_sidebar", prev ? "open" : "closed");
      } catch {
        /* storage unavailable: the choice just is not remembered */
      }
      return !prev;
    });
  }, []);

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
  const [invoiceSummary, setInvoiceSummary] = useState<InvoiceCurrencySummary[]>([]);
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
  const [isDraggingFiles, setIsDraggingFiles] = useState(false);
  const isImportingRef = useRef(false);
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
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "b") {
        e.preventDefault();
        toggleSidebar();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [workspace]);

  useEffect(() => {
    isImportingRef.current = isImporting;
  }, [isImporting]);

  // Files dropped anywhere on the window are imported once a workspace is unlocked.
  const importPathsRef = useRef<(paths: string[]) => Promise<void>>(async () => {});
  useEffect(() => {
    if (!workspace) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listenForDroppedFiles(setIsDraggingFiles, (paths) => void importPathsRef.current(paths))
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch((err) => console.error("Drag and drop is unavailable:", err));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [workspace]);

  // Opening a data screen always shows the current records.
  useEffect(() => {
    if (workspace && activeTab !== "chat") void refreshWorkspaceData();
  }, [activeTab]);

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
    // Each list loads on its own, so one failing call cannot leave every other screen stale.
    const [docs, txs, links, anoms, summary, logs, exports, invoices] = await Promise.allSettled([
      SaarApi.listDocuments(),
      SaarApi.listTransactions(),
      SaarApi.listReconciliationLinks(),
      SaarApi.listAnomalies(),
      SaarApi.getFinancialSummary(baseCurrency),
      SaarApi.listAuditLog(),
      SaarApi.listReportExports(),
      SaarApi.getInvoiceSummary(),
    ]);
    if (docs.status === "fulfilled") setDocuments(docs.value);
    if (txs.status === "fulfilled") setTransactions(txs.value);
    if (links.status === "fulfilled") setReconciliationLinks(links.value);
    if (anoms.status === "fulfilled") setAnomalies(anoms.value);
    if (summary.status === "fulfilled") setFinancialSummary(summary.value);
    if (logs.status === "fulfilled") setAuditLogs(logs.value);
    if (exports.status === "fulfilled") setReportExports(exports.value);
    if (invoices.status === "fulfilled") setInvoiceSummary(invoices.value);
    for (const r of [docs, txs, links, anoms, summary, logs, exports, invoices]) {
      if (r.status === "rejected") console.error("Refresh workspace data error:", r.reason);
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
    // The unlock screen lists workspaces from this state, so refresh it (for example after a restore).
    SaarApi.listWorkspaces().then(setWorkspaces).catch(console.error);
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

  // Ingestion
  type IngestOutcome = { kind: "ok" | "duplicate" | "failed"; text: string; documentId?: string };

  const ingestOne = async (filePath: string): Promise<IngestOutcome> => {
    try {
      const res = await SaarApi.ingestFile(filePath.trim());
      if (res.type === "success") {
        return {
          kind: "ok",
          documentId: res.documentId,
          text: `Successfully ingested "${res.fileName}" (${res.docType}) — ${res.recordsExtracted} records extracted.`,
        };
      }
      return {
        kind: "duplicate",
        text: `Deduplication: "${res.fileName}" already stored in encrypted workspace (SHA-256: ${res.sha256.substring(0, 16)}...).`,
      };
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      return { kind: "failed", text: `Ingestion failed: ${msg}` };
    }
  };

  const handleIngestFile = async (filePath: string) => {
    if (!filePath.trim()) return;
    setIsImporting(true);
    setImportMessage({ text: `Ingesting document from ${filePath}...`, type: "info" });
    const outcome = await ingestOne(filePath);
    setImportMessage({
      text: outcome.text,
      type: outcome.kind === "ok" ? "success" : outcome.kind === "duplicate" ? "info" : "error",
    });
    // A failed import is kept as a FAILED record; show it now, not after the next unlock.
    await refreshWorkspaceData();
    if (outcome.documentId) await loadDocumentDetails(outcome.documentId);
    setIsImporting(false);
  };

  // Imports files chosen in the system dialog or dropped on the window, one after another.
  const handleImportPaths = async (paths: string[]) => {
    if (isImportingRef.current) return;
    setActiveTab("documents");
    const supported = paths.filter(isSupportedImport);
    if (supported.length === 0) {
      setImportMessage({ text: "SAAR imports PDF, PNG, JPG and CSV files. None of the dropped files can be imported.", type: "error" });
      return;
    }
    if (paths.length === 1) {
      await handleIngestFile(supported[0]);
      return;
    }
    setIsImporting(true);
    let imported = 0;
    let duplicates = 0;
    const failures: string[] = [];
    let lastDocumentId: string | undefined;
    for (let i = 0; i < supported.length; i++) {
      setImportMessage({ text: `Importing ${i + 1} of ${supported.length}: ${baseName(supported[i])}`, type: "info" });
      const outcome = await ingestOne(supported[i]);
      if (outcome.kind === "ok") {
        imported++;
        lastDocumentId = outcome.documentId;
      } else if (outcome.kind === "duplicate") duplicates++;
      else failures.push(`${baseName(supported[i])}: ${outcome.text.replace("Ingestion failed: ", "")}`);
      await refreshWorkspaceData();
    }
    const skipped = paths.length - supported.length;
    const parts = [`Imported ${imported} of ${supported.length} files`];
    if (duplicates) parts.push(`${duplicates} already stored`);
    if (skipped) parts.push(`${skipped} unsupported skipped`);
    if (failures.length) parts.push(`${failures.length} failed (${failures.join("; ")})`);
    setImportMessage({ text: `${parts.join(", ")}.`, type: failures.length ? "error" : "success" });
    if (lastDocumentId) await loadDocumentDetails(lastDocumentId);
    setIsImporting(false);
  };

  importPathsRef.current = handleImportPaths;

  const handlePickFiles = async () => {
    try {
      const paths = await pickImportFiles();
      if (paths.length) await handleImportPaths(paths);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setActiveTab("documents");
      setImportMessage({ text: `Could not open the file dialog: ${msg}`, type: "error" });
    }
  };

  // Reconciliation Handlers
  const handleRunReconciliation = async () => {
    setIsReconciling(true);
    setReviewMessage("Matching documents to bank rows and checking for anomalies...");
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
        `Restored archive successfully! Workspace: ${manifest.workspaceName}, Blobs: ${manifest.blobCount}. ` +
          "It now appears on the unlock screen; unlock it with its own workspace passphrase."
      );
      setWorkspaces(await SaarApi.listWorkspaces());
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
    // Each suggestion maps to an operation that SAAR computes from the records and marks verified.
    if (transactions.length > 0) {
      suggestions.push("What are my total expenses?");
      suggestions.push("What are my largest expenses?");
    }
    if (financialSummary) {
      suggestions.push(`What is my net cash flow in ${workspace?.baseCurrency ?? "USD"}?`);
    }
    if (anomalies.length > 0) {
      suggestions.push("Are there any duplicate payments?");
    }
    if (reconciliationLinks.length > 0 || transactions.length > 0) {
      suggestions.push("Which payments have no matching invoice?");
    }
    if (documents.length > 0) {
      suggestions.push("How many documents are in this workspace?");
    }
    return suggestions.slice(0, 4);
  }, [documents, anomalies, reconciliationLinks, transactions, financialSummary, workspace]);

  // Header Title
  const activeTabTitle = useMemo(() => {
    if (activeTab === "chat") {
      return activeThread?.title || "New conversation";
    }
    const titles: Record<WorkspaceTab, string> = {
      chat: "Conversation",
      documents: "Documents",
      review: "Reconciliation",
      reports: "Reports",
      security: "Security",
    };
    return titles[activeTab];
  }, [activeTab, activeThread]);

  const hasInspectorContent = Boolean(selectedDocDetails || selectedTx || selectedLink);

  return (
    <div className={`saar-app-root ${sidebarCollapsed ? "sidebar-collapsed" : ""}`} data-theme={theme}>
      {isDraggingFiles && (
        <div className="drop-overlay" aria-hidden="true">
          <div className="drop-overlay-card">
            <strong>Drop to import</strong>
            <span>PDF, PNG, JPG or CSV</span>
          </div>
        </div>
      )}
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
            onCollapse={toggleSidebar}
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
              sidebarCollapsed={sidebarCollapsed}
              onToggleSidebar={toggleSidebar}
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
              modelLabel={status?.llmModelLabel ?? "Local model"}
            />

            {/* Split Viewport: Central Stage + Contextual Evidence Inspector Drawer */}
            <div className="claude-split-body">
              <main className="claude-central-stage">
                <ErrorBoundary key={activeTab} label={activeTabTitle}>
                {activeTab === "chat" && (
                  <ChatWorkspace
                    messages={activeThread?.messages || []}
                    onSendMessage={handleSendMessage}
                    modelLabel={status?.llmModelLabel ?? "the local model"}
                    isLoading={isChatLoading}
                    errorMessage={chatError}
                    suggestions={chatSuggestions}
                    onSelectCitation={handleSelectCitation}
                    onNavigateToTab={(tab) => {
                      if (tab === "documents") setActiveTab("documents");
                      else if (tab === "review") setActiveTab("review");
                      else if (tab === "reports") setActiveTab("reports");
                    }}
                    onPickFiles={handlePickFiles}
                  />
                )}

                {activeTab === "documents" && (
                  <DocumentsView
                    documents={documents}
                    selectedDocumentId={selectedDocDetails?.document.id || null}
                    onSelectDocument={loadDocumentDetails}
                    onIngestFile={handleIngestFile}
                    onPickFiles={handlePickFiles}
                    isDraggingFiles={isDraggingFiles}
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
                    invoiceSummary={invoiceSummary}
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
                </ErrorBoundary>
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
