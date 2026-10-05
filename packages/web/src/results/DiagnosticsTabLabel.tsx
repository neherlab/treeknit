import { useWorkspace } from "../workspace/context";

export function DiagnosticsTabLabel({ label }: DiagnosticsTabLabelProps) {
  const count = useWorkspace((state) => state.result?.summary.diagnostics.length ?? 0);

  return (
    <>
      {label}
      {count === 0 ? null : (
        <span className="rounded-control bg-pane text-ink min-w-5 px-1 text-center text-xs tabular-nums">{count}</span>
      )}
    </>
  );
}

export interface DiagnosticsTabLabelProps {
  label: string;
}
