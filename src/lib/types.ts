/**
 * OpenDeck Typed Domain Entities & IPC Schema
 * Strictly enforces integer minor-unit amounts and offline auditability.
 */

export type CurrencyCode = "USD" | "EUR" | "GBP" | "INR" | "CAD" | "AUD" | "JPY";

export type IngestionStatus =
  | "SELECTED"
  | "IMPORTING"
  | "STORED"
  | "EXTRACTING"
  | "EXTRACTED"
  | "NEEDS_REVIEW"
  | "VERIFIED"
  | "FAILED"
  | "CANCELLED";

export type DocumentType = "INVOICE" | "RECEIPT" | "BANK_STATEMENT" | "TAX_FORM" | "UNKNOWN";

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
  id: string;
  description: string;
  amountMinor: number; // Exact minor unit (cents)
  quantity?: number;
  unitPriceMinor?: number;
}

export interface ExtractedInvoice {
  id: string;
  documentId: string;
  vendorName: string;
  invoiceNumber?: string;
  invoiceDate?: string;
  dueDate?: string;
  currency: CurrencyCode;
  totalMinor: number; // Cents
  taxMinor: number; // Cents
  lineItems: DocumentLineItem[];
  confidence: number;
  reviewState: ReviewState;
}

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
 * NEVER uses float division without exact decimal string formatting.
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
