import type { Diagnostic, Level } from "@neherlab/treeknit-wasm";
import { cn } from "cn";
import { DateTime } from "luxon";
import type { ComponentType, SVGProps } from "react";
import ErrorIcon from "~icons/lucide/circle-alert";
import InfoIcon from "~icons/lucide/info";
import WarningIcon from "~icons/lucide/triangle-alert";

import { EmptyState } from "../ui/EmptyState";
import { useWorkspace } from "../workspace/context";

export const NO_DIAGNOSTICS = "No warnings or errors in this run.";

const NO_ENTRIES: readonly Diagnostic[] = [];

const LEVELS: Record<Level, { label: string; icon: ComponentType<SVGProps<SVGSVGElement>>; className: string }> = {
  error: { label: "Error", icon: ErrorIcon, className: "text-danger" },
  warn: { label: "Warning", icon: WarningIcon, className: "text-ink" },
  info: { label: "Information", icon: InfoIcon, className: "text-ink-muted" },
  debug: { label: "Debug", icon: InfoIcon, className: "text-ink-muted" },
};

export function DiagnosticsView() {
  const diagnostics = useWorkspace((state) => state.result?.summary.diagnostics ?? NO_ENTRIES);

  if (diagnostics.length === 0) {
    return (
      <div className="px-8 py-10">
        <EmptyState title={NO_DIAGNOSTICS} />
      </div>
    );
  }

  return (
    <div className="flex max-w-5xl flex-col gap-4 px-8 py-6">
      <h2 className="text-base font-semibold">Warnings and errors</h2>
      <ol className="border-rule flex flex-col border-y">
        {diagnostics.map((diagnostic, index) => (
          <DiagnosticRow key={`${String(index)}-${diagnostic.time}`} diagnostic={diagnostic} />
        ))}
      </ol>
    </div>
  );
}

function DiagnosticRow({ diagnostic }: DiagnosticRowProps) {
  const level = LEVELS[diagnostic.level];
  const Icon = level.icon;
  const time = DateTime.fromISO(diagnostic.time);

  return (
    <li className="border-rule flex items-start gap-3 border-b px-1 py-2.5 text-sm last:border-b-0">
      <Icon aria-hidden className={cn("mt-0.5 shrink-0", level.className)} />
      <span className="sr-only">{level.label}: </span>
      <p className="min-w-0 flex-1 wrap-anywhere">{diagnostic.message}</p>
      {time.isValid ? (
        <time dateTime={diagnostic.time} className="text-ink-muted shrink-0 text-xs tabular-nums">
          {time.toLocaleString(DateTime.TIME_WITH_SECONDS)}
        </time>
      ) : null}
    </li>
  );
}

interface DiagnosticRowProps {
  diagnostic: Diagnostic;
}
