import { createFileRoute, Link } from "@tanstack/react-router";
import { REPO_ISSUES_URL } from "@/data/site";

export const Route = createFileRoute("/faq")({
  head: () => ({
    meta: [
      { title: "FAQ | SAAR" },
      {
        name: "description",
        content:
          "Answers about SAAR's data handling, supported files, verified figures, storage location, hardware, licensing, and current readiness.",
      },
    ],
  }),
  component: Faq,
});

const groups = [
  {
    label: "The product",
    items: [
      {
        q: "What is SAAR?",
        a: "A Windows desktop workspace for financial documents. It imports invoices, receipts, and bank statement CSVs, extracts their fields, proposes matches between records, and calculates totals from the stored records with exact arithmetic.",
      },
      {
        q: "Which files can I import?",
        a: "Digital PDFs with a text layer, bank statement CSVs, and image-based documents such as scanned receipts. Scanned images are read by a local OCR worker. Extracted fields keep a link to the source document so you can check them.",
      },
      {
        q: "Does the AI calculate my totals?",
        a: "No. A local model helps interpret a question and writes the explanation. Totals and other verified figures come from defined operations in the Rust core. Answers distinguish computed figures from generated text.",
      },
      {
        q: "Why does a record appear as excluded?",
        a: "SAAR keeps totals separate by currency. A record in a currency that is not part of the requested total is reported as excluded instead of being converted silently. Currency conversion is not implemented.",
      },
    ],
  },
  {
    label: "Data and privacy",
    items: [
      {
        q: "Is my data sent anywhere?",
        a: "The workflow is designed to keep records on your machine. The webview's Content Security Policy blocks outbound connections, and the AI and OCR runtimes run locally. The egress audit in the repository documents how this was tested.",
      },
      {
        q: "Where are workspaces stored?",
        a: "Each workspace has its own encrypted database and document store. The location can be changed with the SAAR_DATA_DIR setting, which is useful for portable or isolated runs.",
      },
      {
        q: "How are backups protected?",
        a: "Backups are authenticated archives of the workspace. Keys are derived from a passphrase with Argon2id. Restore is covered by tests, but the full restore success path is still open, so keep an independent copy of important originals.",
      },
    ],
  },
  {
    label: "Using it",
    items: [
      {
        q: "Can I use SAAR with real financial records now?",
        a: "Not yet as a finished tool. The release record is marked not ready, and the installed application has not been verified end to end. Use synthetic or copied test records until the open items on the security page are closed.",
      },
      {
        q: "What hardware do I need?",
        a: "The local model uses GPU offload when available and falls back to the CPU. Performance on a target GPU has not been measured yet, so treat response times as unknown until you test them on your own machine.",
      },
      {
        q: "Which platform is supported?",
        a: "Windows is the documented target. The app is built with Tauri 2, but the Windows build guide is the only path the project currently describes in detail.",
      },
    ],
  },
  {
    label: "Project",
    items: [
      {
        q: "Is SAAR open source?",
        a: "The source is published on GitHub. The repository does not yet include a root LICENSE file, so no project-wide open source license is asserted. Check the repository before reusing the code.",
      },
      {
        q: "How do I report a problem or suggest a feature?",
        a: "Open an issue in the repository. Keep financial records, account numbers, and personal data out of public issues. Describe the defect with synthetic data instead.",
      },
    ],
  },
] as const;

function Faq() {
  return (
    <>
      <section className="s-page-lead s-page-lead-faq">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">FAQ / Questions</p>
            <h1>Straight answers, including the uncomfortable ones.</h1>
          </div>
          <p>
            What SAAR does, what it sends and where it keeps data, and what is still unfinished. If
            your question is not here, the <Link to="/contact">contact page</Link> lists the right
            channel.
          </p>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-faq-layout">
          {groups.map((group) => (
            <div key={group.label} className="s-faq-group">
              <h2 className="s-faq-group-label">
                <span className="s-section-label">{group.label}</span>
              </h2>
              <div className="s-faq-list">
                {group.items.map((item) => (
                  <details key={item.q} className="s-faq-item">
                    <summary>
                      <span>{item.q}</span>
                      <span className="s-faq-toggle" aria-hidden="true">
                        +
                      </span>
                    </summary>
                    <p>{item.a}</p>
                  </details>
                ))}
              </div>
            </div>
          ))}

          <div className="s-faq-footer">
            <p>Still stuck? Read the source or open an issue without real records.</p>
            <a className="s-text-link" href={REPO_ISSUES_URL} target="_blank" rel="noreferrer">
              Open the issue tracker ↗
            </a>
          </div>
        </div>
      </section>
    </>
  );
}
