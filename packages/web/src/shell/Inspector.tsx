import { EmptyState } from "../ui/EmptyState";

export function Inspector() {
  return (
    <div className="px-4 py-4">
      <EmptyState title="Select a leaf, a branch, or an MCC to see details." className="text-ink-muted" />
    </div>
  );
}
