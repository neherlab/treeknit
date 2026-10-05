import { describe, expect, test } from "vitest";

import { itemAt } from "../lookup";

describe("itemAt", () => {
  test("returns the item at the index", () => {
    expect(itemAt(["a", "b"], 1, "leaf")).toBe("b");
  });

  test("names the kind, the index, and the count when the index is missing", () => {
    expect(() => itemAt(["a", "b"], 2, "leaf")).toThrow(
      new RangeError("The display data refers to leaf 2, but it has 2 of them"),
    );
  });
});
