import { cn } from "@/lib/utils";
import { Annotation } from "@/components/editorial";

/**
 * Presentation frame for genuine SAAR application screenshots from the
 * official repository. A restrained desktop chrome — no faux-OS decoration,
 * just a title strip, a hairline border and a controlled shadow.
 */
export function AppFrame({
  src,
  alt,
  caption,
  index,
  className,
  priority = false,
}: {
  src: string;
  alt: string;
  caption?: string;
  index?: string;
  className?: string;
  priority?: boolean;
}) {
  return (
    <figure className={cn("group", className)}>
      <div className="overflow-hidden rounded-md border border-ink/15 bg-paper-raised shadow-[0_1px_2px_rgba(3,3,3,0.08),0_18px_44px_-18px_rgba(18,52,88,0.28)] transition-shadow duration-500 group-hover:shadow-[0_1px_2px_rgba(3,3,3,0.08),0_24px_56px_-16px_rgba(18,52,88,0.36)]">
        <div className="flex items-center justify-between border-b border-ink/10 bg-stone/60 px-4 py-2">
          <span className="data-mono text-[10px] font-medium tracking-[0.12em] text-ink-soft uppercase">SAAR — Windows Desktop</span>
          <span className="data-mono hidden text-[10px] text-ink-soft sm:inline">Air-gapped · Local</span>
        </div>
        <img
          src={src}
          alt={alt}
          loading={priority ? "eager" : "lazy"}
          decoding="async"
          className="block w-full"
        />
      </div>
      {caption ? (
        <figcaption className="mt-4 flex gap-3">
          {index ? <span className="data-mono shrink-0 text-[11px] font-medium text-navy">{index}</span> : null}
          <Annotation className="max-w-[68ch]">{caption}</Annotation>
        </figcaption>
      ) : null}
    </figure>
  );
}
