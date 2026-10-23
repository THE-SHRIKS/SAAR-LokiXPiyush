import React from "react";
import { Search, RefreshCw, PanelRightClose, PanelRightOpen, Cpu, Sun, Moon } from "lucide-react";
import { SearchResult, WorkspaceInfo } from "../lib/types";

interface HeaderBarProps {
  workspace: WorkspaceInfo | null;
  activeTabTitle: string;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  searchResults: SearchResult[];
  isSearching: boolean;
  onSelectSearchResult: (result: SearchResult) => void;
  isInspectorOpen: boolean;
  onToggleInspector: () => void;
  hasInspectorContent: boolean;
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export const HeaderBar: React.FC<HeaderBarProps> = ({
  workspace: _workspace,
  activeTabTitle,
  searchQuery,
  onSearchChange,
  searchResults,
  isSearching,
  onSelectSearchResult,
  isInspectorOpen,
  onToggleInspector,
  hasInspectorContent,
  theme,
  onToggleTheme,
}) => {
  return (
    <header className="claude-header-bar">
      {/* Left: Active View / Thread Title */}
      <div className="header-left-col">
        <h2 className="header-active-title">{activeTabTitle}</h2>
      </div>

      {/* Center: Global SQLite FTS5 Search */}
      <div className="header-center-col">
        <div className="global-search-container">
          <div className="search-icon-slot">
            {isSearching ? (
              <RefreshCw size={13} className="spin icon-muted" />
            ) : (
              <Search size={13} className="icon-muted" />
            )}
          </div>
          <input
            type="text"
            className="global-search-input"
            placeholder="Search documents, invoices, transactions... (FTS5)"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
          />
          <kbd className="search-shortcut-kbd">Ctrl K</kbd>

          {searchResults.length > 0 && (
            <div className="search-results-popover">
              <div className="popover-header">
                <span>Matched Records ({searchResults.length})</span>
              </div>
              <div className="popover-results-list">
                {searchResults.map((res, idx) => (
                  <div
                    key={idx}
                    className="popover-result-row"
                    onClick={() => onSelectSearchResult(res)}
                  >
                    <div className="result-row-top">
                      <span className={`result-tag ${res.entityType.toLowerCase()}`}>
                        {res.entityType}
                      </span>
                      <span className="result-title">{res.title}</span>
                    </div>
                    <div
                      className="result-snippet"
                      dangerouslySetInnerHTML={{ __html: res.snippet }}
                    />
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Right: Model Status, Theme Switcher & Evidence Inspector Toggle */}
      <div className="header-right-col">
        {/* Theme Toggle Button */}
        <button
          className="theme-toggle-btn"
          onClick={onToggleTheme}
          title={theme === "dark" ? "Switch to Warm Linen Light Theme (#EAE0CF)" : "Switch to Midnight Navy Dark Theme (#111844)"}
          aria-label="Toggle Theme"
        >
          {theme === "dark" ? (
            <Sun size={14} className="theme-icon sun" />
          ) : (
            <Moon size={14} className="theme-icon moon" />
          )}
          <span className="theme-label">{theme === "dark" ? "Linen" : "Midnight"}</span>
        </button>

        <div className="model-chip" title="Local AVX2 inference via llama-cli. Zero WAN sockets.">
          <Cpu size={12} className="model-chip-icon" />
          <span className="model-chip-label">Qwen2.5-3B</span>
        </div>

        <button
          className={`inspector-toggle-btn ${isInspectorOpen ? "open" : ""} ${
            hasInspectorContent ? "has-content" : ""
          }`}
          onClick={onToggleInspector}
          title={isInspectorOpen ? "Close Evidence Inspector" : "Open Evidence Inspector"}
          aria-label="Toggle Evidence Inspector"
        >
          {isInspectorOpen ? <PanelRightClose size={15} /> : <PanelRightOpen size={15} />}
          <span className="inspector-btn-label">Evidence</span>
          {hasInspectorContent && <span className="evidence-dot" />}
        </button>
      </div>
    </header>
  );
};
