import { createFileRoute, Link } from "@tanstack/react-router";
import { REPO_URL } from "@/data/site";

export const Route = createFileRoute("/security")({
  head: () => ({
    meta: [
      { title: "Security | SAAR" },
      {
        name: "description",
        content:
          "How SAAR protects financial records: per-workspace encryption, local AI, a network-blocked webview, and the known limits that remain open.",
      },
    ],
  }),
  component: Security,
});

const layers = [
  {
    index: "01",
    title: "Storage",
    points: [
      "Each workspace has its own SQLCipher database.",
      "Passphrases derive keys with Argon2id.",
      "Original documents are kept in an encrypted document store.",
      "Backups are authenticated archives that restore into a workspace.",
    ],
  },
  {
    index: "02",
    title: "Network",
    points: [
      "The desktop webview's Content Security Policy sets connect-src to 'none'.",
      "An egress audit records the denial of outbound sockets in a Linux test run.",
      "The normal workflow does not call a hosted AI or OCR service.",
    ],
  },
  {
    index: "03",
    title: "Local intelligence",
    points: [
      "Qwen3 4B runs through llama.cpp on your machine, with GPU offload and a CPU fallback.",
      "The model interprets questions and writes explanations.",
      "It does not produce a verified financial figure; defined Rust operations do.",
    ],
  },
  {
    index: "04",
    title: "Exact arithmetic",
    points: [
      "Amounts are signed integers in minor units, never binary floating point.",
      "Totals are separated by currency, and other-currency records are reported as excluded.",
      "Possible duplicate payments are flagged for review.",
    ],
  },
] as const;

const limits = [
  {
    title: "Temporary decrypted files",
    body: "Some decrypted working files are written to the system temporary folder. Moving them into the encrypted workspace is an open item.",
  },
  {
    title: "Restore path",
    body: "Backup creation is covered by tests. The full restore success path has not yet been re-verified end to end.",
  },
  {
    title: "Installed application",
    body: "The documented workflow has not yet been run against an installed Windows build on a disconnected machine.",
  },
  {
    title: "Model-written text",
    body: "Chat explanations are generated text. A figure is labelled as verified only when a defined operation computed it.",
  },
] as const;

const evidence = [
  { label: "Network egress denial audit", path: "Docs/AUDIT/0001-network-egress-denial-audit.md" },
  {
    label: "Threat model and fixture registry",
    path: "Docs/ADR/0002-threat-model-and-fixture-registry.md",
  },
  { label: "Security and offline verification", path: "Docs/SECURITY_AND_OFFLINE_VERIFICATION.md" },
  { label: "Release readiness gates", path: "Docs/RELEASE_READINESS.md" },
] as const;

