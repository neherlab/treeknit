import { useCallback, useMemo, useRef } from "react";
import type { Key } from "react-aria-components";
import MoreIcon from "~icons/lucide/ellipsis";
import OpenIcon from "~icons/lucide/folder-open";
import RestoredIcon from "~icons/lucide/history";
import ClearIcon from "~icons/lucide/trash-2";
import UndoIcon from "~icons/lucide/undo-2";

import { PersistenceSwitch } from "../persistence/PersistenceSwitch";
import { Button } from "../ui/Button";
import { FilePicker, type FilePickerHandle } from "../ui/FilePicker";
import { IconButton } from "../ui/IconButton";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { Menu, MenuItem, MenuSeparator } from "../ui/Menu";
import { useWorkspace, useWorkspaceStore } from "../workspace/context";
import type { WorkspaceReplacement } from "../workspace/store";
import { SESSION_FILE_TYPES } from "./treeFiles";
import { useTreeInput } from "./useTreeInput";

export const RESTORED_FROM_BROWSER = "Restored from this browser";

const REPLACEMENT_NOTICES: Record<WorkspaceReplacement, string> = {
  clear: "Workspace cleared.",
  session: "Session file opened.",
};

const WORKSPACE_ACTIONS_TRIGGER = <IconButton label="Workspace actions" icon={MoreIcon} />;

export function WorkspaceHeader() {
  const store = useWorkspaceStore();
  const restored = useWorkspace((state) => state.restored);
  const undo = useWorkspace((state) => state.undo);
  const { input, error, dismissError } = useTreeInput();
  const sessionPicker = useRef<FilePickerHandle>(null);

  const act = useCallback(
    (key: Key) => {
      if (key === "open") {
        sessionPicker.current?.open();
      } else if (key === "clear") {
        store.getState().clear();
      }
    },
    [store],
  );

  const openSession = useCallback(
    (files: FileList | null) => {
      const file = files?.[0];

      if (file !== undefined) {
        void input.openSessionFile(file);
      }
    },
    [input],
  );

  const restore = useCallback(() => store.getState().restoreUndo(), [store]);

  const undoAction = useMemo(
    () => (
      <Button variant="quiet" size="sm" icon={UndoIcon} onPress={restore}>
        Undo
      </Button>
    ),
    [restore],
  );

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-start gap-2">
        <div className="min-w-0 flex-1 pt-1.5">
          <PersistenceSwitch />
        </div>
        <Menu aria-label="Workspace actions" placement="bottom end" onAction={act} trigger={WORKSPACE_ACTIONS_TRIGGER}>
          <MenuItem id="open" icon={OpenIcon}>
            Open session file
          </MenuItem>
          <MenuSeparator />
          <MenuItem id="clear" icon={ClearIcon} tone="danger">
            Clear workspace
          </MenuItem>
        </Menu>
      </div>
      <NoticeRegion>
        {restored ? (
          <p className="text-ink-muted flex items-center gap-1.5 text-xs">
            <RestoredIcon aria-hidden />
            {RESTORED_FROM_BROWSER}
          </p>
        ) : null}
        {undo?.kind === "workspace" ? (
          <InlineNotice tone="info" action={undoAction}>
            {REPLACEMENT_NOTICES[undo.reason]}
          </InlineNotice>
        ) : null}
      </NoticeRegion>
      {error === null ? null : (
        <InlineNotice tone="danger" onDismiss={dismissError}>
          {error}
        </InlineNotice>
      )}
      <FilePicker ref={sessionPicker} acceptedFileTypes={SESSION_FILE_TYPES} onSelect={openSession} />
    </div>
  );
}
