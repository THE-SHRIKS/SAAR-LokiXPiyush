import { useEffect, useState } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { Menu, X } from "lucide-react";
import { NAV } from "@/data/site";
import { Logo } from "@/components/logo";

export function SiteHeader() {
  const [open, setOpen] = useState(false);
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  useEffect(() => setOpen(false), [pathname]);

  return (
    <header className="s-header">
      <div className="s-wrap s-header-inner">
        <Link to="/" className="s-brand" aria-label="SAAR home">
          <Logo />
        </Link>
        <nav aria-label="Primary" className="s-nav">
          {NAV.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              aria-current={pathname === item.to ? "page" : undefined}
            >
              {item.label}
            </Link>
          ))}
        </nav>
        <Link className="s-button" to="/download">
          Download for Windows{" "}
          <span className="s-arrow" aria-hidden="true">
            ↓
          </span>
        </Link>
        <button
          className="s-menu"
          aria-expanded={open}
          aria-controls="mobile-nav"
          aria-label={open ? "Close navigation" : "Open navigation"}
          onClick={() => setOpen((v) => !v)}
        >
          {open ? <X size={19} /> : <Menu size={19} />}
        </button>
      </div>
      {open && (
        <nav id="mobile-nav" className="s-mobile-nav" aria-label="Mobile">
          {NAV.map((item) => (
            <Link key={item.to} to={item.to}>
              {item.label}
            </Link>
          ))}
          <Link className="s-mobile-cta" to="/download">
            Download for Windows ↓
          </Link>
        </nav>
      )}
    </header>
  );
}
