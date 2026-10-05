import { cn } from "cn";
import { useCallback, useEffect, useId, useMemo, useState } from "react";
import CopiedIcon from "~icons/lucide/check";
import FailedIcon from "~icons/lucide/circle-alert";
import CopyIcon from "~icons/lucide/copy";

import { Button } from "./Button";
import { type CodeLine, codeLines, type TextRange } from "./codeLines";
import { copyFeedback, type CopyState } from "./copyFeedback";
import type { IconComponent } from "./icon";
import { revealScroll } from "./revealScroll";
import { nativeFocusRing } from "./styles";

const COPY_BUTTON: Record<CopyState, { label: string; icon: IconComponent }> = {
  idle: { label: "Copy", icon: CopyIcon },
  copied: { label: "Copied", icon: CopiedIcon },
  failed: { label: "Copy failed", icon: FailedIcon },
};

export function CodeBlock({ code, label, highlight, lineNumbers, className }: CodeBlockProps) {
  const [copyState, setCopyState] = useState<CopyState>("idle");
  const [feedback] = useState(() => copyFeedback(writeClipboard, setCopyState));
  const lines = useMemo(() => codeLines(code, highlight), [code, highlight]);
  const showLineNumbers = lineNumbers ?? lines.length > 1;
  const button = COPY_BUTTON[copyState];
  const copy = useCallback(() => void feedback.copy(code), [feedback, code]);
  const captionId = useId();

  useEffect(
    () => () => {
      feedback.dispose();
    },
    [feedback],
  );

  return (
    <figure className={cn("rounded-control border-rule bg-pane flex min-w-0 flex-col border", className)}>
      <div className="border-rule flex h-9 items-center justify-between gap-3 border-b pr-1 pl-3">
        <figcaption id={captionId} className="text-ink-muted truncate text-xs">
          {label}
        </figcaption>
        <Button
          variant="quiet"
          size="sm"
          icon={button.icon}
          onPress={copy}
          className={cn(copyState === "failed" && "text-danger")}
        >
          {button.label}
        </Button>
        <output className="sr-only">{copyState === "idle" ? "" : button.label}</output>
      </div>
      <section aria-labelledby={captionId} className={cn("rounded-b-control max-h-96 overflow-auto", nativeFocusRing)}>
        <pre className="text-ink px-3 py-2.5 font-mono text-sm">
          <code
            className={cn("grid", showLineNumbers ? "grid-cols-[auto_minmax(0,1fr)]" : "grid-cols-[minmax(0,1fr)]")}
          >
            {lines.map((line) => (
              <CodeLineRow
                key={line.number}
                line={line}
                lineNumber={showLineNumbers}
                revealKey={
                  line.number === highlight?.start.line
                    ? `${highlight.start.line}:${highlight.start.column}`
                    : undefined
                }
              />
            ))}
          </code>
        </pre>
      </section>
    </figure>
  );
}

export interface CodeBlockProps {
  code: string;
  label: string;
  highlight?: TextRange;
  lineNumbers?: boolean;
  className?: string;
}

async function writeClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch (error) {
    console.error("Could not copy the text to the clipboard:", error);
    throw error;
  }
}

function CodeLineRow({ line, lineNumber, revealKey }: CodeLineRowProps) {
  const isHighlighted = line.marked !== undefined;

  return (
    <>
      {lineNumber ? (
        <span
          aria-hidden
          className={cn(
            "text-ink-muted -ml-3 pr-3 pl-3 text-right select-none",
            isHighlighted && "bg-danger/10 text-danger font-semibold",
          )}
        >
          {line.number}
        </span>
      ) : null}
      <span className={cn("min-h-lh wrap-anywhere whitespace-pre-wrap", isHighlighted && "bg-danger/10")}>
        {line.before}
        {line.marked === undefined ? null : (
          <mark
            key={revealKey}
            ref={revealKey === undefined ? undefined : revealMark}
            className={cn(
              "bg-danger/25 text-ink decoration-danger rounded-[2px] underline decoration-2 underline-offset-2",
              line.marked === "" && "inline-block h-lh w-2 align-top",
            )}
          >
            {line.marked}
          </mark>
        )}
        {line.after}
      </span>
    </>
  );
}

interface CodeLineRowProps {
  line: CodeLine;
  lineNumber: boolean;
  revealKey: string | undefined;
}

function revealMark(mark: HTMLElement | null) {
  const container = mark?.closest("section");

  if (mark === null || container === null || container === undefined) {
    return;
  }

  const view = container.getBoundingClientRect();
  const item = mark.getBoundingClientRect();
  const top = view.top + container.clientTop;
  const left = view.left + container.clientLeft;

  container.scrollBy({
    top: revealScroll({ start: item.top, end: item.bottom }, { start: top, end: top + container.clientHeight }),
    left: revealScroll({ start: item.left, end: item.right }, { start: left, end: left + container.clientWidth }),
  });
}
