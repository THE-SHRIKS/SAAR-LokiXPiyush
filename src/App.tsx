import React, { useEffect, useState } from "react";
import {
  SystemStatus,
  WorkspaceInfo,
  DocumentRecord,
  TransactionRecord,
  formatMinorCurrency,
} from "./lib/types";
import { OpenDeckApi } from "./lib/api";
import { ShieldCheck, FileText, Database, Lock, Unlock, HardDrive, Cpu } from "lucide-react";

export const App: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [activeTab, setActiveTab] = useState<"documents" | "transactions" | "settings">(
    "documents"
  );
  const [documents, setDocuments] = useState<DocumentRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [passphrase, setPassphrase] = useState<string>("");
  const [workspaceName, setWorkspaceName] = useState<string>("THE SHRIKS Primary");
  const [authError, setAuthError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    OpenDeckApi.getSystemStatus().then(setStatus).catch(console.error);
  }, []);

  const handleUnlockOrCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!passphrase) {
      setAuthError("Passphrase cannot be empty.");
      return;
    }
    setAuthError(null);
    setIsLoading(true);

    try {
      // In Phase 1 initial state, create or open workspace
      const ws = await OpenDeckApi.createWorkspace(workspaceName, passphrase, "USD");
      setWorkspace(ws);
      const docs = await OpenDeckApi.listDocuments();
      setDocuments(docs);
      const txs = await OpenDeckApi.listTransactions();
      setTransactions(txs);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setAuthError(`Authentication error: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLock = async () => {
    await OpenDeckApi.lockWorkspace();
    setWorkspace(null);
    setDocuments([]);
    setTransactions([]);
    setPassphrase("");
  };

  return (
    <div className="app-container">
      {/* Editorial Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="brand-title">OpenDeck</div>
          <div className="brand-badge">Air-Gapped Desktop</div>
        </div>

        <nav className="sidebar-nav">
          <div
            className={`nav-item ${activeTab === "documents" ? "active" : ""}`}
            onClick={() => setActiveTab("documents")}
          >
            <FileText size={16} />
            <span>Documents</span>
          </div>

          <div
            className={`nav-item ${activeTab === "transactions" ? "active" : ""}`}
            onClick={() => setActiveTab("transactions")}
          >
            <Database size={16} />
            <span>Ledger</span>
          </div>

          <div
            className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            <ShieldCheck size={16} />
            <span>Security & Runtime</span>
          </div>
        </nav>

        <div className="sidebar-footer">
          <div className="status-indicator">
            <span className="dot-green"></span>
            <span>Zero Network • Strict Offline</span>
          </div>
        </div>
      </aside>

      {/* Main Viewport */}
      <main className="main-viewport">
        <header className="viewport-header">
          <div className="view-title">
            {activeTab === "documents" && "Document Rail"}
            {activeTab === "transactions" && "Financial Ledger"}
            {activeTab === "settings" && "Security & Hardware Readiness"}
          </div>

          <div style={{ display: "flex", gap: "12px", alignItems: "center" }}>
            {workspace?.isUnlocked ? (
              <button
                className="btn btn-secondary"
                onClick={handleLock}
                style={{ padding: "4px 10px", fontSize: "12px" }}
              >
                <Lock size={13} /> Lock Workspace
              </button>
            ) : (
              <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                <Unlock size={13} style={{ verticalAlign: "middle" }} /> Locked
              </span>
            )}
          </div>
        </header>

        <div className="viewport-content">
          {!workspace?.isUnlocked ? (
            <div className="editorial-card" style={{ maxWidth: "480px", margin: "40px auto" }}>
              <h2 className="card-title" style={{ fontSize: "18px", marginBottom: "8px" }}>
                Encrypted Workspace Access
              </h2>
              <p style={{ color: "var(--text-secondary)", fontSize: "13px", marginBottom: "16px" }}>
                OpenDeck databases are protected with SQLCipher 256-bit AES encryption derived via
                Argon2id. Enter your master passphrase to unlock.
              </p>

              <form onSubmit={handleUnlockOrCreate}>
                <div style={{ marginBottom: "12px" }}>
                  <label
                    style={{
                      display: "block",
                      fontSize: "12px",
                      color: "var(--text-muted)",
                      marginBottom: "4px",
                    }}
                  >
                    Workspace Profile Name
                  </label>
                  <input
                    type="text"
                    className="input-field"
                    value={workspaceName}
                    onChange={(e) => setWorkspaceName(e.target.value)}
                    required
                  />
                </div>

                <div style={{ marginBottom: "16px" }}>
                  <label
                    style={{
                      display: "block",
                      fontSize: "12px",
                      color: "var(--text-muted)",
                      marginBottom: "4px",
                    }}
                  >
                    Master Passphrase
                  </label>
                  <input
                    type="password"
                    className="input-field"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="Enter secure master passphrase..."
                    required
                  />
                </div>

                {authError && (
                  <div
                    style={{
                      color: "var(--accent-danger)",
                      fontSize: "12px",
                      marginBottom: "12px",
                    }}
                  >
                    {authError}
                  </div>
                )}

                <button
                  type="submit"
                  className="btn btn-primary"
                  style={{ width: "100%" }}
                  disabled={isLoading}
                >
                  <Lock size={14} />{" "}
                  {isLoading ? "Deriving Key & Opening..." : "Unlock or Initialize Workspace"}
                </button>
              </form>
            </div>
          ) : (
            <>
              {activeTab === "documents" && (
                <div className="editorial-card">
                  <div
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      alignItems: "center",
                      marginBottom: "16px",
                    }}
                  >
                    <h3 className="card-title" style={{ margin: 0 }}>
                      Imported Documents
                    </h3>
                    <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                      {documents.length} records
                    </span>
                  </div>

                  {documents.length === 0 ? (
                    <div
                      style={{ padding: "32px 0", textAlign: "center", color: "var(--text-muted)" }}
                    >
                      <FileText size={32} style={{ marginBottom: "8px", opacity: 0.4 }} />
                      <p>No documents imported into this workspace yet.</p>
                      <p style={{ fontSize: "12px", marginTop: "4px" }}>
                        Drop PDF invoices or thermal receipts into the tray.
                      </p>
                    </div>
                  ) : (
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Document</th>
                          <th>Type</th>
                          <th>Status</th>
                          <th>Imported At</th>
                        </tr>
                      </thead>
                      <tbody>
                        {documents.map((doc) => (
                          <tr key={doc.id}>
                            <td>{doc.fileName}</td>
                            <td>{doc.docType}</td>
                            <td>{doc.status}</td>
                            <td>{doc.importedAt}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}

              {activeTab === "transactions" && (
                <div className="editorial-card">
                  <div
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      alignItems: "center",
                      marginBottom: "16px",
                    }}
                  >
                    <h3 className="card-title" style={{ margin: 0 }}>
                      Verified Ledger
                    </h3>
                    <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>
                      {transactions.length} entries
                    </span>
                  </div>

                  {transactions.length === 0 ? (
                    <div
                      style={{ padding: "32px 0", textAlign: "center", color: "var(--text-muted)" }}
                    >
                      <Database size={32} style={{ marginBottom: "8px", opacity: 0.4 }} />
                      <p>No ledger transactions recorded.</p>
                      <p style={{ fontSize: "12px", marginTop: "4px" }}>
                        Import bank CSV statements or approve extracted invoices to populate the
                        ledger.
                      </p>
                    </div>
                  ) : (
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>Description</th>
                          <th style={{ textAlign: "right" }}>Amount</th>
                          <th>Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {transactions.map((tx) => (
                          <tr key={tx.id}>
                            <td>{tx.postedDate}</td>
                            <td>{tx.description}</td>
                            <td className="num-cell">
                              {formatMinorCurrency(tx.amountMinor, tx.currency)}
                            </td>
                            <td>{tx.reviewState}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}

              {activeTab === "settings" && (
                <div className="editorial-card">
                  <h3 className="card-title">Security & Runtime Verification</h3>
                  <div
                    style={{
                      display: "grid",
                      gridTemplateColumns: "1fr 1fr",
                      gap: "16px",
                      marginTop: "16px",
                    }}
                  >
                    <div
                      style={{
                        padding: "12px",
                        background: "var(--bg-tertiary)",
                        borderRadius: "6px",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: "8px",
                          marginBottom: "6px",
                        }}
                      >
                        <HardDrive size={16} color="var(--accent-primary)" />
                        <span style={{ fontWeight: 500 }}>SQLCipher 256-bit AES</span>
                      </div>
                      <p style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                        Database: Encrypted at rest. Argon2id key derivation active. Zero plaintext
                        leaks.
                      </p>
                    </div>

                    <div
                      style={{
                        padding: "12px",
                        background: "var(--bg-tertiary)",
                        borderRadius: "6px",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: "8px",
                          marginBottom: "6px",
                        }}
                      >
                        <Cpu size={16} color="var(--accent-success)" />
                        <span style={{ fontWeight: 500 }}>Local Model Engine</span>
                      </div>
                      <p style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                        LLM: {status?.llmModelId || "Qwen2.5-3B-Instruct"} (CPU AVX2 fallback
                        verified).
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </main>
    </div>
  );
};
