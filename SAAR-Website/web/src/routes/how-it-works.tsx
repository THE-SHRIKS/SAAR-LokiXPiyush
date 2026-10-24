import { createFileRoute, Link } from "@tanstack/react-router";
import { SourceButton } from "@/components/site-blocks";
import { SystemMap } from "@/components/product-demos";
import { pageHead } from "@/lib/seo";

export const Route = createFileRoute("/how-it-works")({
  head: () =>
    pageHead({
      title: "How it works | SAAR",
      description:
        "Follow a financial record through SAAR's workspace, import, review, calculation, reporting, and backup.",
      path: "/how-it-works",
    }),
  component: HowItWorks,
});

const stages = [
  {
    n: "01",
    title: "Open a workspace",
    narrative:
      "A workspace has its own encrypted database and document store. SAAR derives keys from a passphrase, so the records and review history stay together under one local workspace identity.",
    input: "Workspace name and passphrase",
    output: "An encrypted place for documents, ledger rows, and decisions",
    check: "The recorded disk check found no plaintext SQLite header.",
  },
  {
    n: "02",
    title: "Bring in the originals",
    narrative:
      "The user imports digital PDFs, scanned receipts, or statement CSVs. The app fingerprints each file, records status, and retains the original. OCR runs locally when its separately supplied worker is available.",
    input: "Invoice, receipt, or statement",
    output: "Stored file, extracted fields, and source reference",
    check: "The recorded run rejected a corrupted PDF and identified a byte-identical re-import.",
  },
  {
    n: "03",
    title: "Inspect and reconcile",
    narrative:
      "The reviewer compares captured fields with the source. Reconciliation rules then compare amounts, dates, vendors, and references. They propose links and flag anomalies; approval remains a separate human step.",
    input: "Extracted fields and bank transactions",
    output: "Candidate links, anomalies, and recorded review decisions",
    check: "The GUI run produced 13 proposed links and 30 anomalies in its synthetic workspace.",
  },
  {
    n: "04",
    title: "Calculate a verified figure",
    narrative:
      "A local model may interpret a question. It is not allowed to author the verified amount. Defined Rust operations select ledger records, apply the period and currency boundary, and return a result with source references.",
    input: "Question plus stored records",
    output: "Calculated figure, scope, and citations",
    check:
      "The GUI run checked all-time USD expenses, April USD expenses, and an INR invoice against its ledger.",
  },
  {
    n: "05",
    title: "Report and preserve",
    narrative:
      "Reports keep money movement, invoice totals, and excluded currencies legible. PDF and CSV exports are outputs of the workspace. An authenticated encrypted archive preserves the underlying material for recovery.",
    input: "Reviewed workspace data",
    output: "Scoped report and encrypted backup",
    check: "The recorded run exported 72 CSV data rows and rejected tampered backup archives.",
  },
] as const;

function HowItWorks() {
  return (
    <>
      <section className="s-page-lead s-page-lead-how">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">How it works / From file to figure</p>
            <h1>Nothing important should disappear between steps.</h1>
          </div>
          <p>
            This is the path a financial record takes through SAAR. Each stage has an input, an
            output, and a point where the result can be checked.
          </p>
        </div>
        <div className="s-wrap s-how-rail" aria-label="Record journey">
          <span>01 Workspace</span>
          <span>02 Import</span>
          <span>03 Review</span>
          <span>04 Calculate</span>
          <span>05 Preserve</span>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The record journey</p>
              <h2 className="s-story-heading">Follow what changes, and what stays traceable.</h2>
            </div>
            <p className="s-section-intro">
              The description is based on the current source and recorded command-layer and GUI
              tests. It does not imply that an installed release has passed all offline checks.
            </p>
          </div>
          <div className="s-stage-list">
            {stages.map((stage) => (
              <article className="s-stage" key={stage.n}>
                <div className="s-stage-number">{stage.n}</div>
                <div className="s-stage-body">
                  <h3>{stage.title}</h3>
                  <p>{stage.narrative}</p>
                  <div className="s-stage-io">
                    <div>
                      <span>Input</span>
                      <strong>{stage.input}</strong>
                    </div>
                    <div>
                      <span>Output</span>
                      <strong>{stage.output}</strong>
                    </div>
                  </div>
                  <p className="s-stage-check">
                    <strong>Recorded check</strong> {stage.check}
                  </p>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The system underneath</p>
              <h2 className="s-story-heading">
                The interface asks. The native core does the work.
              </h2>
            </div>
            <p className="s-section-intro">
              The web interface is the surface of a local desktop app. Tauri carries typed
              operations to Rust, which owns the financial rules and reads the encrypted stores.
            </p>
          </div>
          <SystemMap />
          <div className="s-product-afterword">
            <p>
              Qwen3 4B through llama.cpp and the OCR worker are local supporting runtimes. Their
              binaries and model files are not included in the source repository. Core workspace
              functions can start without the model, but model-assisted features are unavailable.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">When a step cannot complete</p>
          </div>
          <div>
            <h2 className="s-story-heading">Failure should remain visible.</h2>
            <p className="s-story-copy">
              A rejected input is more useful than a confident but unsupported result. The recorded
              end-to-end runs tested several paths where SAAR must show the limit.
            </p>
            <div className="s-failure-ledger">
              <div>
                <span>Damaged document</span>
                <strong>Import marked failed</strong>
                <p>
                  The file remains a visible failed record instead of sitting in processing
                  indefinitely.
                </p>
              </div>
              <div>
                <span>Missing invoice</span>
                <strong>Reported absent</strong>
                <p>A question about a nonexistent invoice does not produce a fabricated total.</p>
              </div>
              <div>
                <span>Model-written text</span>
                <strong>Not verified</strong>
                <p>
                  Free-form prose is labelled separately from figures calculated by the financial
                  core.
                </p>
              </div>
              <div>
                <span>Backup tampering</span>
                <strong>Restore rejected</strong>
                <p>The authenticated archive check blocks modified backup content.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">The release boundary</p>
          </div>
          <div>
            <h2 className="s-story-heading">
              Source-tested is not the same as installed and offline-tested.
            </h2>
            <p className="s-story-copy">
              The source project has unit, integration, command-layer, and real-window evidence. The
              release record still calls for an installed Windows package test and a
              disconnected-network run. WebView2's own outbound connections remain a documented
              issue. This is why the website describes SAAR as a local-first source project rather
              than a finished air-gapped product.
            </p>
            <div className="s-home-actions">
              <SourceButton>Read the build guide</SourceButton>
              <Link className="s-text-link" to="/product">
                Explore the workspace
              </Link>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
