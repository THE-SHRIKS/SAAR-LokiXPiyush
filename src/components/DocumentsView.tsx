import React, { useState } from "react";
import {
  UploadCloud,
  FileText,
  Clock,
  CheckCircle2,
  AlertCircle,
  ChevronRight,
  Filter,
} from "lucide-react";
import { DocumentRecord } from "../lib/types";

interface DocumentsViewProps {
  documents: DocumentRecord[];
  selectedDocumentId: string | null;
  onSelectDocument: (docId: string) => void;
  onIngestFile: (filePath: string) => Promise<void>;
  onPickFiles: () => void;
  isDraggingFiles: boolean;
  isImporting: boolean;
  importMessage: { text: string; type: "success" | "error" | "info" } | null;
}

export const DocumentsView: React.FC<DocumentsViewProps> = ({
  documents,
  selectedDocumentId,
  onSelectDocument,
  onIngestFile,
  onPickFiles,
  isDraggingFiles,
  isImporting,
  importMessage,
}) => {
  const [inputPath, setInputPath] = useState("");
  const [filterType, setFilterType] = useState<"ALL" | "PDF" | "PNG" | "CSV">("ALL");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputPath.trim() || isImporting) return;
    onIngestFile(inputPath.trim());
    setInputPath("");
  };

  const filteredDocs = documents.filter((d) => {
    if (filterType === "ALL") return true;
    return (
      d.docType.toUpperCase() === filterType || (filterType === "PDF" && d.docType === "INVOICE")
    );
  });

  return (
    <div className="documents-workspace">
      {/* Ingestion Command Bar */}
      <div className="ingest-command-card">
        <div className={`drop-zone ${isDraggingFiles ? "active" : ""}`}>
          <UploadCloud size={24} className="drop-zone-icon" />
          <div className="drop-zone-text">
            <strong>Drop invoices, receipts or bank statements here</strong>
            <span>PDF, PNG, JPG or CSV. Up to 50 MB and 50 pages each. You can drop several at once.</span>
          </div>
          <button type="button" className="btn-primary-medium" onClick={onPickFiles} disabled={isImporting}>
            {isImporting ? "Importing..." : "Browse files"}
          </button>
        </div>

        <details className="path-import">
          <summary>Import by file path</summary>
          <form onSubmit={handleSubmit} className="ingest-input-row">
            <input
              type="text"
              className="input-text flex-grow"
              placeholder="C:\path\to\file.pdf"
              value={inputPath}
              onChange={(e) => setInputPath(e.target.value)}
              disabled={isImporting}
            />
            <button type="submit" className="btn-secondary-medium" disabled={isImporting || !inputPath.trim()}>
              Ingest Document
            </button>
          </form>
        </details>

        {importMessage && (
          <div className={`ingest-status-banner ${importMessage.type}`}>
            {importMessage.type === "success" && <CheckCircle2 size={14} className="icon-green" />}
            {importMessage.type === "error" && <AlertCircle size={14} className="icon-danger" />}
            {importMessage.type === "info" && <Clock size={14} className="icon-cream" />}
            <span>{importMessage.text}</span>
          </div>
        )}
      </div>

      {/* Document Archive Repository Table */}
      <div className="documents-table-card">
        <div className="table-card-toolbar">
          <div className="toolbar-left">
            <h4>Document Repository ({documents.length})</h4>
            <span className="toolbar-sub">Stored encrypted on this computer</span>
          </div>

          <div className="filter-button-group">
            <Filter size={12} className="icon-muted" />
            <button
              className={`filter-btn ${filterType === "ALL" ? "active" : ""}`}
              onClick={() => setFilterType("ALL")}
            >
              All
            </button>
            <button
              className={`filter-btn ${filterType === "PDF" ? "active" : ""}`}
              onClick={() => setFilterType("PDF")}
            >
              PDFs
            </button>
            <button
              className={`filter-btn ${filterType === "PNG" ? "active" : ""}`}
              onClick={() => setFilterType("PNG")}
            >
              Scans
            </button>
            <button
              className={`filter-btn ${filterType === "CSV" ? "active" : ""}`}
              onClick={() => setFilterType("CSV")}
            >
              CSVs
            </button>
          </div>
        </div>

        {filteredDocs.length === 0 ? (
          <div className="empty-table-state">
            <FileText size={32} className="icon-muted" />
            <p>{documents.length === 0 ? "No documents yet" : "No documents match this filter"}</p>
            <span>
              {documents.length === 0
                ? "Drop files above or choose Browse files to import your first invoice, receipt or bank statement."
                : "Choose All to see every document."}
            </span>
          </div>
        ) : (
          <div className="table-scroll-wrap">
            <table className="app-data-table">
              <thead>
                <tr>
                  <th>File Name</th>
                  <th>Type</th>
                  <th>Size</th>
                  <th>Pages</th>
                  <th>SHA-256 Fingerprint</th>
                  <th>Status</th>
                  <th>Imported At</th>
                  <th className="action-col">Inspect</th>
                </tr>
              </thead>
              <tbody>
                {filteredDocs.map((doc) => (
                  <tr
                    key={doc.id}
                    className={`table-row ${selectedDocumentId === doc.id ? "selected-row" : ""}`}
                    onClick={() => onSelectDocument(doc.id)}
                  >
                    <td className="file-name-cell">
                      <span className="file-name-text">{doc.fileName}</span>
                    </td>
                    <td>
                      <span className={`doc-type-badge type-${doc.docType.toLowerCase()}`}>
                        {doc.docType}
                      </span>
                    </td>
                    <td className="font-mono">{(doc.fileSizeBytes / 1024).toFixed(1)} KB</td>
                    <td className="font-mono">{doc.pageCount}</td>
                    <td className="font-mono text-muted text-xs">
                      {doc.sha256.substring(0, 16)}...
                    </td>
                    <td>
                      <span className="status-pill ready">{doc.status}</span>
                    </td>
                    <td className="font-mono text-muted text-xs">{doc.importedAt.replace("T", " ").slice(0, 16)}</td>
                    <td className="action-col">
                      <ChevronRight size={14} className="row-chevron" />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
