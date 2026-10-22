/**
 * OpenDeck Typed Domain Entities & IPC Schema
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

export interface FinancialSummary {
  totalIncomeMinor: number;
  totalExpenseMinor: number;
  netSavingsMinor: number;
  currency: string;
  monthlyBreakdown: MonthAggregate[];
  momExpenseChangePercent?: number;
}

export interface ReconciliationResult {
  links: ProposedLink[];
  anomalies: FinancialAnomaly[];
}

