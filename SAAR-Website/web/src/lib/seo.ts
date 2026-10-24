import { SITE_DESCRIPTION, SITE_NAME, SITE_URL, ORG_URL, REPO_URL } from "@/data/site";

export const OG_IMAGE_PATH = "/og-image.png";
const OG_IMAGE_WIDTH = 1200;
const OG_IMAGE_HEIGHT = 630;

export function absoluteUrl(path: string): string {
  return `${SITE_URL}${path === "/" ? "" : path}`;
}

type PageHeadInput = {
  title: string;
  description: string;
  path: string;
  noindex?: boolean;
};

/**
 * Builds the per-route head: title, description, canonical link, robots,
 * Open Graph, and Twitter card tags. Each route passes its own title and
 * description, so search results and link previews describe that page.
 */
export function pageHead({ title, description, path, noindex = false }: PageHeadInput) {
  const url = absoluteUrl(path);
  const image = absoluteUrl(OG_IMAGE_PATH);
  return {
    meta: [
      { title },
      { name: "description", content: description },
      { name: "robots", content: noindex ? "noindex, nofollow" : "index, follow" },
      { property: "og:type", content: "website" },
      { property: "og:site_name", content: SITE_NAME },
      { property: "og:locale", content: "en_US" },
      { property: "og:title", content: title },
      { property: "og:description", content: description },
      { property: "og:url", content: url },
      { property: "og:image", content: image },
      { property: "og:image:width", content: String(OG_IMAGE_WIDTH) },
      { property: "og:image:height", content: String(OG_IMAGE_HEIGHT) },
      { property: "og:image:alt", content: "SAAR logo on a warm paper background" },
      { name: "twitter:card", content: "summary_large_image" },
      { name: "twitter:title", content: title },
      { name: "twitter:description", content: description },
      { name: "twitter:image", content: image },
    ],
    links: [{ rel: "canonical", href: url }],
  };
}

/**
 * Structured data for the organization, the website, and the desktop
 * application. Emitted once from the root route.
 */
export function siteJsonLd() {
  const organization = {
    "@type": "Organization",
    "@id": `${SITE_URL}/#organization`,
    name: "THE SHRIKS",
    url: ORG_URL,
  };
  const website = {
    "@type": "WebSite",
    "@id": `${SITE_URL}/#website`,
    name: SITE_NAME,
    url: SITE_URL,
    description: SITE_DESCRIPTION,
    publisher: { "@id": `${SITE_URL}/#organization` },
    inLanguage: "en-US",
  };
  const application = {
    "@type": "SoftwareApplication",
    "@id": `${SITE_URL}/#application`,
    name: SITE_NAME,
    description: SITE_DESCRIPTION,
    url: SITE_URL,
    downloadUrl: absoluteUrl("/download"),
    operatingSystem: "Windows 10, Windows 11",
    applicationCategory: "FinanceApplication",
    isAccessibleForFree: true,
    author: { "@id": `${SITE_URL}/#organization` },
    codeRepository: REPO_URL,
  };
  return {
    "@context": "https://schema.org",
    "@graph": [organization, website, application],
  };
}
