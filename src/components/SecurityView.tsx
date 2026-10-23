import React from "react";
import {
  ShieldCheck,
  HardDrive,
  Cpu,
  Archive,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";
import { SystemStatus } from "../lib/types";

interface SecurityViewProps {
  status: SystemStatus | null;
  backupPassphrase: string;
  setBackupPassphrase: (pass: string) => void;
  isBackupLoading: boolean;
  backupFeedback: string | null;
  onCreateBackup: (e: React.FormEvent) => Promise<void>;
  restoreArchivePath: string;
  setRestoreArchivePath: (path: string) => void;
  restorePassphrase: string;
  setRestorePassphrase: (pass: string) => void;
  isRestoreLoading: boolean;
  restoreFeedback: string | null;
  onRestoreBackup: (e: React.FormEvent) => Promise<void>;
}

export const SecurityView: React.FC<SecurityViewProps> = ({
  status,
  backupPassphrase,
  setBackupPassphrase,
  isBackupLoading,
  backupFeedback,
  onCreateBackup,
  restoreArchivePath,
  setRestoreArchivePath,
  restorePassphrase,
  setRestorePassphrase,
  isRestoreLoading,
  restoreFeedback,
  onRestoreBackup,
}) => {
  return (
    <div className="security-workspace">
      <div className="security-header-block">
        <h3>Security Boundary & Local Runtime Verification</h3>
        <span className="security-header-sub">
          Cryptographic enforcement · Zero cloud egress · Memory-hard Argon2id key derivation
        </span>
      </div>

      {/* Security Specification Tiles */}
      <div className="security-specs-grid">
        <div className="sec-spec-card">
          <div className="spec-card-top">
            <HardDrive size={16} className="icon-cream" />
            <h4>Storage & Cryptography</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">Database Engine</span>
              <span className="spec-item-val font-mono">SQLite 3 (WAL + FTS5)</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Key Derivation</span>
              <span className="spec-item-val font-mono">Argon2id (64 MiB RAM, 3 Passes)</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Blob Storage</span>
              <span className="spec-item-val font-mono">AES-256-GCM Authenticated</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Deduplication</span>
              <span className="spec-item-val font-mono">SHA-256 Content-Addressed</span>
            </div>
          </div>
        </div>

        <div className="sec-spec-card">
          <div className="spec-card-top">
            <ShieldCheck size={16} className="icon-cream" />
            <h4>Air-Gap Network Boundary</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">Content Security Policy</span>
              <span className="spec-item-val font-mono">connect-src &apos;none&apos;</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Remote Outbound Sockets</span>
              <span className="spec-item-val font-mono">0 (Strictly Blocked)</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Telemetry & Analytics</span>
              <span className="spec-item-val font-mono">Permanently Disabled</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Silent Background Updates</span>
              <span className="spec-item-val font-mono">Zero</span>
            </div>
          </div>
        </div>

        <div className="sec-spec-card">
          <div className="spec-card-top">
            <Cpu size={16} className="icon-cream" />
            <h4>Local Document & AI Runtimes</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">App Version / OS</span>
              <span className="spec-item-val font-mono">
                v{status?.appVersion || "0.1.0"} ({status?.platform || "windows"})
              </span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Local LLM</span>
              <span className="spec-item-val font-mono">
                {status?.llmModelId || "qwen-2.5-3b-instruct-q3_k_m"}
              </span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Inference Mode</span>
              <span className="spec-item-val font-mono">llama.cpp CPU AVX2 (Greedy)</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">OCR Engine</span>
              <span className="spec-item-val font-mono">RapidOCR (DBNet + SVTR ONNX)</span>
            </div>
          </div>
        </div>
      </div>

      {/* Encrypted Backup & Disaster Recovery Card */}
      <div className="backup-card">
        <div className="backup-card-head">
          <Archive size={16} className="icon-cream" />
          <h4>Authenticated Encrypted Backup & Disaster Recovery (.saar)</h4>
        </div>
        <p className="backup-card-desc">
          Generate an authenticated, tamper-evident container packaging the SQLite database and blob
          store. Restorable offline on any Windows workstation with your passphrase.
        </p>

        <div className="backup-split-grid">
          {/* Create Backup */}
          <form onSubmit={onCreateBackup} className="backup-subcard">
            <h5>Create Encrypted .saar Archive</h5>
            <p className="subcard-desc">Derive AES-256 backup key from master passphrase.</p>
            <input
              type="password"
              className="input-text"
              placeholder="Enter backup passphrase..."
              value={backupPassphrase}
              onChange={(e) => setBackupPassphrase(e.target.value)}
              disabled={isBackupLoading}
            />
            <button
              type="submit"
              className="btn-primary-medium"
              disabled={isBackupLoading || !backupPassphrase.trim()}
            >
              <Archive size={13} />
              <span>{isBackupLoading ? "Encrypting Archive..." : "Create Encrypted Backup"}</span>
            </button>

            {backupFeedback && (
              <div
                className={`feedback-msg ${backupFeedback.includes("failed") ? "error" : "success"}`}
              >
                {backupFeedback.includes("failed") ? (
                  <AlertCircle size={13} className="icon-danger" />
                ) : (
                  <CheckCircle2 size={13} className="icon-green" />
                )}
                <span>{backupFeedback}</span>
              </div>
            )}
          </form>

          {/* Restore Backup */}
          <form onSubmit={onRestoreBackup} className="backup-subcard">
            <h5>Restore From .saar Archive</h5>
            <p className="subcard-desc">Decrypt archive into local data directories.</p>
            <input
              type="text"
              className="input-text"
              placeholder="Absolute path to .saar archive..."
              value={restoreArchivePath}
              onChange={(e) => setRestoreArchivePath(e.target.value)}
              disabled={isRestoreLoading}
            />
            <input
              type="password"
              className="input-text"
              placeholder="Archive passphrase..."
              value={restorePassphrase}
              onChange={(e) => setRestorePassphrase(e.target.value)}
              disabled={isRestoreLoading}
            />
            <button
              type="submit"
              className="btn-secondary-medium"
              disabled={isRestoreLoading || !restoreArchivePath.trim() || !restorePassphrase.trim()}
            >
              <RefreshCw size={13} className={isRestoreLoading ? "spin" : ""} />
              <span>{isRestoreLoading ? "Decrypting & Verifying..." : "Restore Workspace"}</span>
            </button>

            {restoreFeedback && (
              <div
                className={`feedback-msg ${restoreFeedback.includes("failed") ? "error" : "success"}`}
              >
                {restoreFeedback.includes("failed") ? (
                  <AlertCircle size={13} className="icon-danger" />
                ) : (
                  <CheckCircle2 size={13} className="icon-green" />
                )}
                <span>{restoreFeedback}</span>
              </div>
            )}
          </form>
        </div>
      </div>
    </div>
  );
};
