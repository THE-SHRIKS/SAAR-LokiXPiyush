import React, { useState, useRef, useEffect } from "react";
import { Send, Paperclip, CheckCircle2, AlertCircle, X } from "lucide-react";
import { ChatMessage } from "../lib/types";

interface ChatWorkspaceProps {
  messages: ChatMessage[];
  onSendMessage: (query: string) => Promise<void>;
  isLoading: boolean;
  onSelectCitation: (citation: string) => void;
  onNavigateToTab: (tab: "documents" | "review" | "reports") => void;
  onIngestFile: (filePath: string) => Promise<void>;
  errorMessage: string | null;
  suggestions: string[];
}

export const ChatWorkspace: React.FC<ChatWorkspaceProps> = ({
  messages,
  onSendMessage,
  isLoading,
  onSelectCitation,
  onNavigateToTab: _onNavigateToTab,
  onIngestFile,
  errorMessage,
  suggestions,
}) => {
  const [inputText, setInputText] = useState("");
  const [showAttachModal, setShowAttachModal] = useState(false);
  const [attachPath, setAttachPath] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isLoading]);

  const handleSend = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!inputText.trim() || isLoading) return;
    const query = inputText.trim();
    setInputText("");
    onSendMessage(query);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleAttachSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!attachPath.trim()) return;
    onIngestFile(attachPath.trim());
    setAttachPath("");
    setShowAttachModal(false);
  };

  const handleChipClick = (suggestion: string) => {
    onSendMessage(suggestion);
  };

  const isEmptyThread = messages.length === 0;

  // Format message text with bolding and structured paragraphs
  const renderMessageText = (content: string) => {
    const lines = content.split("\n");
    return lines.map((line, idx) => {
      const trimmed = line.trim();
      if (!trimmed) {
        return <div key={idx} className="message-line-spacer" />;
      }

      // Check if it's a bullet point or numbered item
      const isBullet = trimmed.startsWith("- ") || trimmed.startsWith("• ");
      const isNumbered = /^\d+\.\s/.test(trimmed);

      // Parse **bold** parts
      const parts = line.split(/(\*\*[^*]+\*\*)/g);
      const renderedParts = parts.map((part, pIdx) => {
        if (part.startsWith("**") && part.endsWith("**")) {
          return <strong key={pIdx}>{part.slice(2, -2)}</strong>;
        }
        return part;
      });

      return (
        <div
          key={idx}
          className={`message-line ${isBullet ? "bullet" : isNumbered ? "numbered" : "standard"}`}
        >
          {renderedParts}
        </div>
      );
    });
  };

  return (
    <div className="chat-workspace-root">
      {isEmptyThread ? (
        /* ================= Claude-Style Hero Empty State ================= */
        <div className="chat-empty-hero-container">
          <div className="chat-hero-content">
            <div className="hero-greeting-block">
              <h1 className="hero-greeting-title">SAAR Financial Workspace</h1>
              <p className="hero-greeting-subtitle">
                Private, local-first intelligence operating directly over your encrypted ledger records via Qwen2.5-3B.
              </p>
            </div>

            {/* Centered Floating Prompt Composer */}
            <div className="hero-composer-card">
              <div className="composer-input-row">
                <button
                  type="button"
                  className="composer-attach-btn"
                  onClick={() => setShowAttachModal(true)}
                  title="Ingest invoice, receipt, or bank statement"
                >
                  <Paperclip size={16} />
                </button>
                <textarea
                  ref={textareaRef}
                  className="composer-textarea"
                  placeholder="Ask a question about your financial records, expenses, or invoices..."
                  value={inputText}
                  onChange={(e) => setInputText(e.target.value)}
                  onKeyDown={handleKeyDown}
                  rows={2}
                  disabled={isLoading}
                />
                <button
                  type="button"
                  className={`composer-send-btn ${inputText.trim() ? "ready" : ""}`}
                  onClick={() => handleSend()}
                  disabled={!inputText.trim() || isLoading}
                  title="Send to local model"
                >
                  <Send size={15} />
                </button>
              </div>
              <div className="composer-footer-row">
                <span className="composer-meta-text">
                  Qwen2.5-3B · CPU AVX2 · Zero cloud telemetry · Local SQLite
                </span>
              </div>
            </div>

            {/* Starter questions derived from records in this workspace */}
            {suggestions.length > 0 && (
              <div className="hero-suggestions-grid">
                {suggestions.map((suggestion) => (
                  <button
                    key={suggestion}
                    className="suggestion-chip-btn"
                    onClick={() => handleChipClick(suggestion)}
                  >
                    <span className="chip-text">{suggestion}</span>
                    <span className="chip-arrow">→</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      ) : (
        /* ================= Active Conversation Stream ================= */
        <div className="chat-active-container">
          <div className="chat-stream-scroll">
            <div className="chat-stream-inner">
              {messages.map((msg) => (
                <div key={msg.id} className={`chat-message-row ${msg.sender}`}>
                  {msg.sender === "user" ? (
                    <div className="user-message-bubble">
                      <div className="user-message-text">{msg.text}</div>
                      <span className="message-timestamp">{msg.timestamp}</span>
                    </div>
                  ) : (
                    <div className="assistant-message-entry">
                      {/* Formatted Message Body */}
                      <div className="assistant-content-flow">
                        {renderMessageText(msg.text)}
                      </div>

                      {/* Provenance Source Citations */}
                      {msg.response?.citations && msg.response.citations.length > 0 && (
                        <div className="citations-provenance-bar">
                          <span className="citations-bar-label">Evidence:</span>
                          <div className="citations-pill-group">
                            {msg.response.citations.map((cite, cIdx) => (
                              <button
                                key={cIdx}
                                className="citation-pill"
                                onClick={() => onSelectCitation(cite)}
                                title={`Inspect source evidence for ${cite}`}
                              >
                                <span>[{cite}]</span>
                              </button>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Grounding Verification Strip */}
                      {msg.response?.verificationNote && (
                        <div
                          className={`grounding-verification-strip ${
                            msg.response.grounded ? "verified" : "unverified"
                          }`}
                        >
                          {msg.response.grounded ? (
                            <CheckCircle2 size={12} className="status-icon green" />
                          ) : (
                            <AlertCircle size={12} className="status-icon amber" />
                          )}
                          <span className="verification-text">
                            {msg.response.verificationNote}
                          </span>
                        </div>
                      )}

                      <span className="assistant-timestamp">{msg.timestamp}</span>
                    </div>
                  )}
                </div>
              ))}

              {errorMessage && (
                <div className="chat-error-banner" role="alert">
                  <AlertCircle size={14} />
                  <span>{errorMessage}</span>
                </div>
              )}

              {/* Real-time Thinking Indicator */}
              {isLoading && (
                <div className="chat-message-row assistant">
                  <div className="assistant-message-entry loading">
                    <div className="thinking-pill">
                      <span className="thinking-dot" />
                      <span className="thinking-dot" />
                      <span className="thinking-dot" />
                      <span className="thinking-label">
                        Reasoning over local SQLite ledger via Qwen2.5-3B...
                      </span>
                    </div>
                  </div>
                </div>
              )}

              <div ref={messagesEndRef} />
            </div>
          </div>

          {/* Docked Bottom Composer */}
          <div className="chat-docked-composer-area">
            <div className="docked-composer-card">
              <div className="composer-input-row">
                <button
                  type="button"
                  className="composer-attach-btn"
                  onClick={() => setShowAttachModal(true)}
                  title="Ingest invoice, receipt, or bank statement"
                >
                  <Paperclip size={16} />
                </button>
                <textarea
                  className="composer-textarea"
                  placeholder="Ask a question or enter financial instructions..."
                  value={inputText}
                  onChange={(e) => setInputText(e.target.value)}
                  onKeyDown={handleKeyDown}
                  rows={1}
                  disabled={isLoading}
                />
                <button
                  type="button"
                  className={`composer-send-btn ${inputText.trim() ? "ready" : ""}`}
                  onClick={() => handleSend()}
                  disabled={!inputText.trim() || isLoading}
                  title="Send to local model"
                >
                  <Send size={15} />
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Ingest File Modal */}
      {showAttachModal && (
        <div className="modal-backdrop" onClick={() => setShowAttachModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">Ingest Document into Workspace</h3>
              <button
                className="modal-close-btn"
                onClick={() => setShowAttachModal(false)}
              >
                <X size={15} />
              </button>
            </div>
            <form onSubmit={handleAttachSubmit} className="modal-form">
              <p className="modal-desc">
                Provide an absolute path to a bank statement (CSV), invoice (PDF), or scanned receipt (PNG/JPG).
              </p>
              <input
                type="text"
                className="modal-input"
                placeholder="Absolute path to a CSV, PDF, PNG or JPG file"
                value={attachPath}
                onChange={(e) => setAttachPath(e.target.value)}
                autoFocus
              />
              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() => setShowAttachModal(false)}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn-primary"
                  disabled={!attachPath.trim()}
                >
                  Ingest & Encrypt
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
