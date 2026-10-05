import { counted } from "../drawing/format";
import { useWorkspace } from "../workspace/context";

export function DiagnosticsTabLabel({ label }: DiagnosticsTabLabelProps) {
  const count = useWorkspace((state) => state.result?.summary.diagnostics.length ?? 0);

  return (
    <>
      {label}
      {count === 0 ? null : (
        <>
          <span aria-hidden className="rounded-control bg-pane text-ink min-w-5 px-1 text-center text-xs tabular-nums">
            {count}
          </span>
          <span className="sr-only">{`, ${counted(count, "message", "messages")}`}</span>
        </>
      )}
    </>
  );
}

export interface DiagnosticsTabLabelProps {
  label: string;
}
