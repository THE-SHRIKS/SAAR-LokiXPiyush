import React, { useState } from "react";
import { X, Check, CheckCircle2, Edit2 } from "lucide-react";
import {
  DocumentDetails,
  DocumentPreview,
  NormalizedField,
  TransactionRecord,
  ProposedLink,
  formatMinorCurrency,
} from "../lib/types";

interface EvidenceInspectorProps {
  isOpen: boolean;
  onClose: () => void;
  documentDetails: DocumentDetails | null;
  documentPreview: DocumentPreview | null;
  selectedTransaction: TransactionRecord | null;
  selectedLink: ProposedLink | null;
  onApproveLink?: (linkId: string) => Promise<void>;
  onRejectLink?: (linkId: string) => Promise<void>;
  onStartCorrection?: (tx: TransactionRecord) => void;
}

export const EvidenceInspector: React.FC<EvidenceInspectorProps> = ({
  isOpen,
  onClose,
  documentDetails,
  documentPreview,
  selectedTransaction,
  selectedLink,
  onApproveLink,
  onRejectLink,
  onStartCorrection,
}) => {
  const [activeField, setActiveField] = useState<NormalizedField | null>(null);

  if (!isOpen) return null;

  const doc = documentDetails?.document;

  return (
    <aside className="evidence-inspector-drawer">
      {/* Header */}
      <div className="inspector-head">
        <div className="inspector-title-group">
          <span className="inspector-kicker">Contextual Evidence Inspector</span>
          <h3 className="inspector-title">
            {doc
              ? doc.fileName
              : selectedTransaction
                ? selectedTransaction.description
                : "Evidence Details"}
          </h3>
        </div>
        <button className="inspector-close-btn" onClick={onClose} aria-label="Close Inspector">
          <X size={16} />
        </button>
      </div>

      <div className="inspector-scroll-body">
        {/* Document Visual Preview Surface */}
        {documentPreview && (
          <div className="inspector-section">
            <div className="section-header-row">
              <span className="section-label">Source Document Preview</span>
              <span className="section-tag">{documentPreview.mimeType}</span>
            </div>

            <div className="preview-canvas-container">
              {documentPreview.mimeType.startsWith("image/") && (
                <div className="preview-image-box">
                  <img
                    src={`data:${documentPreview.mimeType};base64,${documentPreview.dataBase64}`}
                    alt="Document Scan"
                    className="preview-img"
                  />
                  {/* Bounding box highlight overlays */}
                  {activeField &&
                    (() => {
                      try {
                        const box = JSON.parse(activeField.boundingBoxJson);
                        if (Array.isArray(box) && box.length >= 4 && typeof box[0] === "number") {
                          return (
                            <div
                              className="bbox-highlight-rect"
                              style={{
                                left: `${box[0]}px`,
                                top: `${box[1]}px`,
                                width: `${Math.max(box[2] - box[0], 24)}px`,
                                height: `${Math.max(box[3] - box[1], 18)}px`,
                              }}
                            >
                              <span className="bbox-label">{activeField.fieldKey}</span>
                            </div>
                          );
                        }
                      } catch {
                        return null;
                      }
                      return null;
                    })()}
                </div>
              )}

              {documentPreview.mimeType === "application/pdf" && (
                <div className="preview-pdf-box">
                  <object
                    data={`data:application/pdf;base64,${documentPreview.dataBase64}`}
                    type="application/pdf"
                    className="preview-pdf-object"
                  >
                    <p className="pdf-fallback-text">PDF preview loaded under encrypted storage.</p>
                  </object>
                </div>
              )}

              {documentPreview.mimeType === "text/csv" && documentPreview.textPreview && (
                <pre className="preview-csv-text">{documentPreview.textPreview}</pre>
              )}
            </div>
          </div>
        )}

        {/* Normalized Extracted Fields */}
        {documentDetails && documentDetails.fields.length > 0 && (
          <div className="inspector-section">
            <div className="section-header-row">
              <span className="section-label">Extracted Normalized Fields</span>
              <span className="section-tag">{documentDetails.fields.length} Detected</span>
            </div>

            <div className="fields-grid">
              {documentDetails.fields.map((field, idx) => (
                <div
                  key={idx}
                  className={`field-card ${activeField?.fieldKey === field.fieldKey ? "selected" : ""}`}
                  onClick={() => setActiveField(field)}
                  onMouseEnter={() => setActiveField(field)}
                >
                  <div className="field-card-top">
                    <span className="field-name">{field.fieldKey.replace(/_/g, " ")}</span>
                    <span className="field-confidence">
                      {(field.confidence * 100).toFixed(0)}% conf
                    </span>
                  </div>
                  <div className="field-value-text">{field.normalizedValue}</div>
                  <div className="field-source-meta">
                    <span>Page {field.pageNum}</span>
                    <span className="field-bbox-text">Box: {field.boundingBoxJson}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Invoice Line Items */}
        {documentDetails && documentDetails.lineItems && documentDetails.lineItems.length > 0 && (
          <div className="inspector-section">
            <div className="section-header-row">
              <span className="section-label">Parsed Line Items</span>
              <span className="section-tag">{documentDetails.lineItems.length} Items</span>
            </div>

            <div className="line-items-table-wrap">
              <table className="line-items-table">
                <thead>
                  <tr>
                    <th>Description</th>
                    <th className="num-col">Minor Units</th>
                  </tr>
                </thead>
                <tbody>
                  {documentDetails.lineItems.map((item, idx) => (
                    <tr key={idx}>
                      <td>{item.description}</td>
                      <td className="num-col font-mono">
                        {formatMinorCurrency(item.amountMinor, "USD")}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Selected / Linked Transaction */}
        {selectedTransaction && (
          <div className="inspector-section">
            <div className="section-header-row">
              <span className="section-label">Ledger Transaction</span>
              <span className={`status-pill ${selectedTransaction.reviewState.toLowerCase()}`}>
                {selectedTransaction.reviewState}
              </span>
            </div>

            <div className="tx-details-card">
              <div className="tx-card-row">
                <span className="tx-meta-label">Description</span>
                <span className="tx-meta-val">{selectedTransaction.description}</span>
              </div>
              <div className="tx-card-row">
                <span className="tx-meta-label">Posted Date</span>
                <span className="tx-meta-val font-mono">{selectedTransaction.postedDate}</span>
              </div>
              <div className="tx-card-row">
                <span className="tx-meta-label">Reference</span>
                <span className="tx-meta-val font-mono">
                  {selectedTransaction.externalRef || "—"}
                </span>
              </div>
              <div className="tx-card-row">
                <span className="tx-meta-label">Amount</span>
                <span
                  className={`tx-meta-val font-mono ${
                    selectedTransaction.amountMinor < 0 ? "amount-debit" : "amount-credit"
                  }`}
                >
                  {formatMinorCurrency(
                    selectedTransaction.amountMinor,
                    selectedTransaction.currency
                  )}
                </span>
              </div>

              {onStartCorrection && (
                <button
                  className="btn-edit-tx"
                  onClick={() => onStartCorrection(selectedTransaction)}
                >
                  <Edit2 size={12} />
                  <span>Correct Description in Audit Log</span>
                </button>
              )}
            </div>
          </div>
        )}

        {/* Proposed Match Decision Bar */}
        {selectedLink && (
          <div className="inspector-section decision-section">
            <div className="section-header-row">
              <span className="section-label">Reconciliation Link Proposal</span>
              <span className="badge-match">{selectedLink.matchType}</span>
            </div>

            <p className="link-explanation-text">{selectedLink.explanation}</p>

            <div className="link-decision-actions">
              {selectedLink.status === "PROPOSED" ? (
                <>
                  {onApproveLink && (
                    <button
                      className="btn-approve-match"
                      onClick={() => onApproveLink(selectedLink.id)}
                    >
                      <Check size={14} />
                      <span>Approve Match Link</span>
                    </button>
                  )}
                  {onRejectLink && (
                    <button
                      className="btn-reject-match"
                      onClick={() => onRejectLink(selectedLink.id)}
                    >
                      <X size={14} />
                      <span>Reject</span>
                    </button>
                  )}
                </>
              ) : (
                <div className="status-decided-banner">
                  <CheckCircle2 size={14} className="icon-green" />
                  <span>Decision Recorded: {selectedLink.status}</span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Document Technical Integrity Footnote */}
        {doc && (
          <div className="inspector-meta-footer">
            <div className="meta-row">
              <span className="meta-key">SHA-256</span>
              <span className="meta-val font-mono">{doc.sha256}</span>
            </div>
            <div className="meta-row">
              <span className="meta-key">File Size</span>
              <span className="meta-val font-mono">{(doc.fileSizeBytes / 1024).toFixed(1)} KB</span>
            </div>
            <div className="meta-row">
              <span className="meta-key">Storage</span>
              <span className="meta-val">AES-256-GCM Authenticated Blob</span>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