function Security() {
  return (
    <>
      <section className="s-page-lead s-page-lead-security">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Security / Data handling</p>
            <h1>Records stay on the machine that holds them.</h1>
          </div>
          <p>
            SAAR is built around one constraint: financial documents should not need a cloud service
            to be read, calculated, or backed up. This page describes what the design does, where
            its boundaries are, and which checks remain open.
          </p>
        </div>
        <div className="s-wrap s-page-index" aria-label="On this page">
          <a href="#layers">Four layers</a>
          <a href="#diagram">Data path</a>
          <a href="#limits">Known limits</a>
          <a href="#evidence">Evidence</a>
        </div>
      </section>

      <section className="s-story-section" id="layers">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">The design</p>
              <h2 className="s-story-heading">Four layers, each with one job.</h2>
            </div>
            <p className="s-section-intro">
              Storage, network, intelligence, and arithmetic are separate concerns. Each can be
              inspected on its own.
            </p>
          </div>
          <div className="s-layer-grid">
            {layers.map((layer) => (
              <article key={layer.index} className="s-layer">
                <span className="s-layer-index">{layer.index}</span>
                <h3>{layer.title}</h3>
                <ul>
                  {layer.points.map((point) => (
                    <li key={point}>{point}</li>
                  ))}
                </ul>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt" id="diagram">
        <div className="s-wrap">
          <div className="s-section-heading-row">
            <div>
              <p className="s-section-label">Data path</p>
              <h2 className="s-story-heading">Every path ends inside the machine.</h2>
            </div>
            <p className="s-section-intro">
              The interface sends typed commands to a Rust core. The core reads and writes the
              workspace and calls only local helpers.
            </p>
          </div>
          <figure className="s-datapath">
            <svg viewBox="0 0 900 300" role="img" aria-labelledby="datapath-title">
              <title id="datapath-title">
                The React interface sends typed commands to the Rust core, which uses the encrypted
                database, the encrypted document store, the local llama.cpp runtime, and the local
                OCR worker. No path leaves the machine.
              </title>
              <rect className="dp-boundary" x="8" y="8" width="884" height="284" rx="14" />
              <text className="dp-caption" x="28" y="34">
                THIS MACHINE
              </text>
              <rect
                className="dp-node dp-node-strong"
                x="30"
                y="120"
                width="170"
                height="60"
                rx="8"
              />
              <text className="dp-label" x="115" y="146">
                React interface
              </text>
              <text className="dp-sub" x="115" y="165">
                typed command calls
              </text>
              <rect
                className="dp-node dp-node-strong"
                x="300"
                y="120"
                width="170"
                height="60"
                rx="8"
              />
              <text className="dp-label" x="385" y="146">
                Tauri command bridge
              </text>
              <text className="dp-sub" x="385" y="165">
                Rust financial core
              </text>
              <rect className="dp-node" x="590" y="36" width="280" height="54" rx="8" />
              <text className="dp-label" x="730" y="59">
                SQLCipher database
              </text>
              <text className="dp-sub" x="730" y="77">
                one per workspace
              </text>
              <rect className="dp-node" x="590" y="112" width="280" height="54" rx="8" />
              <text className="dp-label" x="730" y="135">
                Encrypted document store
              </text>
              <text className="dp-sub" x="730" y="153">
                originals and backups
              </text>
              <rect className="dp-node" x="590" y="188" width="280" height="54" rx="8" />
              <text className="dp-label" x="730" y="211">
                llama.cpp runtime
              </text>
              <text className="dp-sub" x="730" y="229">
                Qwen3 4B, local weights
              </text>
              <line className="dp-line" x1="200" y1="150" x2="300" y2="150" />
              <line className="dp-line" x1="470" y1="150" x2="590" y2="63" />
              <line className="dp-line" x1="470" y1="150" x2="590" y2="139" />
              <line className="dp-line" x1="470" y1="150" x2="590" y2="215" />
              <text className="dp-sub" x="450" y="266" textAnchor="middle">
                OCR worker runs locally for scanned documents
              </text>
            </svg>
            <figcaption>
              Simplified from the architecture in the project README. The webview has no outbound
              network permission.
            </figcaption>
          </figure>
        </div>
      </section>

      <section className="s-story-section" id="limits">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Known limits</p>
          </div>
          <div>
            <h2 className="s-story-heading">What is not finished yet.</h2>
            <p className="s-story-copy">
              The current release record is marked not ready. These items are open. They are listed
              here so nobody relies on SAAR for a decision it has not yet earned.
            </p>
            <div className="s-limit-grid">
              {limits.map((limit) => (
                <div key={limit.title} className="s-limit">
                  <strong>{limit.title}</strong>
                  <p>{limit.body}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      <section className="s-story-section s-story-section-alt" id="evidence">
        <div className="s-wrap s-story-grid">
          <div>
            <p className="s-section-label">Evidence</p>
          </div>
          <div>
            <h2 className="s-story-heading">Read the audits and the threat model.</h2>
            <p className="s-story-copy">
              The repository keeps the security record next to the code: the egress audit, offline
              verification notes, the threat model, and the release gates.
            </p>
            <ul className="s-link-list">
              {evidence.map((item) => (
                <li key={item.path}>
                  <a href={`${REPO_URL}/blob/main/${item.path}`} target="_blank" rel="noreferrer">
                    {item.label} <span aria-hidden="true">↗</span>
                  </a>
                </li>
              ))}
            </ul>
            <p className="s-fineprint">
              Found a security problem? Do not post financial records or exploit details in a public
              issue. Use the <Link to="/contact">contact page</Link> to find the safe route.
            </p>
          </div>
        </div>
      </section>
    </>
  );
}
