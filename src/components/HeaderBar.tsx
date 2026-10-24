import React from "react";
import { Search, RefreshCw, PanelRightClose, PanelRightOpen, PanelLeftOpen, Cpu, Sun, Moon } from "lucide-react";
import { SearchResult, WorkspaceInfo } from "../lib/types";

interface HeaderBarProps {
  sidebarCollapsed: boolean;
  onToggleSidebar: () => void;
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
  modelLabel: string;
}

export const HeaderBar: React.FC<HeaderBarProps> = ({
  workspace: _workspace,
  sidebarCollapsed,
  onToggleSidebar,
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
  modelLabel,
}) => {
  return (
    <header className="claude-header-bar">
      {/* Left: Active View / Thread Title */}
      <div className="header-left-col">
        {sidebarCollapsed && (
          <button className="header-icon-btn" onClick={onToggleSidebar} title="Show sidebar (Ctrl+B)" aria-label="Show sidebar">
            <PanelLeftOpen size={16} />
          </button>
        )}
        <h2 className="header-active-title">{activeTabTitle}</h2>
      </div>

      {/* Center: global search */}
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
            placeholder="Search documents, invoices and transactions"
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
          title={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
          aria-label="Toggle Theme"
        >
          {theme === "dark" ? (
            <Sun size={14} className="theme-icon sun" />
          ) : (
            <Moon size={14} className="theme-icon moon" />
          )}
          <span className="theme-label">{theme === "dark" ? "Light" : "Dark"}</span>
        </button>

        <div className="model-chip" title="Local inference with llama.cpp. The model runs on this computer.">
          <Cpu size={12} className="model-chip-icon" />
          <span className="model-chip-label">{modelLabel}</span>
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
