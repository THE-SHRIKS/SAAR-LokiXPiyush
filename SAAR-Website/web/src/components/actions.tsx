import { Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { REPO_URL } from "@/data/site";

/**
 * Primary commercial action. Points to the official SAAR repository — the only
 * verified distribution channel while public installer publishing is prepared.
 */
export function GetSaar({
  variant = "solid",
  size = "md",
  label = "Get SAAR for Windows",
  className,
}: {
  variant?: "solid" | "onNavy" | "ghost";
  size?: "md" | "lg";
  label?: string;
  className?: string;
}) {
  return (
    <a
      href={REPO_URL}
      target="_blank"
      rel="noreferrer"
      className={cn(
        "group inline-flex items-center gap-3 font-medium transition-colors duration-200",
        size === "lg" ? "rounded-md px-7 py-4 text-[15px]" : "rounded-md px-5 py-3 text-sm",
        variant === "solid" && "bg-navy text-paper hover:bg-navy-deep",
        variant === "onNavy" && "bg-paper text-ink hover:bg-stone",
        variant === "ghost" && "border border-ink/25 text-ink hover:border-navy hover:text-navy",
        className,
      )}
    >
      <svg aria-hidden="true" viewBox="0 0 16 16" className="h-4 w-4 fill-current" focusable="false">
        <path d="M5.5 2.5h5a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-9a1 1 0 0 1 1-1Zm.5 9.5h4v1H6v-1Z" />
      </svg>
      {label}
      <ArrowRight className="h-4 w-4 transition-transform duration-200 group-hover:translate-x-0.5" aria-hidden="true" />
    </a>
  );
}

/** Internal editorial link with rule underline. */
export function TextLink({
  to,
  children,
  className,
}: {
  to: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <Link
      to={to}
      className={cn(
        "group inline-flex items-center gap-2 text-sm font-medium text-navy transition-colors hover:text-navy-deep",
        className,
      )}
    >
      {children}
      <ArrowRight className="h-3.5 w-3.5 transition-transform duration-200 group-hover:translate-x-0.5" aria-hidden="true" />
    </Link>
  );
}

/** Wordmark: navy "S" plate + serif SAAR, echoing the desktop app's own shell. */
export function Wordmark({ tone = "ink", className }: { tone?: "ink" | "paper"; className?: string }) {
  return (
    <span className={cn("inline-flex items-center gap-3", className)}>
      <span
        aria-hidden="true"
        className={cn(
          "flex h-8 w-8 items-center justify-center rounded-[5px] font-display text-lg leading-none",
          tone === "paper" ? "bg-paper text-navy" : "bg-navy text-paper",
        )}
      >
        S
      </span>
      <span className="flex flex-col leading-none">
        <span className={cn("font-display text-xl tracking-[0.04em]", tone === "paper" ? "text-paper" : "text-ink")}>
          SAAR
        </span>
        <span className={cn("mt-1 text-[9px] font-medium tracking-[0.18em] uppercase", tone === "paper" ? "text-paper/60" : "text-ink-soft")}>
          Financial Intelligence
        </span>
      </span>
    </span>
  );
}
