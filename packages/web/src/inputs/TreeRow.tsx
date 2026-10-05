import type { NumberSetting, TreeInspection } from "@neherlab/treeknit-wasm";
import { cn } from "cn";
import { type ComponentType, type SVGProps, useCallback } from "react";
import { useKeyboard } from "react-aria";
import type { Key, KeyboardEvent } from "react-aria-components";
import ErrorIcon from "~icons/lucide/circle-alert";
import ReadyIcon from "~icons/lucide/circle-check";
import CodeIcon from "~icons/lucide/code";
import MoreIcon from "~icons/lucide/ellipsis";
import ReplaceIcon from "~icons/lucide/file-up";
import RemoveIcon from "~icons/lucide/trash-2";
import WarningIcon from "~icons/lucide/triangle-alert";

import { DraftNumberField } from "../settings/DraftNumberField";
import { useSettingsActions } from "../settings/useSettingsActions";
import { GridListItem } from "../ui/GridList";
import { IconButton } from "../ui/IconButton";
import { Menu, MenuItem, MenuSeparator } from "../ui/Menu";
import { errorStyle } from "../ui/styles";
import { TextField } from "../ui/TextField";
import { useWorkspaceStore } from "../workspace/context";
import { errorProps, type FieldErrors, fieldMessage, seqLengthField, treeField } from "../workspace/fieldErrors";
import type { WorkspaceTree } from "../workspace/store";
import { isGridNavigationKey, treeFacts } from "./treeFacts";
import { TREE_STATUS_LABELS, type TreeStatus, type TreeStatusKind, treeStatus } from "./treeStatus";

type Icon = ComponentType<SVGProps<SVGSVGElement>>;

const STATUS_ICONS: Record<Exclude<TreeStatusKind, "pending">, { icon: Icon; className: string }> = {
  error: { icon: ErrorIcon, className: "text-danger" },
  warning: { icon: WarningIcon, className: "text-ink" },
  ok: { icon: ReadyIcon, className: "text-ink-muted" },
};

const TREE_ACTIONS_TRIGGER = <IconButton label="Tree actions" icon={MoreIcon} />;

export function TreeRow({
  tree,
  index,
  inspection,
  missing,
  errors,
  seqLength,
  onViewNewick,
  onReplace,
}: TreeRowProps) {
  const store = useWorkspaceStore();
  const actions = useSettingsActions();
  const { keyboardProps } = useKeyboard({ onKeyDown: keepRowKeys });
  const status = treeStatus(inspection, errors.byField.get(treeField(index, "newick")) ?? []);
  const facts = treeFacts(inspection, missing);

  const rename = useCallback((label: string) => store.getState().renameTree(tree.id, label), [store, tree.id]);
  const commitSeqLength = useCallback((value: number) => actions.setSeqLength(tree.id, value), [actions, tree.id]);

  const viewNewick = useCallback(() => {
    onViewNewick(tree.id);
  }, [onViewNewick, tree.id]);

  const act = useCallback(
    (key: Key) => {
      if (key === "replace") {
        onReplace(tree.id);
      } else if (key === "newick") {
        onViewNewick(tree.id);
      } else if (key === "remove") {
        store.getState().removeTree(tree.id);
      }
    },
    [onReplace, onViewNewick, store, tree.id],
  );

  return (
    <GridListItem id={tree.id} textValue={tree.label}>
      <div {...keyboardProps} className="flex flex-col gap-1.5">
        <div className="flex items-start gap-1">
          <TextField
            label="Label"
            labelHidden
            value={tree.label}
            onChange={rename}
            {...errorProps(fieldMessage(errors, treeField(index, "label")))}
            className="flex-1"
          />
          <StatusButton status={status} onPress={viewNewick} />
          <Menu
            aria-label={`Actions for ${tree.label}`}
            placement="bottom end"
            onAction={act}
            trigger={TREE_ACTIONS_TRIGGER}
          >
            <MenuItem id="replace" icon={ReplaceIcon}>
              Replace file
            </MenuItem>
            <MenuItem id="newick" icon={CodeIcon}>
              View Newick
            </MenuItem>
            <MenuSeparator />
            <MenuItem id="remove" icon={RemoveIcon} tone="danger">
              Remove
            </MenuItem>
          </Menu>
        </div>
        {facts.length === 0 ? null : (
          <p className="text-ink-muted flex flex-wrap gap-x-3 px-0.5 text-xs">
            {facts.map((fact) => (
              <span key={fact}>{fact}</span>
            ))}
          </p>
        )}
        {status.messages.map((message) => (
          <p key={message} className={cn(errorStyle, "wrap-anywhere", status.kind === "warning" && "text-ink-muted")}>
            {status.kind === "warning" ? <WarningIcon aria-hidden /> : <ErrorIcon aria-hidden />}
            <span>{message}</span>
          </p>
        ))}
        {seqLength === null ? null : (
          <DraftNumberField
            name={`seqLengths.${tree.id}`}
            label="Sequence length"
            setting={seqLength}
            error={fieldMessage(errors, seqLengthField(index))}
            onCommit={commitSeqLength}
          />
        )}
      </div>
    </GridListItem>
  );
}

export interface TreeRowProps {
  tree: WorkspaceTree;
  index: number;
  inspection: TreeInspection | undefined;
  missing: number | undefined;
  errors: FieldErrors;
  seqLength: NumberSetting | null;
  onViewNewick: (treeId: string) => void;
  onReplace: (treeId: string) => void;
}

function StatusButton({ status, onPress }: StatusButtonProps) {
  if (status.kind === "pending") {
    return null;
  }

  const { icon, className } = STATUS_ICONS[status.kind];

  return <IconButton label={TREE_STATUS_LABELS[status.kind]} icon={icon} onPress={onPress} className={className} />;
}

interface StatusButtonProps {
  status: TreeStatus;
  onPress: () => void;
}

function keepRowKeys(event: KeyboardEvent): void {
  if (!isGridNavigationKey(event.key, event.ctrlKey || event.metaKey || event.altKey)) {
    event.continuePropagation();
  }
}
