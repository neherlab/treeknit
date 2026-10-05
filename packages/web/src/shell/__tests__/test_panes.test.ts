import { describe, expect, test } from "vitest";

import { INITIAL_PANE_STATE, isTextEntry, paneIsOpen, paneShortcut, withPaneOpen } from "../panes";

const KEY = {
  code: "KeyB",
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  repeat: false,
  isComposing: false,
};

describe("pane open state", () => {
  test("starts open where the panes take space and closed where they cover the content", () => {
    expect([
      paneIsOpen(INITIAL_PANE_STATE, "auspice", false),
      paneIsOpen(INITIAL_PANE_STATE, "auspice", true),
    ]).toStrictEqual([true, false]);
  });

  test("keeps the choice in overlay mode apart from the remembered choice for wide space", () => {
    const state = withPaneOpen(INITIAL_PANE_STATE, "rail", true, true);

    expect({ overlay: paneIsOpen(state, "rail", true), wide: state.wide }).toStrictEqual({
      overlay: true,
      wide: INITIAL_PANE_STATE.wide,
    });
  });

  test("changes only the pane it is given", () => {
    const state = withPaneOpen(INITIAL_PANE_STATE, "inspector", false, false);

    expect(state.wide).toStrictEqual({ rail: true, inspector: false, auspice: true });
  });
});

describe("paneShortcut", () => {
  test.each([
    ["Ctrl+B", { ...KEY, ctrlKey: true }, "rail"],
    ["Cmd+B", { ...KEY, metaKey: true }, "rail"],
    ["Ctrl+Alt+B", { ...KEY, ctrlKey: true, altKey: true }, "inspector"],
    ["Cmd+Option+B", { ...KEY, metaKey: true, altKey: true }, "inspector"],
    ["B alone", KEY, null],
    ["Ctrl+Cmd+B", { ...KEY, ctrlKey: true, metaKey: true }, null],
    ["Ctrl+Shift+B", { ...KEY, ctrlKey: true, shiftKey: true }, null],
    ["a held Ctrl+B", { ...KEY, ctrlKey: true, repeat: true }, null],
    ["Ctrl+N", { ...KEY, ctrlKey: true, code: "KeyN" }, null],
  ] as const)("%s toggles %s", (_name, key, pane) => {
    expect(paneShortcut(key)).toBe(pane);
  });
});

describe("isTextEntry", () => {
  test.each([
    ["a text field", { tagName: "INPUT", isContentEditable: false, type: "text" }, true],
    ["a number field", { tagName: "INPUT", isContentEditable: false, type: "number" }, true],
    ["a checkbox", { tagName: "INPUT", isContentEditable: false, type: "checkbox" }, false],
    ["a text area", { tagName: "TEXTAREA", isContentEditable: false }, true],
    ["an editable element", { tagName: "DIV", isContentEditable: true }, true],
    ["a button", { tagName: "BUTTON", isContentEditable: false }, false],
  ])("%s takes text: %s", (_name, element, expected) => {
    expect(isTextEntry(element)).toBe(expected);
  });

  test("nothing focused takes no text", () => {
    expect(isTextEntry(null)).toBe(false);
  });
});
