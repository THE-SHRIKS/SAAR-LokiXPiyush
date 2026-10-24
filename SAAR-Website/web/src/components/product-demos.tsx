import { useState } from "react";

const answers = [
  {
    id: "all",
    prompt: "What are my total expenses?",
    amount: "124,285.28",
    currency: "USD",
    scope: "All recorded dates · USD expense rows",
    operation: "Sum the selected expense rows in integer minor units.",
    evidence: "The GUI test compared this verified result with the synthetic ledger.",
    source: "Stored statement transactions",
  },
  {
    id: "april",
    prompt: "How much did I spend in April 2025?",
    amount: "79,922.81",
    currency: "USD",
    scope: "April 2025 · USD expense rows",
    operation: "Filter by month and currency, then sum the matching expense rows.",
    evidence: "The GUI test recorded the monthly answer as verified against the ledger.",
    source: "April 2025 statement transactions",
  },
  {
    id: "invoice",
    prompt: "What is the total of invoice RPW-2025-0004?",
    amount: "9,123.76",
    currency: "INR",
    scope: "One invoice · RPW-2025-0004",
    operation: "Read the stored invoice total for the identified record.",
    evidence: "The GUI test confirmed the amount and a source citation.",
    source: "Cited invoice record",
  },
] as const;

export function EvidenceExplorer() {
  const [selected, setSelected] = useState<(typeof answers)[number]["id"]>("all");
  const answer = answers.find((item) => item.id === selected) ?? answers[0];

  return (
    <div className="s-inspector">
      <div className="s-inspector-toolbar">
        <span>Verified answer inspector</span>
        <span>Recorded synthetic test corpus</span>
      </div>
      <div className="s-inspector-layout">
        <div className="s-inspector-queries">
          <p className="s-inspector-label">Select a tested question</p>
          {answers.map((item, index) => (
            <button
              key={item.id}
              type="button"
              className={selected === item.id ? "s-inspector-query is-active" : "s-inspector-query"}
              aria-pressed={selected === item.id}
              onClick={() => setSelected(item.id)}
            >
              <span>{String(index + 1).padStart(2, "0")}</span>
              <strong>{item.prompt}</strong>
            </button>
          ))}
        </div>
        <div className="s-inspector-result" aria-live="polite">
          <div className="s-inspector-result-top">
            <span>Answer / verified</span>
            <span>{answer.scope}</span>
          </div>
          <p className="s-inspector-amount">
            <strong>{answer.amount}</strong>
            <span>{answer.currency}</span>
          </p>
          <div className="s-inspector-detail">
            <div>
              <span>Operation</span>
              <p>{answer.operation}</p>
            </div>
            <div>
              <span>Source</span>
              <p>{answer.source}</p>
            </div>
            <div>
              <span>Test evidence</span>
              <p>{answer.evidence}</p>
            </div>
          </div>
        </div>
      </div>
      <p className="s-inspector-foot">
        These are recorded test results, not a live query into the user's data.
      </p>
    </div>
  );
}

export function ReconciliationSpecimen() {
  return (
    <div className="s-review-specimen" aria-label="How a reconciliation candidate is reviewed">
      <div className="s-specimen-top">
        <span>Reconciliation logic / illustration</span>
        <span>Human decision required</span>
      </div>
      <div className="s-review-pair">
        <div>
          <span className="s-specimen-tag">Record A</span>
          <strong>Invoice</strong>
          <p>Amount, currency, invoice number, vendor, issue date</p>
        </div>
        <div className="s-review-connector" aria-hidden="true">
          <span>?</span>
        </div>
        <div>
          <span className="s-specimen-tag">Record B</span>
          <strong>Bank transaction</strong>
          <p>Amount, currency, reference, description, posting date</p>
        </div>
      </div>
      <div className="s-review-decision">
        <div>
          <span>Engine</span>
          <strong>Proposes a link</strong>
          <p>Rules compare the records and surface anomalies.</p>
        </div>
        <div>
          <span>Reviewer</span>
          <strong>Approves, rejects, or corrects</strong>
          <p>The result and later corrections remain auditable.</p>
        </div>
      </div>
    </div>
  );
}

export function CurrencyBoundary() {
  return (
    <div className="s-currency-specimen" aria-label="Currency separation in SAAR reports">
      <div className="s-specimen-top">
        <span>Report boundary / illustration</span>
        <span>No automatic conversion</span>
      </div>
      <div className="s-currency-rows">
        <div>
          <span>USD ledger</span>
          <strong>USD total</strong>
          <small>Calculated only from USD rows</small>
        </div>
        <div>
          <span>INR ledger</span>
          <strong>INR total</strong>
          <small>Reported separately</small>
        </div>
      </div>
      <p>
        Other-currency records are named as excluded from a selected base-currency summary. SAAR
        does not silently merge them into one figure.
      </p>
    </div>
  );
}

export function SystemMap() {
  return (
    <div className="s-system-map" aria-label="SAAR system architecture">
      <div className="s-system-top">
        <span>Desktop boundary</span>
        <span>Local supporting runtimes</span>
      </div>
      <div className="s-system-grid">
        <div className="s-system-stage">
          <span>01 / Interface</span>
          <strong>React + TypeScript</strong>
          <p>Documents, review, conversations, reports, and vault.</p>
        </div>
        <div className="s-system-stage">
          <span>02 / Bridge</span>
          <strong>Tauri commands</strong>
          <p>Typed calls between the web interface and native operations.</p>
        </div>
        <div className="s-system-stage">
          <span>03 / Core</span>
          <strong>Rust</strong>
          <p>Financial arithmetic, reconciliation, ingestion coordination.</p>
        </div>
        <div className="s-system-stage">
          <span>04 / Storage</span>
          <strong>SQLCipher + files</strong>
          <p>Encrypted database and document store per workspace.</p>
        </div>
      </div>
      <div className="s-system-support">
        <span>Optional local workers</span>
        <strong>llama.cpp / Qwen3 4B</strong>
        <strong>OCR worker</strong>
      </div>
    </div>
  );
}
