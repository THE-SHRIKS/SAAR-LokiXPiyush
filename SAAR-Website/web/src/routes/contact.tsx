import { createFileRoute } from "@tanstack/react-router";
import { ORG_URL, REPO_ISSUES_URL, REPO_URL } from "@/data/site";

export const Route = createFileRoute("/contact")({
  head: () => ({
    meta: [
      { title: "Contact | SAAR" },
      {
        name: "description",
        content: "Find SAAR's official repository, issue tracker, and project organization.",
      },
    ],
  }),
  component: Contact,
});

function Contact() {
  return (
    <>
      <section className="s-page-lead">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Contact / Public project channels</p>
            <h1>Start with the right place for your question.</h1>
          </div>
          <p>
            SAAR is a source project. Its repository and issue tracker are the clearest places to
            inspect the work, report a reproducible defect, or request a change.
          </p>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap">
          <div className="s-contact-row">
            <span className="s-contact-index">01</span>
            <div>
              <h2>Question, defect, or feature request</h2>
              <p>
                Open a GitHub issue. Include your Windows version, the action you took, the result
                you saw, and what you expected.
              </p>
            </div>
            <a href={REPO_ISSUES_URL} target="_blank" rel="noreferrer">
              Open issues ↗
            </a>
          </div>
          <div className="s-contact-row">
            <span className="s-contact-index">02</span>
            <div>
              <h2>Code, setup, or test evidence</h2>
              <p>
                Use the repository for the build guide, source tree, architecture notes, and
                recorded verification.
              </p>
            </div>
            <a href={REPO_URL} target="_blank" rel="noreferrer">
              Read repository ↗
            </a>
          </div>
          <div className="s-contact-row">
            <span className="s-contact-index">03</span>
            <div>
              <h2>The people behind SAAR</h2>
              <p>
                THE SHRIKS organization page is the public starting point for broader project
                information.
              </p>
            </div>
            <a href={ORG_URL} target="_blank" rel="noreferrer">
              View organization ↗
            </a>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Before you post</p>
          </div>
          <div>
            <h2 className="s-story-heading">Keep financial records out of public issues.</h2>
            <p className="s-story-copy">
              GitHub issues are public. Use synthetic or redacted examples when describing a problem
              with invoices, statements, or transaction data. A minimal sequence of steps and an
              error message are usually more useful than a real financial file.
            </p>
          </div>
        </div>
      </section>
    </>
  );
}
