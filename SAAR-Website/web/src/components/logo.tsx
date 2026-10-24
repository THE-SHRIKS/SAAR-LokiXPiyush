/**
 * The SAAR logo, used everywhere on the site. The image is the same artwork
 * as the desktop app's icon source (LOGO.png at the repository root).
 */
export function Logo({ className = "s-logo-img" }: { className?: string }) {
  return <img className={className} src="/logo.png" alt="SAAR" width={960} height={316} />;
}
