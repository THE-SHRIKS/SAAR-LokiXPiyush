import React from "react";
import { MessageSquare, FileText, GitCompare, BarChart3, ShieldCheck, Lock } from "lucide-react";

export type WorkspaceTab = "chat" | "documents" | "review" | "reports" | "security";

interface NavRailProps {
  activeTab: WorkspaceTab;
  onSelectTab: (tab: WorkspaceTab) => void;
  pendingReviewCount: number;
  onLockWorkspace: () => void;
  isUnlocked: boolean;
}

export const NavRail: React.FC<NavRailProps> = ({
  activeTab,
  onSelectTab,
  pendingReviewCount,
  onLockWorkspace,
  isUnlocked,
}) => {
  return (
    <aside className="nav-rail">
      {/* Brand Monogram */}
      <div className="rail-brand" title="SAAR — Local Financial Workstation">
        <div className="brand-monogram">S</div>
      </div>

      {/* Main Navigation Stack */}
      <nav className="rail-nav">
        <button
          className={`rail-btn ${activeTab === "chat" ? "active" : ""}`}
          onClick={() => onSelectTab("chat")}
          title="Conversation Workspace (Primary)"
          aria-label="Conversation Workspace"
        >
          <div className="rail-icon-wrap">
            <MessageSquare size={18} strokeWidth={2} />
          </div>
          <span className="rail-label">Chat</span>
        </button>

        <button
          className={`rail-btn ${activeTab === "documents" ? "active" : ""}`}
          onClick={() => onSelectTab("documents")}
          title="Document & Invoice Ingestion"
          aria-label="Documents"
        >
          <div className="rail-icon-wrap">
            <FileText size={18} strokeWidth={2} />
          </div>
          <span className="rail-label">Docs</span>
        </button>

        <button
          className={`rail-btn ${activeTab === "review" ? "active" : ""}`}
          onClick={() => onSelectTab("review")}
          title="Reconciliation & Ledger Review"
          aria-label="Review"
        >
          <div className="rail-icon-wrap">
            <GitCompare size={18} strokeWidth={2} />
            {pendingReviewCount > 0 && <span className="rail-badge">{pendingReviewCount}</span>}
          </div>
          <span className="rail-label">Review</span>
        </button>

        <button
          className={`rail-btn ${activeTab === "reports" ? "active" : ""}`}
          onClick={() => onSelectTab("reports")}
          title="Deterministic Financial Reports"
          aria-label="Reports"
        >
          <div className="rail-icon-wrap">
            <BarChart3 size={18} strokeWidth={2} />
          </div>
          <span className="rail-label">Reports</span>
        </button>

        <button
          className={`rail-btn ${activeTab === "security" ? "active" : ""}`}
          onClick={() => onSelectTab("security")}
          title="Security, Keys & Offline Runtime"
          aria-label="Security"
        >
          <div className="rail-icon-wrap">
            <ShieldCheck size={18} strokeWidth={2} />
          </div>
          <span className="rail-label">Security</span>
        </button>
      </nav>

      {/* Rail Footer */}
      <div className="rail-footer">
        <div className="rail-status-dot" title="Zero WAN egress · connect-src 'none' active">
          <span className="dot-airgap" />
        </div>

        {isUnlocked && (
          <button
            className="rail-lock-btn"
            onClick={onLockWorkspace}
            title="Lock workspace and zeroize memory keys"
            aria-label="Lock Workspace"
          >
            <Lock size={15} />
          </button>
        )}
      </div>
    </aside>
  );
};
