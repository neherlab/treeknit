import { useMemo } from "react";
import TreesIcon from "~icons/lucide/list-tree";

import { shortAttribution, TREEKNIT_PUBLICATION } from "../help/citation";
import { CiteButton } from "../help/CiteButton";
import { TreeActions } from "../inputs/TreeActions";
import { TreeDropZone } from "../inputs/TreeDropZone";
import { useTreeInput } from "../inputs/useTreeInput";
import { EmptyState } from "../ui/EmptyState";
import { InlineNotice } from "../ui/InlineNotice";

export const EMPTY_CENTER_TITLE = "Drop Newick files here, paste a tree, or start with an example.";

export function EmptyCenter() {
  const { input, error, dismissError } = useTreeInput();

  const actions = useMemo(() => <TreeActions input={input} size="md" />, [input]);

  return (
    <div className="flex h-full flex-col p-6">
      <TreeDropZone
        input={input}
        label="Drop Newick files"
        className="border-rule flex min-h-64 flex-1 flex-col items-start justify-center gap-4 border border-dashed px-8 py-10"
      >
        <EmptyState icon={TreesIcon} title={EMPTY_CENTER_TITLE} action={actions} />
        {error === null ? null : (
          <InlineNotice tone="danger" onDismiss={dismissError} className="max-w-xl">
            {error}
          </InlineNotice>
        )}
        <div className="text-ink-muted mt-4 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
          <span>TreeKnit by {shortAttribution(TREEKNIT_PUBLICATION)}</span>
          <CiteButton variant="quiet" size="xs" placement="bottom start" />
        </div>
      </TreeDropZone>
    </div>
  );
}
