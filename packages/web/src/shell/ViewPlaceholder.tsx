import { EmptyState } from "../ui/EmptyState";

export function ViewPlaceholder() {
  return (
    <div className="px-8 py-10">
      <EmptyState title="Nothing to show for this result yet." />
    </div>
  );
}
