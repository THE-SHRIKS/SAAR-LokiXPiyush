import { useEffect } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import { APP_VERSION, DOWNLOAD_CHECKSUM_URL, DOWNLOAD_FILE_NAME, DOWNLOAD_URL } from "@/data/site";
import { pageHead } from "@/lib/seo";

export const Route = createFileRoute("/download")({
  head: () =>
    pageHead({
      title: "Download SAAR for Windows | SAAR",
      description:
        "Download the SAAR Windows installer. Runs offline, keeps financial records on your machine, and includes a checksum to verify the file.",
      path: "/download",
    }),
  component: Download,
});

const requirements = [
  "Windows 10 or Windows 11, 64-bit",
  "Disk space for the app and for each encrypted workspace",
  "Local AI and OCR need model files and the llama.cpp runtime, which are not in the installer. See Get started.",
] as const;

const checks = [
  "The file is not code-signed yet, so Windows SmartScreen may warn before the installer runs.",
  "The release is pre-release. The open items are listed on the Security page.",
] as const;

function Download() {
  // The download starts as soon as this page opens, so the button on the rest
  // of the site leads straight to the installer.
  useEffect(() => {
    const link = document.createElement("a");
    link.href = DOWNLOAD_URL;
    link.download = DOWNLOAD_FILE_NAME;
    document.body.appendChild(link);
    link.click();
    link.remove();
  }, []);

  return (
    <>
      <section className="s-page-lead s-page-lead-download">
        <div className="s-wrap s-page-lead-grid">
          <div>
            <p className="s-section-label">Download / Windows installer</p>
            <h1>Your download starts now.</h1>
          </div>
          <p>
            If the installer did not start, use the button below. The installer runs on your
            machine, and SAAR works without an internet connection once it is installed.
          </p>
        </div>
      </section>

      <section className="s-story-section">
        <div className="s-wrap s-download-layout">
          <div className="s-download-card">
            <p className="s-section-label">SAAR {APP_VERSION}</p>
            <h2>{DOWNLOAD_FILE_NAME}</h2>
            <p className="s-download-meta">Windows installer · 64-bit · per-user install</p>
            <a className="s-button s-button-wide" href={DOWNLOAD_URL} download={DOWNLOAD_FILE_NAME}>
              Download installer <span aria-hidden="true">↓</span>
            </a>
            <p className="s-fineprint">
              Verify the file with its SHA-256 checksum:{" "}
              <a href={DOWNLOAD_CHECKSUM_URL}>{DOWNLOAD_FILE_NAME}.sha256</a>. In a command prompt,
              run <code>certutil -hashfile {DOWNLOAD_FILE_NAME} SHA256</code> and compare the
              result.
            </p>
          </div>

          <aside className="s-download-notes" aria-label="Before you install">
            <div>
              <p className="s-section-label">Requirements</p>
              <ul>
                {requirements.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
            <div>
              <p className="s-section-label">Before you install</p>
              <ul>
                {checks.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
            <p className="s-fineprint">
              Read the <Link to="/security">known limits</Link> before you import real financial
              records.
            </p>
          </aside>
        </div>
      </section>
    </>
  );
}
