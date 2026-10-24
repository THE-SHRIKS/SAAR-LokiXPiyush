import React from "react";
import {
  Plus,
  Pin,
  PinOff,
  Trash2,
  MessageSquare,
  FileText,
  GitCompare,
  BarChart3,
  ShieldCheck,
  Lock,
  Sun,
  Moon,
} from "lucide-react";
import { ChatThread, WorkspaceInfo } from "../lib/types";

export type WorkspaceTab = "chat" | "documents" | "review" | "reports" | "security";

interface SidebarProps {
  workspace: WorkspaceInfo | null;
  activeTab: WorkspaceTab;
  onSelectTab: (tab: WorkspaceTab) => void;
  threads: ChatThread[];
  activeThreadId: string | null;
  onSelectThread: (threadId: string) => void;
  onNewChat: () => void;
  onDeleteThread: (threadId: string, e: React.MouseEvent) => void;
  onTogglePinThread: (threadId: string, e: React.MouseEvent) => void;
  pendingReviewCount: number;
  documentCount: number;
  onLockWorkspace: () => void;
  theme?: "dark" | "light";
  onToggleTheme?: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  workspace,
  activeTab,
  onSelectTab,
  threads,
  activeThreadId,
  onSelectThread,
  onNewChat,
  onDeleteThread,
  onTogglePinThread,
  pendingReviewCount,
  documentCount,
  onLockWorkspace,
  theme,
  onToggleTheme,
}) => {
  const pinnedThreads = threads.filter((t) => t.pinned);
  const recentThreads = threads.filter((t) => !t.pinned);

  return (
    <aside className="claude-sidebar">
      {/* Brand & Air-Gap Badge */}
      <div className="sidebar-brand-block">
        <div className="brand-header-row">
          <div className="brand-monogram">S</div>
          <div className="brand-text-col">
            <span className="brand-title">SAAR</span>
            <span className="brand-subtitle">Local financial workspace</span>
          </div>
        </div>
        <div className="airgap-badge">
          <span className="airgap-dot" />
          <span className="airgap-text">Runs on this computer</span>
        </div>
      </div>

      {/* Start New Chat Action */}
      <div className="sidebar-new-chat-container">
        <button
          className="new-chat-btn"
          onClick={onNewChat}
          title="Start new conversation thread (Ctrl+N)"
        >
          <div className="new-chat-content">
            <Plus size={15} strokeWidth={2.2} className="new-chat-icon" />
            <span className="new-chat-text">Start new chat</span>
          </div>
          <kbd className="new-chat-kbd">Ctrl N</kbd>
        </button>
      </div>

      {/* Conversations List Area */}
      <div className="sidebar-scrollable-area">
        {/* Pinned Threads Section */}
        {pinnedThreads.length > 0 && (
          <div className="thread-section">
            <div className="section-header-label">
              <span>Pinned</span>
            </div>
            <div className="thread-items-list">
              {pinnedThreads.map((thread) => {
                const isActive = activeTab === "chat" && activeThreadId === thread.id;
                return (
                  <div
                    key={thread.id}
                    className={`thread-item ${isActive ? "active" : ""}`}
                    onClick={() => {
                      onSelectTab("chat");
                      onSelectThread(thread.id);
                    }}
                    title={thread.title}
                  >
                    <MessageSquare size={13} className="thread-icon" />
                    <span className="thread-title-text">{thread.title}</span>
                    <div className="thread-item-actions">
                      <button
                        className="thread-action-btn pin active"
                        onClick={(e) => onTogglePinThread(thread.id, e)}
                        title="Unpin conversation"
                      >
                        <PinOff size={12} />
                      </button>
                      <button
                        className="thread-action-btn delete"
                        onClick={(e) => onDeleteThread(thread.id, e)}
                        title="Delete conversation"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Recent Threads Section */}
        <div className="thread-section">
          <div className="section-header-label">
            <span>Recent</span>
          </div>
          <div className="thread-items-list">
            {recentThreads.length === 0 ? (
              <div className="empty-threads-note">No recent conversations</div>
            ) : (
              recentThreads.map((thread) => {
                const isActive = activeTab === "chat" && activeThreadId === thread.id;
                return (
                  <div
                    key={thread.id}
                    className={`thread-item ${isActive ? "active" : ""}`}
                    onClick={() => {
                      onSelectTab("chat");
                      onSelectThread(thread.id);
                    }}
                    title={thread.title}
                  >
                    <MessageSquare size={13} className="thread-icon" />
                    <span className="thread-title-text">{thread.title}</span>
                    <div className="thread-item-actions">
                      <button
                        className="thread-action-btn pin"
                        onClick={(e) => onTogglePinThread(thread.id, e)}
                        title="Pin conversation"
                      >
                        <Pin size={12} />
                      </button>
                      <button
                        className="thread-action-btn delete"
                        onClick={(e) => onDeleteThread(thread.id, e)}
                        title="Delete conversation"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Dedicated Financial Workspaces */}
        <div className="thread-section secondary-tools-section">
          <div className="section-header-label">
            <span>Workspace</span>
          </div>
          <div className="tool-links-list">
            <button
              className={`tool-nav-btn ${activeTab === "documents" ? "active" : ""}`}
              onClick={() => onSelectTab("documents")}
            >
              <div className="tool-nav-content">
                <FileText size={14} className="tool-icon" />
                <span className="tool-label">Documents</span>
              </div>
              {documentCount > 0 && <span className="tool-badge">{documentCount}</span>}
            </button>

            <button
              className={`tool-nav-btn ${activeTab === "review" ? "active" : ""}`}
              onClick={() => onSelectTab("review")}
            >
              <div className="tool-nav-content">
                <GitCompare size={14} className="tool-icon" />
                <span className="tool-label">Reconciliation</span>
              </div>
              {pendingReviewCount > 0 && (
                <span className="tool-badge pending">{pendingReviewCount}</span>
              )}
            </button>

            <button
              className={`tool-nav-btn ${activeTab === "reports" ? "active" : ""}`}
              onClick={() => onSelectTab("reports")}
            >
              <div className="tool-nav-content">
                <BarChart3 size={14} className="tool-icon" />
                <span className="tool-label">Reports</span>
              </div>
            </button>

            <button
              className={`tool-nav-btn ${activeTab === "security" ? "active" : ""}`}
              onClick={() => onSelectTab("security")}
            >
              <div className="tool-nav-content">
                <ShieldCheck size={14} className="tool-icon" />
                <span className="tool-label">Security</span>
              </div>
            </button>
          </div>
        </div>
      </div>

      {/* Sidebar Footer — Workspace Profile & Lock */}
      <div className="sidebar-footer">
        <div className="workspace-profile-card">
          <div className="profile-info">
            <span className="profile-name">{workspace?.displayName || "THE SHRIKS Primary"}</span>
            <span className="profile-status">Argon2id · SQLCipher Vault</span>
          </div>
          <div className="profile-actions-row">
            {onToggleTheme && (
              <button
                className="workspace-theme-trigger"
                onClick={onToggleTheme}
                title={theme === "dark" ? "Switch to Warm Linen Light Theme" : "Switch to Midnight Navy Dark Theme"}
                aria-label="Toggle Theme"
              >
                {theme === "dark" ? <Sun size={14} /> : <Moon size={14} />}
              </button>
            )}
            <button
              className="workspace-lock-trigger"
              onClick={onLockWorkspace}
              title="Lock workspace and zeroize memory keys"
              aria-label="Lock Workspace"
            >
              <Lock size={14} />
            </button>
          </div>
        </div>
      </div>
    </aside>
  );
};
