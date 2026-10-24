/**
 * Shared site constants. Every external link points to a verified official
 * resource: the SAAR desktop repository owned by THE SHRIKS on GitHub.
 */

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
