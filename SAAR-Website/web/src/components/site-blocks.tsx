import { Link } from "@tanstack/react-router";
import { REPO_URL } from "@/data/site";

export function Screenshot({
  src,
  alt,
  caption,
  priority = false,
}: {
  src: string;
  alt: string;
  caption?: string;
  priority?: boolean;
}) {
  return (
    <figure className="s-shot">
      <div className="s-shot-media">
        <img src={src} alt={alt} loading={priority ? "eager" : "lazy"} decoding="async" />
      </div>
      {caption && <figcaption>{caption}</figcaption>}
    </figure>
  );
}

export function SourceButton({
  light = false,
  children = "View source",
}: {
  light?: boolean;
  children?: string;
}) {
  return (
    <a
      href={REPO_URL}
      target="_blank"
      rel="noreferrer"
      className={`s-button${light ? " s-button-light" : ""}`}
    >
      {children}
      <span className="s-arrow" aria-hidden="true">
        ↗
      </span>
    </a>
  );
}

export function InternalButton({
  to,
  children,
}: {
  to: "/product" | "/solutions" | "/how-it-works" | "/contact";
  children: string;
}) {
  return (
    <Link className="s-button s-button-quiet" to={to}>
      {children}
      <span className="s-arrow" aria-hidden="true">
        →
      </span>
    </Link>
  );
}
