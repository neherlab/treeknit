import { cn } from "cn";
import {
  type ComponentType,
  type RefObject,
  type SVGProps,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import CopiedIcon from "~icons/lucide/check";
import FailedIcon from "~icons/lucide/circle-alert";
import CopyIcon from "~icons/lucide/copy";

import { Button } from "./Button";
import { type CodeLine, codeLines, type TextRange } from "./codeLines";
import { copyFeedback, type CopyState } from "./copyFeedback";
import { revealOffset } from "./revealScroll";
import { nativeFocusRing } from "./styles";

const COPY_BUTTON: Record<CopyState, { label: string; icon: ComponentType<SVGProps<SVGSVGElement>> }> = {
  idle: { label: "Copy", icon: CopyIcon },
  copied: { label: "Copied", icon: CopiedIcon },
  failed: { label: "Copy failed", icon: FailedIcon },
};

export function CodeBlock({ code, label, errorRange, lineNumbers, className }: CodeBlockProps) {
  const [copyState, setCopyState] = useState<CopyState>("idle");
  const [feedback] = useState(() => copyFeedback(writeClipboard, setCopyState));
  const startLine = errorRange?.start.line;
  const startColumn = errorRange?.start.column;
  const endLine = errorRange?.end.line;
  const endColumn = errorRange?.end.column;

  const lines = useMemo(
    () => codeLines(code, textRange(startLine, startColumn, endLine, endColumn)),
    [code, startLine, startColumn, endLine, endColumn],
  );

  const scrollerRef = useRef<HTMLDivElement>(null);
  const markRef = useRef<HTMLElement>(null);
  const showLineNumbers = lineNumbers ?? lines.length > 1;
  const button = COPY_BUTTON[copyState];
  const copy = useCallback(() => void feedback.copy(code), [feedback, code]);

  useEffect(() => feedback.attach(), [feedback]);

  const markedLine = lines.find((line) => line.marked !== undefined)?.number;
  const overflows = useOverflow(scrollerRef);

  useLayoutEffect(() => {
    if (lines.some((line) => line.marked !== undefined)) {
      revealMark(markRef.current, scrollerRef.current);
    }
  }, [lines]);

  return (
    <figure className={cn("rounded-control border-rule bg-pane flex min-w-0 flex-col border", className)}>
      <div className="border-rule flex h-9 items-center justify-between gap-3 border-b pr-1 pl-3">
        <figcaption className="text-ink-muted truncate text-xs">{label}</figcaption>
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
      <div
        ref={scrollerRef}
        // oxlint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- a scroller with overflow must take focus so keyboard users can scroll long code, and WebKit does not make scroll containers focusable by itself; a landmark role would name the code a second time after the caption
        tabIndex={overflows ? 0 : undefined}
        className={cn("rounded-b-control max-h-96 overflow-auto", nativeFocusRing)}
      >
        <pre className="text-ink px-3 py-2.5 font-mono text-sm">
          <code
            className={cn("grid", showLineNumbers ? "grid-cols-[auto_minmax(0,1fr)]" : "grid-cols-[minmax(0,1fr)]")}
          >
            {lines.map((line) => (
              <CodeLineRow
                key={line.number}
                line={line}
                lineNumber={showLineNumbers}
                markRef={line.number === markedLine ? markRef : undefined}
              />
            ))}
          </code>
        </pre>
      </div>
    </figure>
  );
}

export interface CodeBlockProps {
  code: string;
  label: string;
  errorRange?: TextRange;
  lineNumbers?: boolean;
  className?: string;
}

function useOverflow(ref: RefObject<HTMLElement | null>): boolean {
  const subscribe = useCallback(
    (onChange: () => void) => {
      const element = ref.current;

      if (element === null) {
        return () => undefined;
      }

      const observer = new ResizeObserver(onChange);

      observer.observe(element);

      for (const child of element.children) {
        observer.observe(child);
      }

      return () => {
        observer.disconnect();
      };
    },
    [ref],
  );

  const overflows = useCallback(() => {
    const element = ref.current;

    return (
      element !== null && (element.scrollWidth > element.clientWidth || element.scrollHeight > element.clientHeight)
    );
  }, [ref]);

  return useSyncExternalStore(subscribe, overflows, noOverflowOnServer);
}

function noOverflowOnServer(): boolean {
  return false;
}

async function writeClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch (error) {
    console.error("Could not copy the text to the clipboard:", error);
    throw error;
  }
}

function CodeLineRow({ line, lineNumber, markRef }: CodeLineRowProps) {
  const isErrorLine = line.marked !== undefined;

  return (
    <>
      {lineNumber ? (
        <span
          aria-hidden
          className={cn(
            "text-ink-muted -ml-3 pr-3 pl-3 text-right select-none",
            isErrorLine && "bg-danger/10 text-danger font-semibold",
          )}
        >
          {line.number}
        </span>
      ) : null}
      <span className={cn("min-h-lh wrap-anywhere whitespace-pre-wrap", isErrorLine && "bg-danger/10")}>
        {line.before}
        {line.marked === undefined ? null : (
          <mark
            ref={markRef}
            className={cn(
              "bg-danger/25 text-ink decoration-danger rounded-inner underline decoration-2 underline-offset-2",
              line.marked === "" && "inline-block h-lh w-2 align-top",
            )}
          >
            {line.marked === "" ? <span className="sr-only">Error position</span> : line.marked}
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
  markRef: RefObject<HTMLElement | null> | undefined;
}

function textRange(
  startLine: number | undefined,
  startColumn: number | undefined,
  endLine: number | undefined,
  endColumn: number | undefined,
): TextRange | undefined {
  if (startLine === undefined || startColumn === undefined || endLine === undefined || endColumn === undefined) {
    return undefined;
  }

  return { start: { line: startLine, column: startColumn }, end: { line: endLine, column: endColumn } };
}

function revealMark(mark: HTMLElement | null, scroller: HTMLElement | null) {
  if (mark === null || scroller === null) {
    return;
  }

  scroller.scrollBy(revealOffset(mark.getBoundingClientRect(), scroller.getBoundingClientRect(), scroller));
}
