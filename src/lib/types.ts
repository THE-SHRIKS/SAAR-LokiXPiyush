/**
 * SAAR Typed Domain Entities & IPC Schema
 * Strictly enforces integer minor-unit amounts and offline auditability.
 */

export type CurrencyCode = "USD" | "EUR" | "GBP" | "INR" | "CAD" | "AUD" | "JPY";

export type IngestionStatus =
  | "SELECTED"
  | "IMPORTING"
  | "PROCESSING"
  | "STORED"
  | "READY"
  | "EXTRACTING"
  | "EXTRACTED"
  | "NEEDS_REVIEW"
  | "VERIFIED"
  | "FAILED"
  | "CANCELLED";

export type DocumentType = "PDF" | "PNG" | "JPEG" | "CSV" | "INVOICE" | "RECEIPT" | "UNKNOWN";

export type ReviewState = "UNREVIEWED" | "APPROVED" | "REJECTED" | "CORRECTED";

export interface WorkspaceInfo {
  id: string;
  displayName: string;
  baseCurrency: CurrencyCode;
  schemaVersion: number;
  createdAt: string;
  isUnlocked: boolean;
}

export interface WorkspaceSummary {
  id: string;
  displayName: string;
  baseCurrency: string;
  createdAt: string;
}

export interface ChatThreadRecord {
  id: string;
  title: string;
  pinned: boolean;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
}

export interface ChatMessageRecord {
  id: string;
  threadId: string;
  sender: "user" | "assistant";
  text: string;
  response: GroundedChatResponse | null;
  createdAt: string;
}

export interface DocumentRecord {
  id: string;
  workspaceId: string;
  fileName: string;
  fileSizeBytes: number;
  sha256: string;
  docType: DocumentType;
  pageCount: number;
  status: IngestionStatus;
  importedAt: string;
}

export interface DocumentLineItem {
  description: string;
  amountMinor: number; // Exact minor unit (cents)
}

export interface NormalizedField {
  fieldKey: string;
  originalText: string;
  normalizedValue: string;
  confidence: number;
  pageNum: number;
  boundingBoxJson: string;
}

export interface DocumentDetails {
  document: DocumentRecord;
  fields: NormalizedField[];
  lineItems: DocumentLineItem[];
  invoiceTotalMinor?: number;
  currency?: string | null;
}

export interface DocumentPreview {
  documentId: string;
  mimeType: string;
  dataBase64: string;
  textPreview?: string;
}

export interface AuditEventRecord {
  id: string;
  entityType: string;
  entityId: string;
  action: string;
  payloadJson: string;
  createdAt: string;
}

export interface IngestResultSuccess {
  type: "success";
  documentId: string;
  fileName: string;
  docType: string;
  pageCount: number;
  recordsExtracted: number;
}

export interface IngestResultDuplicate {
  type: "duplicate";
  existingDocumentId: string;
  fileName: string;
  sha256: string;
}

export type IngestResult = IngestResultSuccess | IngestResultDuplicate;

export interface TransactionRecord {
  id: string;
  workspaceId: string;
  sourceDocId?: string;
  postedDate: string;
  description: string;
  amountMinor: number; // Positive = credit/income, Negative = debit/expense
  currency: CurrencyCode;
  externalRef?: string;
  matchedInvoiceId?: string;
  reviewState: ReviewState;
}

export interface SystemStatus {
  appVersion: string;
  platform: string;
  offline: boolean;
  ocrReady: boolean;
  llmReady: boolean;
  llmModelId: string;
  activeWorkspaceId: string | null;
  databaseEncrypted: boolean;
}

/**
 * Format integer minor units (cents) into human-readable currency.
 * NEVER uses binary float division without exact decimal string formatting.
 */
export function formatMinorCurrency(minorUnits: number, currency: CurrencyCode): string {
  const isNegative = minorUnits < 0;
  const absUnits = Math.abs(minorUnits);
  const major = Math.floor(absUnits / 100);
  const minor = absUnits % 100;
  const minorPadded = minor.toString().padStart(2, "0");
  const sign = isNegative ? "-" : "";

  const symbols: Record<CurrencyCode, string> = {
    USD: "$",
    EUR: "€",
    GBP: "£",
    INR: "₹",
    CAD: "CA$",
    AUD: "A$",
    JPY: "¥",
  };

  const symbol = symbols[currency] || `${currency} `;
  if (currency === "JPY") {
    // JPY does not use fractional units
    return `${sign}${symbol}${absUnits.toLocaleString("en-US")}`;
  }

  return `${sign}${symbol}${major.toLocaleString("en-US")}.${minorPadded}`;
}

export interface ProposedLink {
  id: string;
  invoiceId: string;
  transactionId: string;
  matchedAmountMinor: number;
  confidenceScore: number;
  matchType: string;
  explanation: string;
  status: "PROPOSED" | "APPROVED" | "REJECTED";
}

export interface FinancialAnomaly {
  id: string;
  entityType: "INVOICE" | "TRANSACTION";
  entityId: string;
  anomalyType: string;
  description: string;
  severity: "LOW" | "MEDIUM" | "HIGH";
}

export interface MonthAggregate {
  monthKey: string;
  totalIncomeMinor: number;
  totalExpenseMinor: number;
  netCashFlowMinor: number;
  currency: string;
  transactionCount: number;
}

export interface CurrencyExclusion {
  currency: string;
  transactionCount: number;
  grossMinor: number;
}

export interface FinancialSummary {
  totalIncomeMinor: number;
  totalExpenseMinor: number;
  netSavingsMinor: number;
  currency: string;
  monthlyBreakdown: MonthAggregate[];
  /** null when fewer than two months of data exist. */
  momExpenseChangePercent?: number | null;
  excludedCurrencies: CurrencyExclusion[];
}

export interface ReconciliationResult {
  links: ProposedLink[];
  anomalies: FinancialAnomaly[];
}

export type QueryIntent =
  | "FINANCIAL_SUMMARY"
  | "DOCUMENT_QUERY"
  | "RECONCILIATION_STATUS"
  | "TRANSACTION_SEARCH"
  | "GENERAL_FINANCIAL";

export interface GroundedChatResponse {
  answer: string;
  intent: QueryIntent;
  citations: string[];
  grounded: boolean;
  verificationNote: string;
}

export interface ChatMessage {
  id: string;
  sender: "user" | "assistant";
  text: string;
  response?: GroundedChatResponse;
  timestamp: string;
}

export interface ChatThread {
  id: string;
  title: string;
  pinned?: boolean;
  createdAt: string;
  updatedAt: string;
  messages: ChatMessage[];
}

export interface CategoryProposal {
  id: string;
  transactionId: string;
  suggestedCategory: string;
  confidenceScore: number;
  rationale: string;
  status: "PROPOSED" | "APPROVED" | "REJECTED";
  createdAt: string;
}

export interface SearchResult {
  entityType: "DOCUMENT" | "TRANSACTION" | "INVOICE";
  entityId: string;
  title: string;
  snippet: string;
  score: number;
}

export interface ExportResult {
  exportId: string;
  filePath: string;
  exportFormat: string;
  recordCount: number;
  recordSnapshotHash: string;
  fileSizeBytes: number;
  createdAt: string;
}

export interface BackupManifest {
  version: number;
  appVersion: string;
  createdAt: string;
  workspaceId: string;
  workspaceName: string;
  dbSha256: string;
  blobCount: number;
  totalRawBytes: number;
}
