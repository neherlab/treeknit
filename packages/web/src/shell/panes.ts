const PANES = ["rail", "inspector", "auspice"] as const;

export type PaneId = (typeof PANES)[number];

export type ShellPaneId = Exclude<PaneId, "auspice">;

export const INITIAL_PANE_STATE: PaneOpenState = {
  wide: { rail: true, inspector: true, auspice: true },
  overlay: { rail: false, inspector: false, auspice: false },
};

export const PANE_SHORTCUTS: Record<ShellPaneId, string> = {
  rail: "Ctrl+B or Cmd+B",
  inspector: "Ctrl+Alt+B or Cmd+Option+B",
};

const NON_TEXT_INPUTS: ReadonlySet<string> = new Set([
  "checkbox",
  "radio",
  "button",
  "submit",
  "reset",
  "range",
  "color",
  "file",
]);

export function paneIsOpen(state: PaneOpenState, pane: PaneId, overlay: boolean): boolean {
  return overlay ? state.overlay[pane] : state.wide[pane];
}

export function withPaneOpen(state: PaneOpenState, pane: PaneId, overlay: boolean, open: boolean): PaneOpenState {
  return overlay
    ? { ...state, overlay: { ...state.overlay, [pane]: open } }
    : { ...state, wide: { ...state.wide, [pane]: open } };
}

export function paneShortcut({
  code,
  ctrlKey,
  metaKey,
  altKey,
  shiftKey,
  repeat,
  isComposing,
}: PaneShortcutKey): ShellPaneId | null {
  if (code !== "KeyB" || ctrlKey === metaKey || shiftKey || repeat || isComposing) {
    return null;
  }

  return altKey ? "inspector" : "rail";
}

export function isTextEntry(element: TextEntryCandidate | null): boolean {
  if (element === null) {
    return false;
  }

  if (element.isContentEditable || element.tagName === "TEXTAREA" || element.tagName === "SELECT") {
    return true;
  }

  return element.tagName === "INPUT" && !NON_TEXT_INPUTS.has(element.type ?? "text");
}

export interface PaneOpenState {
  wide: PaneOpen;
  overlay: PaneOpen;
}

export type PaneOpen = Readonly<Record<PaneId, boolean>>;

export type PaneShortcutKey = Pick<
  KeyboardEvent,
  "code" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey" | "repeat" | "isComposing"
>;

export interface TextEntryCandidate {
  tagName: string;
  isContentEditable: boolean;
  type?: string;
}
