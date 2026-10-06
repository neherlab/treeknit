import { match } from "ts-pattern";

import type { UndoEntry, WorkspaceReplacement } from "./store";

const REPLACEMENT_MESSAGES: Record<WorkspaceReplacement, string> = {
  clear: "Workspace cleared",
  session: "Session file opened",
  link: "Link opened",
};

export function undoMessage(entry: UndoEntry): string {
  return match(entry)
    .with({ kind: "tree" }, ({ tree }) => `Removed ${tree.label}`)
    .with({ kind: "workspace" }, ({ reason }) => REPLACEMENT_MESSAGES[reason])
    .with({ kind: "settings" }, () => "Settings reset to defaults")
    .exhaustive();
}
