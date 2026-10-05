import { describe, expect, test } from "vitest";

import { AUSPICE_I18N } from "../i18n";

describe("auspice translations", () => {
  test("leave the tree card untitled, because the zoom buttons of the left tree cover its title", () => {
    expect(AUSPICE_I18N.t("Phylogeny")).toBe("");
  });

  test("keep the other auspice texts", () => {
    expect(AUSPICE_I18N.t("Zoom to Root")).toBe("Zoom to Root");
  });
});
