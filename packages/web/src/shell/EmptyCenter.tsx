import { useId, useMemo } from "react";
import TreesIcon from "~icons/lucide/list-tree";
import QuoteIcon from "~icons/lucide/quote";

import { CITE_REQUEST, Citation } from "../help/Citation";
import { TreeActions } from "../inputs/TreeActions";
import { TreeDropZone } from "../inputs/TreeDropZone";
import { useTreeInput } from "../inputs/useTreeInput";
import { EmptyState } from "../ui/EmptyState";
import { InlineNotice } from "../ui/InlineNotice";

export const EMPTY_CENTER_TITLE = "Drop Newick files here, paste a tree, or start with an example.";

export function EmptyCenter() {
  const { input, error, dismissError } = useTreeInput();

  const citeHeadingId = useId();

  const actions = useMemo(() => <TreeActions input={input} size="md" />, [input]);

  return (
    <div className="flex h-full flex-col gap-8 overflow-y-auto p-6">
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
      </TreeDropZone>
      <section aria-labelledby={citeHeadingId} className="flex w-full max-w-3xl shrink-0 flex-col gap-3">
        <h2 id={citeHeadingId} className="text-ink flex items-center gap-2 text-base font-semibold">
          <QuoteIcon aria-hidden className="text-ink-muted" />
          Cite TreeKnit
        </h2>
        <p className="text-sm">{CITE_REQUEST}</p>
        <Citation />
      </section>
    </div>
  );
}
