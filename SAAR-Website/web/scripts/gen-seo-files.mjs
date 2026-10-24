// Writes public/sitemap.xml and public/robots.txt from the production origin.
// Run before every build (see the prebuild script). The origin comes from
// VITE_SITE_URL, the same variable the app reads, so the sitemap, canonical
// links, and Open Graph URLs always agree.
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const publicDir = join(here, "..", "public");
const fallbackOrigin = "https://saar.vercel.app";
const origin = (process.env.VITE_SITE_URL || fallbackOrigin).replace(/\/$/, "");

const routes = [
  { path: "/", priority: "1.0" },
  { path: "/product", priority: "0.9" },
  { path: "/solutions", priority: "0.8" },
  { path: "/how-it-works", priority: "0.8" },
  { path: "/security", priority: "0.8" },
  { path: "/download", priority: "0.9" },
  { path: "/get-started", priority: "0.7" },
  { path: "/faq", priority: "0.7" },
  { path: "/about", priority: "0.6" },
  { path: "/contact", priority: "0.5" },
];

const lastmod = new Date().toISOString().slice(0, 10);
const urls = routes
  .map(
    (r) =>
      `  <url>\n    <loc>${origin}${r.path === "/" ? "/" : r.path}</loc>\n    <lastmod>${lastmod}</lastmod>\n    <priority>${r.priority}</priority>\n  </url>`,
  )
  .join("\n");

writeFileSync(
  join(publicDir, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`,
);

writeFileSync(
  join(publicDir, "robots.txt"),
  `User-agent: *\nAllow: /\n\nSitemap: ${origin}/sitemap.xml\n`,
);

console.log(`SEO files written for ${origin}`);
