import { createFileRoute, Link } from "@tanstack/react-router";
import { SourceButton } from "@/components/site-blocks";

export const Route = createFileRoute("/solutions")({
  head: () => ({
    meta: [
      { title: "Use cases | SAAR" },
      {
        name: "description",
        content:
          "See where SAAR helps with financial document review, reconciliation, evidence-linked answers, and scoped reports.",
      },
    ],
  }),
  component: Solutions,
});

const situations = [
  {
    id: "intake",
    n: "01",
    eyebrow: "Document intake",
    title: "A file arrives with a number you need to trust.",
    question: "Did the system read the amount, currency, date, and vendor correctly?",
    input: "An invoice, receipt, or bank statement enters the workspace.",
    work: "SAAR stores the original, fingerprints it, extracts fields, and shows import status. A reviewer compares those fields with the source document.",
    decision: "Correct a field or hold a failed import before it enters later work.",
    output: "A record whose values can be traced back to the file.",
  },
  {
    id: "matching",
    n: "02",
    eyebrow: "Payment review",
    title: "An invoice looks paid, but the bank row is not conclusive.",
    question: "Is this the right transaction, a duplicate, or still unmatched?",
    input: "An invoice and one or more possible transaction rows.",
    work: "Deterministic rules compare amount, date, vendor, and reference. SAAR surfaces candidate links and anomalies without silently deciding for the reviewer.",
    decision: "Approve, reject, or correct the proposed relationship.",
    output: "An explicit link with a review history.",
  },
  {
    id: "answer",
    n: "03",
    eyebrow: "A question about spending",
    title: "Someone asks for a figure that may shape a decision.",
    question: "Which rows and which currency produced that amount?",
    input: "A period, currency, vendor, or specific invoice question.",
    work: "A local model may interpret the request. Defined Rust operations select stored records and calculate verified amounts in integer minor units.",
    decision: "Inspect the cited records and the calculation scope before relying on the result.",
    output: "A figure with its basis, rather than a free-form guess.",
  },
  {
    id: "report",
    n: "04",
    eyebrow: "Reporting and handoff",
    title: "A summary is about to leave the workspace.",
    question: "What does this total include, and what was left out?",
    input: "Bank rows, imported invoices, and an intended report period.",
    work: "Reports show money movement and invoice totals by currency. Other-currency records are stated as exclusions rather than converted without a rate.",
    decision: "Check the scope, resolve review items, and export PDF or CSV when ready.",
    output: "A shareable report with a stated boundary and a recoverable source workspace.",
  },
] as const;

function Solutions() {
  return (
    <>
      <section className="s-page-lead s-page-lead-solutions">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Use cases / Where SAAR helps</p>
            <h1>Four places to ask, “show me the source.”</h1>
          </div>
          <p>
            Financial work often breaks at handoffs. A file becomes fields. Fields become a match.
            Matches become a figure. A figure becomes a report. SAAR keeps those transitions open
            for review.
          </p>
        </div>
        <div className="s-wrap s-solution-ribbon" aria-label="Four review moments">
          <span>Receive</span>
          <span>Compare</span>
          <span>Calculate</span>
          <span>Explain</span>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The review moments</p>
              <h2 className="s-story-heading">A useful workflow is specific about the decision.</h2>
            </div>
            <p className="s-section-intro">
              Each situation below distinguishes what SAAR prepares from what a person still needs
              to judge.
            </p>
          </div>
          <div className="s-situations">
            {situations.map((item) => (
              <article className="s-situation" id={item.id} key={item.id}>
                <div className="s-situation-side">
                  <span>{item.n}</span>
                  <strong>{item.eyebrow}</strong>
                </div>
                <div className="s-situation-main">
                  <h3>{item.title}</h3>
                  <p className="s-situation-question">{item.question}</p>
                  <div className="s-situation-grid">
                    <div>
                      <span>What enters</span>
                      <p>{item.input}</p>
                    </div>
                    <div>
                      <span>What SAAR does</span>
                      <p>{item.work}</p>
                    </div>
                    <div>
                      <span>What you decide</span>
                      <p>{item.decision}</p>
                    </div>
                    <div>
                      <span>What remains</span>
                      <p>{item.output}</p>
                    </div>
                  </div>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">One record, several states</p>
          </div>
          <div>
            <h2 className="s-story-heading">A status should tell you what needs attention.</h2>
            <p className="s-story-copy">
              The interface exposes an import that failed, a proposed match that needs review, or a
              currency that was excluded. Those are work items, not decorative warning colors.
            </p>
            <div className="s-state-table" role="table" aria-label="Illustrative review states">
              <div role="row">
                <span role="columnheader">Record</span>
                <span role="columnheader">State</span>
                <span role="columnheader">Next action</span>
              </div>
              <div role="row">
                <span role="cell">Corrupted PDF</span>
                <strong role="cell">Failed import</strong>
                <span role="cell">Inspect the error and replace the file.</span>
              </div>
              <div role="row">
                <span role="cell">Invoice + bank row</span>
                <strong role="cell">Proposed link</strong>
                <span role="cell">Approve, reject, or correct it.</span>
              </div>
              <div role="row">
                <span role="cell">Other-currency row</span>
                <strong role="cell">Excluded from total</strong>
                <span role="cell">Read in its own currency.</span>
              </div>
            </div>
            <p className="s-fineprint">
              This is a schematic of states supported by the project, not a screenshot or live data.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Know the edges</p>
          </div>
          <div>
            <h2 className="s-story-heading">What SAAR does not decide for you.</h2>
            <p className="s-story-copy">
              A trustworthy tool has visible limits. The current financial correctness report
              records three boundaries that matter in real use:
            </p>
            <ol className="s-limits-list">
              <li>
                <span>01</span>
                <div>
                  <strong>No currency conversion</strong>
                  <p>
                    Other-currency rows are reported separately or excluded from a selected
                    base-currency total.
                  </p>
                </div>
              </li>
              <li>
                <span>02</span>
                <div>
                  <strong>Partial payments need manual review</strong>
                  <p>They are not linked as if they were exact paid-in-full matches.</p>
                </div>
              </li>
              <li>
                <span>03</span>
                <div>
                  <strong>Real-world scans are not fully covered by the test corpus</strong>
                  <p>
                    The recorded OCR tests include rendered text with blur, noise, and rotation, not
                    every photograph or handwritten note.
                  </p>
                </div>
              </li>
            </ol>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-closer">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Choose the next view</p>
          </div>
          <div>
            <h2 className="s-story-heading">See the product, then follow the record.</h2>
            <div className="s-home-actions">
              <Link className="s-button" to="/product">
                Inside the workspace →
              </Link>
              <Link className="s-text-link" to="/how-it-works">
                How the system works
              </Link>
              <SourceButton>Read the source</SourceButton>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
