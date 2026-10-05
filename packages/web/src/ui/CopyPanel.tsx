import { cn } from "cn";
import { type ComponentType, type ReactNode, type SVGProps, useCallback, useEffect, useState } from "react";
import CopiedIcon from "~icons/lucide/check";
import FailedIcon from "~icons/lucide/circle-alert";
import CopyIcon from "~icons/lucide/copy";

import { Button } from "./Button";
import { copyFeedback, type CopyState } from "./copyFeedback";
import { TooltipTrigger } from "./TooltipTrigger";

const COPY_TOOLTIP = "Copy to the clipboard";

const COPY_BUTTON: Record<CopyState, { label: string; icon: ComponentType<SVGProps<SVGSVGElement>> }> = {
  idle: { label: "Copy", icon: CopyIcon },
  copied: { label: "Copied", icon: CopiedIcon },
  failed: { label: "Copy failed", icon: FailedIcon },
};

export function CopyPanel({ label, text, html, copyTooltip = COPY_TOOLTIP, className, children }: CopyPanelProps) {
  const [copyState, setCopyState] = useState<CopyState>("idle");
  const [feedback] = useState(() => copyFeedback(writeClipboard, setCopyState));
  const button = COPY_BUTTON[copyState];
  const copy = useCallback(() => void feedback.copy({ text, html }), [feedback, text, html]);

  useEffect(() => feedback.attach(), [feedback]);

  return (
    <figure className={cn("rounded-control border-rule bg-pane flex min-w-0 flex-col border", className)}>
      <div className="border-rule flex h-9 items-center justify-between gap-3 border-b pr-1 pl-3">
        <figcaption className="text-ink-muted truncate text-xs">{label}</figcaption>
        <TooltipTrigger tooltip={copyTooltip}>
          <Button
            variant="quiet"
            size="sm"
            icon={button.icon}
            onPress={copy}
            className={cn(copyState === "failed" && "text-danger")}
          >
            {button.label}
          </Button>
        </TooltipTrigger>
        <output className="sr-only">{copyState === "idle" ? "" : button.label}</output>
      </div>
      {children}
    </figure>
  );
}

export interface CopyPanelProps {
  label: string;
  text: string;
  html?: string | undefined;
  copyTooltip?: string | undefined;
  className?: string | undefined;
  children: ReactNode;
}

interface ClipboardContent {
  text: string;
  html: string | undefined;
}

async function writeClipboard({ text, html }: ClipboardContent): Promise<void> {
  try {
    await (html === undefined || typeof ClipboardItem === "undefined"
      ? navigator.clipboard.writeText(text)
      : navigator.clipboard.write([
          new ClipboardItem({
            "text/plain": new Blob([text], { type: "text/plain" }),
            "text/html": new Blob([html], { type: "text/html" }),
          }),
        ]));
  } catch (error) {
    console.error("Could not copy the text to the clipboard:", error);
    throw error;
  }
}
