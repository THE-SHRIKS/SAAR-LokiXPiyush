import appIcon from "../assets/brand/saar-app-icon.png";

/**
 * The SAAR logo as shown in the app. It is the square tile version, so the
 * black and blue strokes stay legible in both light and dark mode.
 */
export function BrandMark({ className }: { className?: string }) {
  return <img className={className} src={appIcon} alt="SAAR" draggable={false} />;
}
