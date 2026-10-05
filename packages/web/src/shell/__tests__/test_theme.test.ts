import { describe, expect, test } from "vitest";

import { themeToggle } from "../theme";

describe("themeToggle", () => {
  test.each([
    { theme: "light", label: "Switch to dark theme", next: "dark" },
    { theme: "dark", label: "Switch to light theme", next: "light" },
    { theme: undefined, label: "Switch to dark theme", next: "dark" },
  ] as const)("offers $next for the $theme theme", ({ theme, label, next }) => {
    expect(themeToggle(theme)).toStrictEqual({ label, next });
  });
});
