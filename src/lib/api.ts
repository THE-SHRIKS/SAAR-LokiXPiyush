/**
 * Typed IPC bridge for SAAR.
 *
 * SAAR runs only inside the Tauri desktop shell. Every method maps to a command
 * registered in `src-tauri/src/lib.rs`, so the UI never renders data that did not
 * come from the encrypted local workspace.
 */

import { invoke } from "@tauri-apps/api/core";
import {
  SystemStatus,
  WorkspaceInfo,
  WorkspaceSummary,
  DocumentRecord,
  TransactionRecord,
  DocumentDetails,
  IngestResult,
  ProposedLink,
  FinancialAnomaly,
  FinancialSummary,
  ReconciliationResult,
  GroundedChatResponse,
  CategoryProposal,
  SearchResult,
  DocumentPreview,
  AuditEventRecord,
  ExportResult,
  BackupManifest,
  ChatThreadRecord,
  ChatMessageRecord,
} from "./types";

const isTauri =
  typeof window !== "undefined" &&
  Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri) {
    return Promise.reject(
      new Error("SAAR must run inside the desktop application. Start it with the Tauri shell."),
    );
  }
  return invoke<T>(command, args);
}

export class SaarApi {
  static getSystemStatus(): Promise<SystemStatus> {
    return call<SystemStatus>("get_system_status");
  }

  static listWorkspaces(): Promise<WorkspaceSummary[]> {
    return call<WorkspaceSummary[]>("list_workspaces");
  }

  static createWorkspace(
    workspaceName: string,
    passphrase: string,
    baseCurrency: string,
  ): Promise<WorkspaceInfo> {
    return call<WorkspaceInfo>("create_workspace", {
      displayName: workspaceName,
      passphrase,
      baseCurrency,
    });
  }

  static unlockWorkspace(workspaceId: string, passphrase: string): Promise<WorkspaceInfo> {
    return call<WorkspaceInfo>("unlock_workspace", { workspaceId, passphrase });
  }

  static lockWorkspace(): Promise<void> {
    return call<void>("lock_workspace");
  }

  static listDocuments(): Promise<DocumentRecord[]> {
    return call<DocumentRecord[]>("list_documents");
  }

  static listTransactions(): Promise<TransactionRecord[]> {
    return call<TransactionRecord[]>("list_transactions");
  }

  static ingestFile(filePath: string): Promise<IngestResult> {
    return call<IngestResult>("ingest_file", { filePath });
  }

  static getDocumentDetails(documentId: string): Promise<DocumentDetails> {
    return call<DocumentDetails>("get_document_details", { documentId });
  }

  static getDocumentPreview(documentId: string): Promise<DocumentPreview> {
    return call<DocumentPreview>("get_document_preview", { documentId });
  }

  static runReconciliation(): Promise<ReconciliationResult> {
    return call<ReconciliationResult>("run_reconciliation");
  }

  static listReconciliationLinks(): Promise<ProposedLink[]> {
    return call<ProposedLink[]>("list_reconciliation_links");
  }

  static listAnomalies(): Promise<FinancialAnomaly[]> {
    return call<FinancialAnomaly[]>("list_anomalies");
  }

  static approveReconciliationLink(linkId: string): Promise<void> {
    return call<void>("approve_reconciliation_link", { linkId });
  }

  static rejectReconciliationLink(linkId: string): Promise<void> {
    return call<void>("reject_reconciliation_link", { linkId });
  }

  static correctTransaction(transactionId: string, newDescription: string): Promise<void> {
    return call<void>("correct_transaction", { transactionId, newDescription });
  }

  static listAuditLog(): Promise<AuditEventRecord[]> {
    return call<AuditEventRecord[]>("list_audit_log");
  }

  static getFinancialSummary(currency: string): Promise<FinancialSummary> {
    return call<FinancialSummary>("get_financial_summary", { currency });
  }

  static listChatThreads(): Promise<ChatThreadRecord[]> {
    return call<ChatThreadRecord[]>("list_chat_threads");
  }

  static createChatThread(title?: string): Promise<ChatThreadRecord> {
    return call<ChatThreadRecord>("create_chat_thread", { title: title ?? null });
  }

  static listChatMessages(threadId: string): Promise<ChatMessageRecord[]> {
    return call<ChatMessageRecord[]>("list_chat_messages", { threadId });
  }

  static renameChatThread(threadId: string, title: string): Promise<void> {
    return call<void>("rename_chat_thread", { threadId, title });
  }

  static setChatThreadPinned(threadId: string, pinned: boolean): Promise<void> {
    return call<void>("set_chat_thread_pinned", { threadId, pinned });
  }

  static deleteChatThread(threadId: string): Promise<void> {
    return call<void>("delete_chat_thread", { threadId });
  }

  static chatGrounded(threadId: string, query: string): Promise<GroundedChatResponse> {
    return call<GroundedChatResponse>("chat_grounded", { threadId, query });
  }

  static classifyTransaction(transactionId: string): Promise<CategoryProposal> {
    return call<CategoryProposal>("classify_transaction", { transactionId });
  }

  static approveCategory(proposalId: string): Promise<void> {
    return call<void>("approve_category", { proposalId });
  }

  static searchWorkspace(query: string): Promise<SearchResult[]> {
    return call<SearchResult[]>("search_workspace", { query });
  }

  static exportReconciliationReport(format: "pdf" | "csv"): Promise<ExportResult> {
    return call<ExportResult>("export_reconciliation_report", {
      format,
      destinationPath: null,
    });
  }

  static listReportExports(): Promise<ExportResult[]> {
    return call<ExportResult[]>("list_report_exports");
  }

  static createEncryptedBackup(passphrase: string, destinationPath?: string): Promise<BackupManifest> {
    return call<BackupManifest>("create_encrypted_backup", {
      passphrase,
      destinationPath: destinationPath || null,
    });
  }

  static restoreEncryptedBackup(
    archivePath: string,
    passphrase: string,
    targetDataDir?: string,
  ): Promise<BackupManifest> {
    return call<BackupManifest>("restore_encrypted_backup", {
      archivePath,
      passphrase,
      targetDataDir: targetDataDir || null,
    });
  }
}
