import { Link } from "@tanstack/react-router";
import { FOOTER_NAV, NAV, REPO_URL } from "@/data/site";
import { Logo } from "@/components/logo";

export function SiteFooter() {
  return (
    <footer className="s-footer">
      <div className="s-wrap">
        <div className="s-footer-main">
          <div>
            <Link to="/" className="s-brand" aria-label="SAAR home">
              <Logo />
            </Link>
            <p className="s-footer-copy">
              A local workspace for financial records, exact calculations, and evidence linked to
              verified figures.
            </p>
          </div>
          <nav className="s-footer-nav" aria-label="Footer">
            {[...NAV, ...FOOTER_NAV].map((item) => (
              <Link key={item.to} to={item.to}>
                {item.label}
              </Link>
            ))}
            <a href={REPO_URL} target="_blank" rel="noreferrer">
              GitHub repository ↗
            </a>
          </nav>
        </div>
        <div className="s-footer-bottom">
          <p>© 2026 THE SHRIKS</p>
          <p>SAAR is developed for local use on Windows.</p>
        </div>
      </div>
    </footer>
  );
}
