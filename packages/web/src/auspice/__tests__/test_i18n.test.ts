import { describe, expect, test } from "vitest";

import { AUSPICE_I18N } from "../i18n";

describe("auspice translations", () => {
  test.each([
    ["Showing {{x}} of {{y}} genomes", { x: 3, y: 5 }, "Showing 3 of 5 leaves"],
    [
      "Showing {{x}} of {{y}} genomes sampled between {{from}} and {{to}}",
      { x: 3, y: 5, from: "2017", to: "2018" },
      "Showing 3 of 5 leaves sampled between 2017 and 2018",
    ],
  ])("count leaves instead of genomes: %s", (key, values, expected) => {
    expect(AUSPICE_I18N.t(key, values)).toBe(expected);
  });

  test("point a click to the details of the workspace instead of auspice's own panel", () => {
    expect(AUSPICE_I18N.t("Click on tip to display more info")).toBe("Click to show the details of this leaf");
  });

  test("keep the other auspice texts", () => {
    expect([AUSPICE_I18N.t("Phylogeny"), AUSPICE_I18N.t("Zoom to Root")]).toStrictEqual(["Phylogeny", "Zoom to Root"]);
  });
});
