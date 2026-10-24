import { createFileRoute, Link } from "@tanstack/react-router";
import { SourceButton } from "@/components/site-blocks";
import { pageHead } from "@/lib/seo";

const facts = [
  { value: "0", label: "mismatches against a ground-truth ledger in the synthetic corpus test" },
  { value: "3", label: "currencies kept separate, never converted silently" },
  { value: "Exact", label: "money math in integer minor units, never binary floating point" },
  { value: "Local", label: "model and OCR run on your machine; no hosted service in the flow" },
] as const;

export const Route = createFileRoute("/")({
  head: () =>
    pageHead({
      title: "SAAR | Financial clarity with a source",
      description:
        "SAAR is a Windows desktop workspace for financial documents, reconciliation, exact calculations, and source-linked answers.",
      path: "/",
    }),
  component: Home,
});

const workflow = [
  {
    number: "01",
    title: "Bring the records in",
    body: "Add invoices, receipts, and bank statement CSVs to a local workspace. SAAR keeps the original document alongside the fields extracted from it.",
  },
  {
    number: "02",
    title: "Check what was extracted",
    body: "Review the source of a date, amount, vendor, or reference before it becomes part of a calculation. File fingerprints and import status help you track what entered the workspace.",
  },
  {
    number: "03",
    title: "Reconcile with a person in control",
    body: "Rules suggest links between invoices and transactions and flag possible duplicates. Matches remain proposals until someone reviews them.",
  },
  {
    number: "04",
    title: "Ask, calculate, and inspect",
    body: "A local model can help interpret a question. Defined operations calculate verified figures from stored records, while model-written explanation is identified separately.",
  },
  {
    number: "05",
    title: "Report and retain",
    body: "Read summaries by currency, inspect excluded records, export a report, and create an encrypted backup of the workspace.",
  },
] as const;

function EvidenceExample() {
  return (
    <div className="s-evidence" aria-label="How a SAAR answer is traced to its sources">
      <div className="s-evidence-head">
        <span>Illustrative record trail</span>
        <span>Source → operation → answer</span>
      </div>
      <div className="s-evidence-grid">
        <div className="s-evidence-cell">
          <span className="s-evidence-index">01 / Source</span>
          <strong>Original records</strong>
          <p>Statement rows, invoices, and receipts stay available for inspection.</p>
        </div>
        <div className="s-evidence-cell">
          <span className="s-evidence-index">02 / Review</span>
          <strong>Defined calculation</strong>
          <p>Rules select records, separate currencies, and calculate in integer minor units.</p>
        </div>
        <div className="s-evidence-cell s-evidence-result">
          <span className="s-evidence-index">03 / Result</span>
          <strong>A figure with citations</strong>
          <p>The answer names the operation and links back to the records that support it.</p>
        </div>
      </div>
    </div>
  );
}

function Home() {
  return (
    <>
      <section className="s-home-hero">
        <div className="s-wrap">
          <div className="s-home-hero-top">
            <p className="s-kicker">A local workspace for financial evidence</p>
            <p className="s-home-edition">Windows desktop / Source project</p>
          </div>
          <h1 className="s-home-title">
            Know the number.
            <br />
            <span>Know where it came from.</span>
          </h1>
          <div className="s-home-hero-bottom">
            <p className="s-lede">
              SAAR brings documents, bank rows, reconciliation, and questions into one workspace.
              Its useful distinction is simple: a financial answer stays connected to the records
              and calculation behind it.
            </p>
            <div className="s-home-actions">
              <Link className="s-button" to="/download">
                Download for Windows <span aria-hidden="true">↓</span>
              </Link>
              <Link className="s-text-link" to="/product">
                Understand the product →
              </Link>
            </div>
          </div>
        </div>
      </section>

      <section className="s-fact-strip" aria-label="Key facts">
        <dl className="s-wrap s-fact-grid">
          {facts.map((fact) => (
            <div key={fact.label}>
              <dt>{fact.value}</dt>
              <dd>{fact.label}</dd>
            </div>
          ))}
        </dl>
      </section>

      <section className="s-story-section s-story-intro">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">The problem</p>
          </div>
          <div>
            <h2 className="s-story-heading">
              The difficult part is not finding a total. It is trusting it.
            </h2>
            <p className="s-story-copy">
              An invoice says one thing, a bank row says another, and a report gives you a number
              without the path that produced it. Reviewing financial work means moving between those
              pieces and deciding which records belong together.
            </p>
            <p className="s-story-copy">
              SAAR keeps that trail in the same desktop workspace. It stores the original files,
              exposes extracted fields, proposes matches for review, and makes the source of a
              verified figure inspectable.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt" id="evidence">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">A different kind of answer</p>
              <h2 className="s-story-heading">From record to result, in plain sight.</h2>
            </div>
            <p className="s-section-intro">
              An answer is useful when its inputs and calculation can be examined. SAAR keeps those
              pieces connected inside the workspace.
            </p>
          </div>
          <EvidenceExample />
          <p className="s-fineprint">
            Example is illustrative. Actual figures depend on the records in a workspace.
          </p>
          <div className="s-home-proof">
            <div>
              <span>Recorded test result / synthetic ledger</span>
              <strong>
                124,285.28 <small>USD</small>
              </strong>
            </div>
            <div>
              <p>
                In the real-window test, SAAR calculated all-time USD expenses and the result
                matched the independent ledger. The amount is a demo result, not a claim about
                anyone's finances.
              </p>
              <Link className="s-text-link" to="/product" hash="answers">
                Inspect the recorded answer →
              </Link>
            </div>
          </div>
        </div>
      </section>

      <section className="s-story-section" id="workflow">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The work, end to end</p>
              <h2 className="s-story-heading">A deliberate sequence, not a black box.</h2>
            </div>
            <Link className="s-text-link" to="/how-it-works">
              Read the full workflow →
            </Link>
          </div>
          <ol className="s-workflow-list">
            {workflow.map((step) => (
              <li key={step.number} className="s-workflow-item">
                <span className="s-workflow-num">{step.number}</span>
                <h3>{step.title}</h3>
                <p>{step.body}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Where the boundary sits</p>
          </div>
          <div>
            <h2 className="s-story-heading">The model explains. The financial core calculates.</h2>
            <p className="s-story-copy">
              SAAR can run a local Qwen3 4B model when its binary and weights are installed. The
              model helps interpret requests and write explanations. It does not supply a verified
              financial figure. Defined Rust operations calculate those figures from the workspace.
            </p>
            <dl className="s-definition-list">
              <div>
                <dt>Money</dt>
                <dd>Signed integer minor units instead of floating point arithmetic.</dd>
              </div>
              <div>
                <dt>Currencies</dt>
                <dd>Totals stay separate; records in another currency are reported as excluded.</dd>
              </div>
              <div>
                <dt>Evidence</dt>
                <dd>Source records and audit history remain available for review.</dd>
              </div>
              <div>
                <dt>Storage</dt>
                <dd>Each workspace has an encrypted database and document store.</dd>
              </div>
            </dl>
          </div>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Current project status</p>
          </div>
          <div>
            <h2 className="s-story-heading">Explore the source. Understand the limits.</h2>
            <p className="s-story-copy">
              SAAR is available as source code with a build guide and test evidence. The repository
              is not a complete binary distribution: model weights and native sidecars are provided
              separately. The release record still lists installer and disconnected-network checks
              as open, so the site does not present SAAR as a finished installed release.
            </p>
            <div className="s-home-actions">
              <SourceButton>Read the repository</SourceButton>
              <Link className="s-text-link" to="/contact">
                Project channels
              </Link>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
