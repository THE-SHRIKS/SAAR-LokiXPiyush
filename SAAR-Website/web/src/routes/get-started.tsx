import { createFileRoute, Link } from "@tanstack/react-router";
import { REPO_URL } from "@/data/site";
import { pageHead } from "@/lib/seo";

export const Route = createFileRoute("/get-started")({
  head: () =>
    pageHead({
      title: "Get started | SAAR",
      description:
        "Build SAAR from source on Windows: prerequisites, local model files, the development run, and the checks to run before you trust a result.",
      path: "/get-started",
    }),
  component: GetStarted,
});

const steps = [
  {
    index: "01",
    title: "Install the prerequisites",
    body: "SAAR is a source repository. You build the desktop app yourself with the Windows native toolchain.",
    items: [
      "Node.js and pnpm, for the React interface.",
      "Rust, plus the Tauri 2 build prerequisites for Windows.",
      "Visual Studio C++ build tools, and WebView2.",
    ],
    code: null,
  },
  {
    index: "02",
    title: "Clone and install dependencies",
    body: "Install with the lockfile so the frontend matches the recorded versions.",
    items: [],
    code: `git clone ${REPO_URL}.git
cd SAAR-LokiXPiyush
pnpm install --frozen-lockfile
cargo install tauri-cli --version '^2' --locked`,
  },
  {
    index: "03",
    title: "Add the local model files",
    body: "Model weights and the llama.cpp binary are not committed to Git. Without them the workspace still starts, but chat and classification are unavailable.",
    items: [
      "Place the runtime at bin/llama/llama-cli.exe.",
      "Place the weights at models/weights/Qwen3-4B-Q4_K_M.gguf.",
      "Check the weights against the recorded SHA-256 before first use.",
    ],
    code: `sha256sum models/weights/Qwen3-4B-Q4_K_M.gguf
# expected: 7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5`,
  },
  {
    index: "04",
    title: "Start the development build",
    body: "The first run compiles the Rust core, which takes a while. Later runs reuse the build cache.",
    items: [],
    code: "cargo tauri dev",
  },
  {
    index: "05",
    title: "Run the checks",
    body: "Before you rely on a build, run the test suites. A green build is not the same as a verified workflow.",
    items: [],
    code: `pnpm test
pnpm run build
pnpm run lint
cargo test --manifest-path src-tauri/Cargo.toml`,
  },
] as const;

function GetStarted() {
  return (
    <>
      <section className="s-page-lead s-page-lead-start">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Get started / Build from source</p>
            <h1>From clone to a running workspace.</h1>
          </div>
          <p>
            Five steps take you from a fresh checkout to the desktop app. The steps match the
            project README and the Windows build guide, and each one ends with something you can
            check.
          </p>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-start-layout">
          <ol className="s-start-steps">
            {steps.map((step) => (
              <li key={step.index} className="s-start-step">
                <span className="s-start-index">{step.index}</span>
                <div className="s-start-body">
                  <h2>{step.title}</h2>
                  <p>{step.body}</p>
                  {step.items.length > 0 && (
                    <ul>
                      {step.items.map((item) => (
                        <li key={item}>{item}</li>
                      ))}
                    </ul>
                  )}
                  {step.code && (
                    <pre className="s-code">
                      <code>{step.code}</code>
                    </pre>
                  )}
                </div>
              </li>
            ))}
          </ol>

          <aside className="s-start-aside" aria-label="Before you begin">
            <p className="s-section-label">Before you begin</p>
            <h2>Use synthetic records first.</h2>
            <p>
              The repository includes a synthetic corpus and golden fixtures under{" "}
              <code>tests/</code>. Try SAAR on those before you import anything real.
            </p>
            <p>
              The release is not yet marked ready. The <Link to="/security">known limits</Link> page
              lists what is still open.
            </p>
            <a
              className="s-button s-button-wide"
              href={`${REPO_URL}/blob/main/Docs/WINDOWS_BUILD_GUIDE.md`}
              target="_blank"
              rel="noreferrer"
            >
              Windows build guide <span aria-hidden="true">↗</span>
            </a>
            <Link className="s-text-link" to="/faq">
              Read the FAQ →
            </Link>
          </aside>
        </div>
      </section>
    </>
  );
}
