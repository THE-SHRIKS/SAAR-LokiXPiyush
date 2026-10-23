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
  isImporting: boolean;
  importMessage: { text: string; type: "success" | "error" | "info" } | null;
}

export const DocumentsView: React.FC<DocumentsViewProps> = ({
  documents,
  selectedDocumentId,
  onSelectDocument,
  onIngestFile,
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
        <div className="ingest-card-head">
          <div className="ingest-title-wrap">
            <UploadCloud size={16} className="icon-cream" />
            <h3>Air-Gapped Ingestion Pipeline</h3>
          </div>
          <span className="ingest-guidance">
            Eligible: Digital PDF, Scanned OCR Image (PNG/JPG), Bank Statement CSV (Max 50MB, 50
            pages)
          </span>
        </div>

        <form onSubmit={handleSubmit} className="ingest-input-row">
          <input
            type="text"
            className="input-text flex-grow"
            placeholder="Enter the absolute path to a CSV, PDF, PNG or JPG file..."
            value={inputPath}
            onChange={(e) => setInputPath(e.target.value)}
            disabled={isImporting}
          />
          <button
            type="submit"
            className="btn-primary-medium"
            disabled={isImporting || !inputPath.trim()}
          >
            {isImporting ? "Processing OCR..." : "Ingest Document"}
          </button>
        </form>

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
            <span className="toolbar-sub">AES-256-GCM Content-Addressed Blob Storage</span>
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
            <p>No documents found matching the filter.</p>
            <span>Import a PDF invoice or bank statement using the bar above.</span>
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
                    <td className="font-mono text-muted text-xs">{doc.importedAt}</td>
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
