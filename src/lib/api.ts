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
}
