import { createFileRoute, Link } from "@tanstack/react-router";
import { Screenshot, SourceButton } from "@/components/site-blocks";
import {
  CurrencyBoundary,
  EvidenceExplorer,
  ReconciliationSpecimen,
} from "@/components/product-demos";
import chat from "@/assets/light_02_chat_seed_report.asset.json";
import { pageHead } from "@/lib/seo";

export const Route = createFileRoute("/product")({
  head: () =>
    pageHead({
      title: "Product | SAAR",
      description:
        "Explore SAAR's document workspace, reconciliation review, verified answers, reports, and local storage.",
      path: "/product",
    }),
  component: Product,
});

function Product() {
  return (
    <>
      <section className="s-page-lead s-page-lead-product">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Product / Inside the workspace</p>
            <h1>Financial work with its evidence attached.</h1>
          </div>
          <p>
            SAAR joins the tasks that usually live in separate folders and spreadsheets: receiving
            files, checking extraction, comparing payments, calculating figures, and inspecting the
            records behind an answer.
          </p>
        </div>
        <div className="s-wrap s-page-index" aria-label="On this page">
          <a href="#answers">Verified answers</a>
          <a href="#documents">Documents</a>
          <a href="#reconciliation">Reconciliation</a>
          <a href="#reports">Reports</a>
          <a href="#workspace">Workspace</a>
        </div>
      </section>

      <section className="s-story-section s-product-feature" id="answers">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">01 / Ask and inspect</p>
              <h2 className="s-story-heading">
                The answer is a starting point. The evidence is the point.
              </h2>
            </div>
            <p className="s-section-intro">
              Choose one of three questions from the project's recorded synthetic GUI test. The
              amount, scope, operation, and source change together.
            </p>
          </div>
          <EvidenceExplorer />
          <div className="s-product-afterword">
            <p>
              A local model can interpret a question and write an explanation when its runtime is
              installed. Verified financial figures come from defined operations in the Rust core.
              Model-written prose is labelled separately so the two are not confused.
            </p>
            <Link className="s-text-link" to="/how-it-works">
              Trace the full calculation path →
            </Link>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt" id="documents">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">02 / Receive and inspect</p>
              <h2 className="s-story-heading">A document is more than extracted text.</h2>
            </div>
            <p className="s-section-intro">
              SAAR keeps the original, records a fingerprint, and exposes the fields it read. A
              reviewer can check a value at the source before it affects a match or report.
            </p>
          </div>
          <div className="s-register">
            <div className="s-register-head">
              <span>Recorded import run</span>
              <span>20 supported files</span>
            </div>
            <div className="s-register-row">
              <strong>15</strong>
              <span>PDF invoices</span>
              <p>
                Digital documents whose vendor, number, date, currency, and total can be reviewed.
              </p>
            </div>
            <div className="s-register-row">
              <strong>04</strong>
              <span>Scanned receipts</span>
              <p>Images processed through the separately supplied local OCR worker.</p>
            </div>
            <div className="s-register-row">
              <strong>01</strong>
              <span>Bank statement CSV</span>
              <p>Rows enter the workspace ledger for calculation and reconciliation.</p>
            </div>
            <div className="s-register-foot">
              In the same test, a byte-identical re-import was reported as a duplicate and a
              corrupted PDF appeared as a failed record.
            </div>
          </div>
        </div>
      </section>

      <section className="s-story-section" id="reconciliation">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">03 / Compare and decide</p>
              <h2 className="s-story-heading">A likely match still needs a reviewer.</h2>
            </div>
            <p className="s-section-intro">
              Rules compare amounts, dates, vendors, and references. The result is a candidate link
              or anomaly for inspection, with approval, rejection, and correction kept in the audit
              history.
            </p>
          </div>
          <ReconciliationSpecimen />
          <div className="s-product-metrics">
            <div>
              <strong>13</strong>
              <span>proposed links in the recorded GUI run</span>
            </div>
            <div>
              <strong>30</strong>
              <span>anomalies surfaced for review</span>
            </div>
            <p>
              These counts describe one synthetic test workspace. They are not a benchmark or a
              claim about a user's records.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt" id="reports">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">04 / Summarize and export</p>
              <h2 className="s-story-heading">
                A report has a currency, a period, and a boundary.
              </h2>
            </div>
            <p className="s-section-intro">
              The report view separates money in, money out, and imported invoice totals. It names
              the rows excluded from a selected currency instead of hiding them inside a grand
              total.
            </p>
          </div>
          <CurrencyBoundary />
          <div className="s-product-afterword">
            <p>
              The recorded end-to-end run produced a CSV with one row for each of 72 ledger
              transactions and a valid PDF export. Exporting the summary does not replace keeping
              the underlying workspace for later review.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section" id="workspace">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">05 / Keep control</p>
          </div>
          <div>
            <h2 className="s-story-heading">A local workspace with a recovery path.</h2>
            <p className="s-story-copy">
              Each workspace uses a SQLCipher database and encrypted document storage. SAAR derives
              keys from a passphrase with Argon2id, records review activity, and can create an
              authenticated encrypted backup archive.
            </p>
            <div className="s-fact-grid">
              <div>
                <span>Storage</span>
                <strong>Encrypted per workspace</strong>
                <p>Database and document files stay together.</p>
              </div>
              <div>
                <span>Arithmetic</span>
                <strong>Integer minor units</strong>
                <p>No floating point money totals.</p>
              </div>
              <div>
                <span>Recovery</span>
                <strong>Authenticated backup</strong>
                <p>Wrong passphrases and tampered archives are rejected in the recorded test.</p>
              </div>
            </div>
            <p className="s-story-copy s-product-limit">
              The source repository is not an installed release. Installer and disconnected-network
              verification remain open, and WebView2's own traffic is a documented concern. The
              project should not be described as fully air gapped today.
            </p>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt s-product-view">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The current interface</p>
              <h2 className="s-story-heading">The answer and its record, side by side.</h2>
            </div>
            <p className="s-section-intro">
              A single screen from the light mode Windows demo shows how a citation opens the
              supporting transaction beside a verified answer.
            </p>
          </div>
          <Screenshot
            src={chat.url}
            alt="SAAR desktop conversation showing a verified expense answer and the related transaction"
            caption="Current Windows desktop build with synthetic demo records."
          />
        </div>
      </section>

      <section className="s-story-section s-story-closer">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Go deeper</p>
          </div>
          <div>
            <h2 className="s-story-heading">Review the workflow and the implementation.</h2>
            <p className="s-story-copy">
              The repository includes the build guide, source tree, and test evidence behind the
              examples on this page.
            </p>
            <div className="s-home-actions">
              <Link className="s-button" to="/how-it-works">
                How SAAR works →
              </Link>
              <SourceButton>Open source</SourceButton>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
