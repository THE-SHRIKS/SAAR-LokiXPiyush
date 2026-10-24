import { createFileRoute, Link } from "@tanstack/react-router";
import { ORG_URL, REPO_URL } from "@/data/site";

export const Route = createFileRoute("/about")({
  head: () => ({
    meta: [
      { title: "About | SAAR" },
      {
        name: "description",
        content:
          "Why SAAR exists, the principles behind it, what the logo means, the stack it is built on, and the team at THE SHRIKS.",
      },
    ],
  }),
  component: About,
});

const principles = [
  {
    title: "Evidence before fluency",
    body: "A well-written answer is not evidence. Every figure should lead back to the records and the operation that produced it.",
  },
  {
    title: "Exact money",
    body: "Amounts are integers in minor units. Rounding errors are not allowed to hide inside a total.",
  },
  {
    title: "Local by default",
    body: "Financial records are personal. The workflow, storage, and model all run on the machine that owns the records.",
  },
  {
    title: "Honest status",
    body: "A release is ready when its gates pass, not when the screens look finished. Open items stay visible.",
  },
] as const;

const stack = [
  { layer: "Interface", value: "React and TypeScript, with a typed command layer" },
  { layer: "Desktop shell", value: "Tauri 2" },
  { layer: "Financial core", value: "Rust, integer minor-unit arithmetic" },
  { layer: "Storage", value: "SQLCipher per workspace, Argon2id key derivation" },
  { layer: "Local AI", value: "llama.cpp with Qwen3 4B weights" },
  { layer: "Document intake", value: "PDF text layer, CSV mapping, local OCR worker" },
] as const;

function About() {
  return (
    <>
      <section className="s-page-lead s-page-lead-about">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">About / The project</p>
            <h1>A careful tool for a difficult kind of work.</h1>
          </div>
          <p>
            SAAR started from a simple frustration: a total arrives without its trail. It is a small
            desktop project by THE SHRIKS, built to keep the path from document to figure visible,
            and to refuse to show a number it cannot explain.
          </p>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Why it exists</p>
          </div>
          <div>
            <h2 className="s-story-heading">Most finance software hides the middle.</h2>
            <p className="s-story-copy">
              Invoices, bank rows, and receipts disagree in small ways: a date format, a missing
              reference, a payment that covers two bills. The work is in the reconciliation, and
              most tools skip it or bury it.
            </p>
            <p className="s-story-copy">
              SAAR keeps the middle in view. Each record stays linked to its source, each match is a
              proposal until a person accepts it, and each total says which records it used.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">Principles</p>
              <h2 className="s-story-heading">Four rules the code is held to.</h2>
            </div>
          </div>
          <div className="s-principle-grid">
            {principles.map((principle, i) => (
              <article key={principle.title} className="s-principle">
                <span className="s-principle-index">0{i + 1}</span>
                <h3>{principle.title}</h3>
                <p>{principle.body}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Built with</p>
          </div>
          <div>
            <h2 className="s-story-heading">The stack, in plain terms.</h2>
            <dl className="s-stack-list">
              {stack.map((row) => (
                <div key={row.layer}>
                  <dt>{row.layer}</dt>
                  <dd>{row.value}</dd>
                </div>
              ))}
            </dl>
          </div>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">The team</p>
          </div>
          <div>
            <h2 className="s-story-heading">THE SHRIKS</h2>
            <p className="s-story-copy">
              SAAR is developed by THE SHRIKS. The organization page and the repository are the
              public record of who builds it and what has changed.
            </p>
            <div className="s-home-actions">
              <a className="s-button" href={ORG_URL} target="_blank" rel="noreferrer">
                Organization on GitHub <span aria-hidden="true">↗</span>
              </a>
              <a className="s-text-link" href={REPO_URL} target="_blank" rel="noreferrer">
                Source repository ↗
              </a>
            </div>
            <p className="s-fineprint">
              Questions about the project? See the <Link to="/faq">FAQ</Link> or the{" "}
              <Link to="/contact">contact page</Link>.
            </p>
          </div>
        </div>
      </section>
    </>
  );
}
