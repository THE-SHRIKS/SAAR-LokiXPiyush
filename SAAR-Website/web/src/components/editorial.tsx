import { cn } from "@/lib/utils";

/** Mono uppercase section identifier. */
export function Eyebrow({
  children,
  className,
  tone = "ink",
}: {
  children: React.ReactNode;
  className?: string;
  tone?: "ink" | "navy" | "paper" | "muted";
}) {
  const toneClass =
    tone === "navy"
      ? "text-navy"
      : tone === "paper"
        ? "text-paper/70"
        : tone === "muted"
          ? "text-ink-soft"
          : "text-ink";
  return (
    <p className={cn("eyebrow flex items-center gap-3", toneClass, className)}>
      <span aria-hidden="true" className="inline-block h-px w-6 bg-current opacity-60" />
      {children}
    </p>
  );
}

/** Large editorial section heading in the display serif. */
export function SectionTitle({
  children,
  className,
  as: Tag = "h2",
  size = "md",
}: {
  children: React.ReactNode;
  className?: string;
  as?: "h1" | "h2" | "h3";
  size?: "sm" | "md" | "lg";
}) {
  const sizeClass =
    size === "lg"
      ? "text-4xl md:text-6xl"
      : size === "sm"
        ? "text-2xl md:text-3xl"
        : "text-3xl md:text-5xl";
  return (
    <Tag className={cn("font-display font-normal leading-[1.08] tracking-[-0.01em]", sizeClass, className)}>
      {children}
    </Tag>
  );
}

/** Body copy column with comfortable measure. */
export function Copy({ children, className }: { children: React.ReactNode; className?: string }) {
  return <p className={cn("max-w-[62ch] text-[15px] leading-[1.75] text-ink-soft md:text-base", className)}>{children}</p>;
}

/** Mono annotation for illustrations and figures. */
export function Annotation({ children, className }: { children: React.ReactNode; className?: string }) {
  return <p className={cn("data-mono text-[11px] leading-relaxed text-ink-soft", className)}>{children}</p>;
}

/** Numbered index used across editorial lists. */
export function IndexNum({ n, tone = "navy" }: { n: string; tone?: "navy" | "paper" }) {
  return (
    <span aria-hidden="true" className={cn("data-mono shrink-0 text-xs font-medium", tone === "navy" ? "text-navy" : "text-paper/60")}>
      {n}
    </span>
  );
}
