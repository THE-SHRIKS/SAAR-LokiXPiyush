import { pickBackupArchive } from "../lib/files";
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
        <h3>Security</h3>
        <span className="security-header-sub">
          How your data is protected, and what the app runs on this computer.
        </span>
      </div>

      {/* Security Specification Tiles */}
      <div className="security-specs-grid">
        <div className="sec-spec-card">
          <div className="spec-card-top">
            <HardDrive size={16} className="icon-cream" />
            <h4>Storage and encryption</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">Database</span>
              <span className="spec-item-val font-mono">SQLCipher, one encrypted file per workspace</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Key derivation</span>
              <span className="spec-item-val font-mono">Argon2id (64 MiB, 3 passes)</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Stored documents</span>
              <span className="spec-item-val font-mono">AES-256-GCM</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Duplicate detection</span>
              <span className="spec-item-val font-mono">SHA-256 of file contents</span>
            </div>
          </div>
        </div>

        <div className="sec-spec-card">
          <div className="spec-card-top">
            <ShieldCheck size={16} className="icon-cream" />
            <h4>Network</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">Page network policy</span>
              <span className="spec-item-val font-mono">connect-src &apos;none&apos;</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">App network features</span>
              <span className="spec-item-val font-mono">None used</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Telemetry</span>
              <span className="spec-item-val font-mono">None</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Browser component</span>
              <span className="spec-item-val font-mono">Windows WebView2; SAAR does not control its own connections</span>
            </div>
          </div>
        </div>

        <div className="sec-spec-card">
          <div className="spec-card-top">
            <Cpu size={16} className="icon-cream" />
            <h4>Runtimes</h4>
          </div>
          <div className="spec-item-list">
            <div className="spec-item">
              <span className="spec-item-label">Version</span>
              <span className="spec-item-val font-mono">
                v{status?.appVersion || "0.1.0"} ({status?.platform || "windows"})
              </span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Language model</span>
              <span className="spec-item-val font-mono">
                {status?.llmModelLabel || "none"} ({status?.llmModelId || "not installed"})
              </span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Inference</span>
              <span className="spec-item-val font-mono">llama.cpp, GPU when available, otherwise CPU</span>
            </div>
            <div className="spec-item">
              <span className="spec-item-label">Text recognition</span>
              <span className="spec-item-val font-mono">RapidOCR (DBNet + SVTR ONNX)</span>
            </div>
          </div>
        </div>
      </div>

      {/* Encrypted Backup & Disaster Recovery Card */}
      <div className="backup-card">
        <div className="backup-card-head">
          <Archive size={16} className="icon-cream" />
          <h4>Encrypted backup and restore</h4>
        </div>
        <p className="backup-card-desc">
          Generate an authenticated, tamper-evident container packaging the encrypted database and stored
          documents. Restorable offline on any Windows workstation with your passphrase.
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
            <div className="path-pick-row">
              <input
                type="text"
                className="input-text"
                placeholder="No backup chosen"
                value={restoreArchivePath}
                onChange={(e) => setRestoreArchivePath(e.target.value)}
                disabled={isRestoreLoading}
              />
              <button
                type="button"
                className="btn-secondary-medium"
                disabled={isRestoreLoading}
                onClick={async () => {
                  const chosen = await pickBackupArchive().catch(() => null);
                  if (chosen) setRestoreArchivePath(chosen);
                }}
              >
                Browse
              </button>
            </div>
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
