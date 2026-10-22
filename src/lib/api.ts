/**
 * Typed Tauri Command Bridge for OpenDeck
 * Bridges React UI to the trusted Rust core via Tauri invoke().
 */

import { invoke } from "@tauri-apps/api/core";
import {
  SystemStatus,
  WorkspaceInfo,
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
} from "./types";

export class OpenDeckApi {
  /**
   * Retrieves host runtime state, offline verification, and engine statuses.
   */
  static async getSystemStatus(): Promise<SystemStatus> {
    try {
      return await invoke<SystemStatus>("get_system_status");
    } catch (err) {
      console.warn("Tauri invoke unavailable, running in preview/test harness", err);
      return {
        appVersion: "0.1.0",
        platform: "linux",
        offline: true,
        ocrReady: true,
        llmReady: true,
        llmModelId: "qwen-2.5-3b-instruct-q3_k_m",
        activeWorkspaceId: null,
        databaseEncrypted: true,
      };
    }
  }

  /**
   * Derives workspace encryption key and unlocks database.
   */
  static async unlockWorkspace(workspaceId: string, passphrase: string): Promise<WorkspaceInfo> {
    return await invoke<WorkspaceInfo>("unlock_workspace", { workspaceId, passphrase });
  }

  /**
   * Closes database connection, clears memory keys, and locks workspace.
   */
  static async lockWorkspace(): Promise<void> {
    return await invoke<void>("lock_workspace");
  }

  /**
   * Creates a new encrypted workspace initialized with Argon2id salt.
   */
  static async createWorkspace(
    displayName: string,
    passphrase: string,
    baseCurrency: string
  ): Promise<WorkspaceInfo> {
    return await invoke<WorkspaceInfo>("create_workspace", {
      displayName,
      passphrase,
      baseCurrency,
    });
  }

  /**
   * Fetches ingested documents for the active workspace.
   */
  static async listDocuments(): Promise<DocumentRecord[]> {
    return await invoke<DocumentRecord[]>("list_documents");
  }

  /**
   * Fetches reconciled/imported transactions for the active workspace.
   */
  static async listTransactions(): Promise<TransactionRecord[]> {
    return await invoke<TransactionRecord[]>("list_transactions");
  }

  /**
   * Ingests a local file (PDF, PNG, JPG, CSV) into the encrypted workspace.
   */
  static async ingestFile(filePath: string): Promise<IngestResult> {
    return await invoke<IngestResult>("ingest_file", { filePath });
  }

  /**
   * Fetches extracted document data with bounding boxes and line items.
   */
  static async getDocumentDetails(documentId: string): Promise<DocumentDetails> {
    return await invoke<DocumentDetails>("get_document_details", { documentId });
  }

  /**
   * Fetches decrypted document preview bytes (base64 data URI).
   */
  static async getDocumentPreview(documentId: string): Promise<DocumentPreview> {
    return await invoke<DocumentPreview>("get_document_preview", { documentId });
  }

  /**
   * Triggers deterministic reconciliation engine over active workspace records.
   */
  static async runReconciliation(): Promise<ReconciliationResult> {
    return await invoke<ReconciliationResult>("run_reconciliation");
  }

  /**
   * Retrieves proposed and reviewed reconciliation links.
   */
  static async listReconciliationLinks(): Promise<ProposedLink[]> {
    return await invoke<ProposedLink[]>("list_reconciliation_links");
  }

  /**
   * Retrieves active financial anomalies and duplicate alerts.
   */
  static async listAnomalies(): Promise<FinancialAnomaly[]> {
    return await invoke<FinancialAnomaly[]>("list_anomalies");
  }

  /**
   * Approves a proposed match link between invoice and transaction.
   */
  static async approveReconciliationLink(linkId: string): Promise<void> {
    return await invoke<void>("approve_reconciliation_link", { linkId });
  }

  /**
   * Rejects a proposed match link.
   */
  static async rejectReconciliationLink(
    linkId: string,
    reason = "Rejected by user"
  ): Promise<void> {
    return await invoke<void>("reject_reconciliation_link", { linkId, reason });
  }

  /**
   * User manual correction of transaction description with full audit logging.
   */
  static async correctTransaction(transactionId: string, newDescription: string): Promise<void> {
    return await invoke<void>("correct_transaction", { transactionId, newDescription });
  }

  /**
   * Retrieves immutable audit log events.
   */
  static async listAuditLog(): Promise<AuditEventRecord[]> {
    return await invoke<AuditEventRecord[]>("list_audit_log");
  }

  /**
   * Computes deterministic financial summary and monthly breakdown in integer minor units.
   */
  static async getFinancialSummary(currency = "USD"): Promise<FinancialSummary> {
    return await invoke<FinancialSummary>("get_financial_summary", { currency });
  }

  /**
   * Local grounded Q&A with Fact Packet citations and hallucination verification.
   */
  static async chatGrounded(query: string): Promise<GroundedChatResponse> {
    return await invoke<GroundedChatResponse>("chat_grounded", { query });
  }

  /**
   * Local GBNF-constrained expense & revenue category proposal.
   */
  static async classifyTransaction(transactionId: string): Promise<CategoryProposal> {
    return await invoke<CategoryProposal>("classify_transaction", { transactionId });
  }

  /**
   * Approves a transaction category classification with immutable audit logging.
   */
  static async approveCategory(proposalId: string): Promise<void> {
    return await invoke<void>("approve_category", { proposalId });
  }

  /**
   * Fast full-text search across all documents and transactions via SQLite FTS5.
   */
  static async searchWorkspace(query: string): Promise<SearchResult[]> {
    return await invoke<SearchResult[]>("search_workspace", { query });
  }

  /**
   * Generates and exports source-traceable PDF or CSV reconciliation report.
   */
  static async exportReconciliationReport(
    format: "pdf" | "csv",
    destinationPath?: string
  ): Promise<ExportResult> {
    return await invoke<ExportResult>("export_reconciliation_report", {
      format,
      destinationPath: destinationPath || null,
    });
  }

  /**
   * Lists historical report exports with snapshot hashes and file paths.
   */
  static async listReportExports(): Promise<ExportResult[]> {
    return await invoke<ExportResult[]>("list_report_exports");
  }

  /**
   * Creates an encrypted authenticated .opendeck backup archive of database and blobs.
   */
  static async createEncryptedBackup(
    passphrase: string,
    destinationPath?: string
  ): Promise<BackupManifest> {
    return await invoke<BackupManifest>("create_encrypted_backup", {
      passphrase,
      destinationPath: destinationPath || null,
    });
  }

  /**
   * Restores an encrypted authenticated .opendeck backup archive into target directories.
   */
  static async restoreEncryptedBackup(
    archivePath: string,
    passphrase: string,
    targetDataDir?: string
  ): Promise<BackupManifest> {
    return await invoke<BackupManifest>("restore_encrypted_backup", {
      archivePath,
      passphrase,
      targetDataDir: targetDataDir || null,
    });
  }
}
