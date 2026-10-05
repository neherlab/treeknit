import { useCallback, useMemo, useRef, useState } from "react";
import { FileTrigger, type Key } from "react-aria-components";
import ErrorIcon from "~icons/lucide/circle-alert";
import UndoIcon from "~icons/lucide/undo-2";

import { useInspectTrees, useOverlap, useSettingsSchema } from "../analysis/queries";
import { visibleGeneralErrors } from "../run/runControl";
import { Button } from "../ui/Button";
import { GridList } from "../ui/GridList";
import { InfoButton } from "../ui/InfoButton";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { reorder, type ReorderMove } from "../ui/reorder";
import { errorStyle } from "../ui/styles";
import { useCurrentRequest, useWorkspace, useWorkspaceStore } from "../workspace/context";
import { TREE_LIST_FIELD } from "../workspace/fieldErrors";
import { useFieldErrors } from "../workspace/useFieldErrors";
import { NewickDialog } from "./NewickDialog";
import { TreeActions } from "./TreeActions";
import { TreeDropZone } from "./TreeDropZone";
import { TREE_FILE_TYPES } from "./treeFiles";
import { TreeRow } from "./TreeRow";
import { treeStatus } from "./treeStatus";
import { useTreeInput } from "./useTreeInput";

export function TreeList() {
  const trees = useWorkspace((state) => state.trees);
  const settings = useWorkspace((state) => state.settings);
  const undo = useWorkspace((state) => state.undo);
  const store = useWorkspaceStore();
  const request = useCurrentRequest();
  const inspections = useInspectTrees(request.trees);
  const { data: overlap } = useOverlap(request.trees);
  const { data: schema } = useSettingsSchema(trees.length, settings);
  const errors = useFieldErrors();
  const { input, error, dismissError } = useTreeInput();
  const [viewingId, setViewingId] = useState<string | null>(null);
  const replacingId = useRef<string | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const seqLengthsOn = settings.seqLengths !== null && settings.seqLengths !== undefined;
  const viewingIndex = trees.findIndex(({ id }) => id === viewingId);
  const viewing = trees[viewingIndex];

  const missing = useMemo(
    () => new Map(overlap?.trees.map(({ index, missing: count }) => [index, count]) ?? []),
    [overlap],
  );

  const move = useCallback(
    (moved: ReorderMove) => {
      const order = reorder(store.getState().trees, ({ id }) => id, moved).map(({ id }) => id);

      store.getState().reorderTrees(order);
    },
    [store],
  );

  const dragLabel = useCallback(
    (key: Key) => store.getState().trees.find(({ id }) => id === key)?.label ?? String(key),
    [store],
  );

  const replace = useCallback((treeId: string) => {
    replacingId.current = treeId;
    fileInput.current?.click();
  }, []);

  const replaceWith = useCallback(
    (files: FileList | null) => {
      const file = files?.[0];
      const treeId = replacingId.current;

      if (file !== undefined && treeId !== null) {
        void input.replaceFile(treeId, file);
      }
    },
    [input],
  );

  const closeNewick = useCallback((open: boolean) => {
    if (!open) {
      setViewingId(null);
    }
  }, []);

  const restore = useCallback(() => store.getState().restoreUndo(), [store]);

  const rowDependencies = useMemo(
    () => [errors, inspections, missing, schema, seqLengthsOn],
    [errors, inspections, missing, schema, seqLengthsOn],
  );

  const undoAction = useMemo(
    () => (
      <Button variant="quiet" size="sm" icon={UndoIcon} onPress={restore}>
        Undo
      </Button>
    ),
    [restore],
  );

  const dismissAction = useMemo(
    () => (
      <Button variant="quiet" size="sm" onPress={dismissError}>
        Dismiss
      </Button>
    ),
    [dismissError],
  );

  const listErrors = visibleGeneralErrors(trees.length, errors.byField.get(TREE_LIST_FIELD) ?? []);

  return (
    <section aria-labelledby="rail-trees" className="flex flex-col gap-3">
      <div className="flex items-center gap-1">
        <h2 id="rail-trees" className="text-ink text-sm font-semibold">
          Trees
        </h2>
        {schema === undefined ? null : <InfoButton topic="tree order">{schema.treeOrderHelp}</InfoButton>}
      </div>
      <TreeDropZone input={input} label="Drop Newick files" className="-mx-1 px-1">
        <GridList
          aria-label="Trees"
          items={trees}
          onReorder={move}
          dragLabel={dragLabel}
          keyboardNavigationBehavior="tab"
          disallowTypeAhead
          renderEmptyState={renderNoTrees}
          dependencies={rowDependencies}
        >
          {(tree) => {
            const index = trees.indexOf(tree);

            return (
              <TreeRow
                tree={tree}
                index={index}
                inspection={inspections[index]}
                missing={missing.get(index)}
                errors={errors}
                seqLength={seqLengthsOn ? (schema?.settings.seqLengths ?? null) : null}
                onViewNewick={setViewingId}
                onReplace={replace}
              />
            );
          }}
        </GridList>
      </TreeDropZone>
      {listErrors.length === 0 ? null : (
        <p className={errorStyle}>
          <ErrorIcon aria-hidden />
          <span>{listErrors.map(({ message }) => message).join("; ")}</span>
        </p>
      )}
      <NoticeRegion>
        {undo?.kind === "tree" ? (
          <InlineNotice tone="info" action={undoAction}>
            Removed {undo.tree.label}.
          </InlineNotice>
        ) : null}
      </NoticeRegion>
      {error === null ? null : (
        <InlineNotice tone="danger" action={dismissAction}>
          {error}
        </InlineNotice>
      )}
      <TreeActions input={input} />
      <FileTrigger ref={fileInput} acceptedFileTypes={TREE_FILE_TYPES} onSelect={replaceWith} />
      {viewing === undefined ? null : (
        <NewickDialog
          label={viewing.label}
          newick={viewing.newick}
          errorRange={treeStatus(inspections[viewingIndex], []).errorRange}
          isOpen
          onOpenChange={closeNewick}
        />
      )}
    </section>
  );
}

function renderNoTrees() {
  return <p className="text-ink-muted px-2 py-3 text-sm">No trees yet.</p>;
}
