/**
 * Shared site constants. Every external link points to a verified official
 * resource: the SAAR desktop repository owned by THE SHRIKS on GitHub.
 */

/**
 * The production origin, used for canonical URLs, sitemap entries, and Open
 * Graph links. Set VITE_SITE_URL in the Vercel project to the real domain.
 */
export const SITE_URL =
  (import.meta.env["VITE_SITE_URL"] as string | undefined)?.replace(/\/$/, "") ??
  "https://saar.vercel.app";
export const SITE_NAME = "SAAR";
export const SITE_DESCRIPTION =
  "SAAR is an offline Windows desktop workspace for financial documents. Import invoices, receipts, and bank rows, reconcile them, and trace every figure back to its source.";

/** Installer served from /downloads. Override with VITE_DOWNLOAD_URL to host it elsewhere. */
export const DOWNLOAD_FILE_NAME = "SAAR-Setup.exe";
export const DOWNLOAD_URL =
  (import.meta.env["VITE_DOWNLOAD_URL"] as string | undefined) ??
  `/downloads/${DOWNLOAD_FILE_NAME}`;

export const REPO_URL = "https://github.com/THE-SHRIKS/SAAR-LokiXPiyush";
export const REPO_ISSUES_URL = "https://github.com/THE-SHRIKS/SAAR-LokiXPiyush/issues";
export const ORG_URL = "https://github.com/THE-SHRIKS";
export const APP_VERSION = "v0.1.0";

export const NAV = [
  { label: "Home", to: "/" },
  { label: "Product", to: "/product" },
  { label: "Solutions", to: "/solutions" },
  { label: "How It Works", to: "/how-it-works" },
  { label: "Security", to: "/security" },
  { label: "Contact", to: "/contact" },
] as const;

export const FOOTER_NAV = [
  { label: "Get started", to: "/get-started" },
  { label: "FAQ", to: "/faq" },
  { label: "About", to: "/about" },
] as const;
