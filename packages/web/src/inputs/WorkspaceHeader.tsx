import { useCallback, useRef } from "react";
import { isDeepEqual } from "remeda";
import OpenIcon from "~icons/lucide/folder-open";
import RestoredIcon from "~icons/lucide/history";
import ClearIcon from "~icons/lucide/trash-2";

import { LaunchNotices } from "../launch/LaunchNotices";
import { PersistenceSwitch } from "../persistence/PersistenceSwitch";
import { FilePicker, type FilePickerHandle } from "../ui/FilePicker";
import { IconButton } from "../ui/IconButton";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { useWorkspace, useWorkspaceStore } from "../workspace/context";
import { SESSION_FILE_TYPES } from "./treeFiles";
import { useTreeInput } from "./useTreeInput";

export const RESTORED_FROM_BROWSER = "Restored from this browser";

export function WorkspaceHeader() {
  const store = useWorkspaceStore();
  const restored = useWorkspace((state) => state.restored);

  const isEmpty = useWorkspace(
    (state) => state.trees.length === 0 && state.result === null && isDeepEqual(state.settings, state.defaults),
  );

  const { input, error, dismissError } = useTreeInput();
  const sessionPicker = useRef<FilePickerHandle>(null);

  const open = useCallback(() => {
    sessionPicker.current?.open();
  }, []);

  const clear = useCallback(() => {
    store.getState().clear();
  }, [store]);

  const openSession = useCallback(
    (files: FileList | null) => {
      const file = files?.[0];

      if (file !== undefined) {
        void input.openSessionFile(file);
      }
    },
    [input],
  );

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-start gap-1">
        <div className="min-w-0 flex-1 pt-1.5">
          <PersistenceSwitch />
        </div>
        <IconButton label="Open session file" icon={OpenIcon} onPress={open} />
        <IconButton label="Clear workspace" icon={ClearIcon} isDisabled={isEmpty} onPress={clear} />
      </div>
      <NoticeRegion>
        {restored ? (
          <p className="text-ink-muted flex items-center gap-1.5 text-xs">
            <RestoredIcon aria-hidden />
            {RESTORED_FROM_BROWSER}
          </p>
        ) : null}
      </NoticeRegion>
      <LaunchNotices />
      {error === null ? null : (
        <InlineNotice tone="danger" onDismiss={dismissError}>
          {error}
        </InlineNotice>
      )}
      <FilePicker ref={sessionPicker} acceptedFileTypes={SESSION_FILE_TYPES} onSelect={openSession} />
    </div>
  );
}
