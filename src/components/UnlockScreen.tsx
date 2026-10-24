import React, { useEffect, useRef, useState } from "react";
import { Lock, Eye, EyeOff, ShieldCheck, ArrowRight, Sun, Moon } from "lucide-react";
import { SystemStatus, WorkspaceSummary } from "../lib/types";

export type UnlockMode =
  | { kind: "unlock"; workspaceId: string }
  | { kind: "create"; displayName: string; baseCurrency: string };

interface UnlockScreenProps {
  status: SystemStatus | null;
  workspaces: WorkspaceSummary[];
  onUnlock: (mode: UnlockMode, passphrase: string) => Promise<void>;
  isLoading: boolean;
  authError: string | null;
  theme?: "dark" | "light";
  onToggleTheme?: () => void;
}

const BASE_CURRENCIES = ["USD", "EUR", "GBP", "INR", "CAD", "AUD", "JPY"];
const MIN_PASSPHRASE_LENGTH = 12;

export const UnlockScreen: React.FC<UnlockScreenProps> = ({
  status,
  workspaces,
  onUnlock,
  isLoading,
  authError,
  theme,
  onToggleTheme,
}) => {
  const hasWorkspaces = workspaces.length > 0;
  const [creating, setCreating] = useState(!hasWorkspaces);
  const [selectedId, setSelectedId] = useState(workspaces[0]?.id ?? "");
  // The workspace list arrives after the first render on a cold start. Until the user chooses a
  // mode themselves, follow the list: existing workspaces mean "unlock", none means "create".
  const userChoseMode = useRef(false);
  useEffect(() => {
    if (!userChoseMode.current) setCreating(workspaces.length === 0);
    setSelectedId((current) =>
      workspaces.some((w) => w.id === current) ? current : (workspaces[0]?.id ?? ""),
    );
  }, [workspaces]);
  const [displayName, setDisplayName] = useState("");
  const [baseCurrency, setBaseCurrency] = useState("USD");
  const [passphrase, setPassphrase] = useState("");
  const [confirmPassphrase, setConfirmPassphrase] = useState("");
  const [showPassphrase, setShowPassphrase] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const isUnlockMode = !creating && hasWorkspaces;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (isUnlockMode) {
      if (!selectedId || !passphrase) return;
      onUnlock({ kind: "unlock", workspaceId: selectedId }, passphrase);
      return;
    }

    if (!displayName.trim()) {
      setLocalError("Give the workspace a name.");
      return;
    }
    if (passphrase.length < MIN_PASSPHRASE_LENGTH) {
      setLocalError(`Use a passphrase of at least ${MIN_PASSPHRASE_LENGTH} characters.`);
      return;
    }
    if (passphrase !== confirmPassphrase) {
      setLocalError("The passphrases do not match.");
      return;
    }
    onUnlock({ kind: "create", displayName: displayName.trim(), baseCurrency }, passphrase);
  };

  const errorText = localError ?? authError;

  return (
    <div className="unlock-screen-wrapper">
      {onToggleTheme && (
        <button
          className="unlock-theme-toggle"
          onClick={onToggleTheme}
          title={theme === "dark" ? "Switch to Warm Linen Light Theme" : "Switch to Midnight Navy Dark Theme"}
          aria-label="Toggle Theme"
        >
          {theme === "dark" ? <Sun size={15} /> : <Moon size={15} />}
          <span>{theme === "dark" ? "Linen" : "Midnight"}</span>
        </button>
      )}
      <div className="unlock-card-container">
        {/* Brand Monogram & Header */}
        <div className="unlock-brand-header">
          <div className="unlock-monogram">S</div>
          <h1 className="unlock-app-title">SAAR</h1>
          <p className="unlock-app-tagline">Sovereign Financial Intelligence</p>
        </div>

        {/* Security Assurance Tag */}
        <div className="unlock-assurance-pill">
          <ShieldCheck size={13} className="pill-icon" />
          <span>Air-Gapped · Local Hardware · Zero Cloud Egress</span>
        </div>

        <form onSubmit={handleSubmit} className="unlock-form">
          {isUnlockMode ? (
            <div className="form-field-group">
              <label className="field-label" htmlFor="workspace-select">
                Workspace
              </label>
              <select
                id="workspace-select"
                className="field-input"
                value={selectedId}
                onChange={(e) => setSelectedId(e.target.value)}
                disabled={isLoading}
              >
                {workspaces.map((ws) => (
                  <option key={ws.id} value={ws.id}>
                    {ws.displayName} · created {ws.createdAt.slice(0, 10)}
                  </option>
                ))}
              </select>
            </div>
          ) : (
            <>
              <div className="form-field-group">
                <label className="field-label" htmlFor="workspace-name-input">
                  Workspace name
                </label>
                <input
                  id="workspace-name-input"
                  type="text"
                  className="field-input"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="e.g. Company operations"
                  disabled={isLoading}
                  required
                />
              </div>

              <div className="form-field-group">
                <label className="field-label" htmlFor="base-currency-select">
                  Base currency
                </label>
                <select
                  id="base-currency-select"
                  className="field-input"
                  value={baseCurrency}
                  onChange={(e) => setBaseCurrency(e.target.value)}
                  disabled={isLoading}
                >
                  {BASE_CURRENCIES.map((code) => (
                    <option key={code} value={code}>
                      {code}
                    </option>
                  ))}
                </select>
              </div>
            </>
          )}

          <div className="form-field-group">
            <label className="field-label" htmlFor="passphrase-input">
              Master passphrase
            </label>
            <div className="passphrase-input-wrapper">
              <input
                id="passphrase-input"
                type={showPassphrase ? "text" : "password"}
                className="field-input passphrase"
                value={passphrase}
                onChange={(e) => setPassphrase(e.target.value)}
                placeholder={isUnlockMode ? "Enter workspace passphrase" : "Choose a passphrase"}
                autoComplete={isUnlockMode ? "current-password" : "new-password"}
                disabled={isLoading}
                required
                autoFocus
              />
              <button
                type="button"
                className="toggle-visibility-btn"
                onClick={() => setShowPassphrase(!showPassphrase)}
                tabIndex={-1}
                aria-label={showPassphrase ? "Hide passphrase" : "Show passphrase"}
              >
                {showPassphrase ? <EyeOff size={15} /> : <Eye size={15} />}
              </button>
            </div>
          </div>

          {!isUnlockMode && (
            <div className="form-field-group">
              <label className="field-label" htmlFor="passphrase-confirm-input">
                Confirm passphrase
              </label>
              <input
                id="passphrase-confirm-input"
                type={showPassphrase ? "text" : "password"}
                className="field-input passphrase"
                value={confirmPassphrase}
                onChange={(e) => setConfirmPassphrase(e.target.value)}
                autoComplete="new-password"
                disabled={isLoading}
                required
              />
            </div>
          )}

          {errorText && (
            <div className="auth-error-banner" role="alert">
              <span>{errorText}</span>
            </div>
          )}

          <div className="form-actions-stack">
            <button type="submit" className="unlock-submit-btn" disabled={isLoading || !passphrase}>
              <Lock size={14} />
              <span>
                {isLoading
                  ? "Deriving keys…"
                  : isUnlockMode
                    ? "Unlock workspace"
                    : "Create encrypted workspace"}
              </span>
              <ArrowRight size={14} className="submit-arrow" />
            </button>

            {hasWorkspaces && (
              <button
                type="button"
                className="quick-demo-btn"
                onClick={() => {
                  userChoseMode.current = true;
                  setLocalError(null);
                  setCreating(!creating);
                }}
                disabled={isLoading}
              >
                <span>{creating ? "Unlock an existing workspace" : "Create a new workspace"}</span>
              </button>
            )}
          </div>
        </form>

        {/* Footer Cryptographic Architecture Note */}
        <div className="unlock-crypto-footer">
          <div className="crypto-badges-row">
            <span className="crypto-badge">Argon2id (64 MiB, 3 passes)</span>
            <span className="crypto-separator">·</span>
            <span className="crypto-badge">SQLCipher AES-256-GCM</span>
          </div>
          <div className="runtime-meta-row">
            <span>Runtime {status?.appVersion ?? "—"}</span>
            <span className="crypto-separator">·</span>
            <span>Platform: {status?.platform ?? "—"}</span>
            <span className="crypto-separator">·</span>
            <span>Model: {status?.llmModelId ?? "—"}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
