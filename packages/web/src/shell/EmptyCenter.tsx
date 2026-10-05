import TreesIcon from "~icons/lucide/list-tree";

import { EmptyState } from "../ui/EmptyState";

export function EmptyCenter() {
  return (
    <div className="px-8 py-10">
      <EmptyState icon={TreesIcon} title="Drop Newick files here, paste a tree, or start with an example." />
    </div>
  );
}
