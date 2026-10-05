import { describe, expect, test } from "vitest";

import { reorder, type ReorderMove } from "../reorder";

const IDS = ["a", "b", "c", "d"] as const;

function identity(id: string) {
  return id;
}

function move(keys: string[], target: string, position: ReorderMove["position"]): ReorderMove {
  return { keys: new Set(keys), target, position };
}

describe("reorder", () => {
  test.each([
    {
      name: "one key before an earlier item",
      keys: ["c"],
      target: "a",
      position: "before",
      expected: ["c", "a", "b", "d"],
    },
    { name: "one key after a later item", keys: ["a"], target: "c", position: "after", expected: ["b", "c", "a", "d"] },
    {
      name: "one key after the last item",
      keys: ["b"],
      target: "d",
      position: "after",
      expected: ["a", "c", "d", "b"],
    },
    {
      name: "one key before its own successor",
      keys: ["b"],
      target: "c",
      position: "before",
      expected: ["a", "b", "c", "d"],
    },
    {
      name: "several keys keep their list order",
      keys: ["d", "a"],
      target: "c",
      position: "before",
      expected: ["b", "a", "d", "c"],
    },
  ] as const)("moves $name", ({ keys, target, position, expected }) => {
    expect(reorder(IDS, identity, move([...keys], target, position))).toStrictEqual(expected);
  });

  test("leaves the order unchanged when the target is one of the moved keys", () => {
    expect(reorder(IDS, identity, move(["b", "c"], "c", "after"))).toStrictEqual([...IDS]);
  });

  test("leaves the order unchanged when the target is missing", () => {
    expect(reorder(IDS, identity, move(["a"], "z", "before"))).toStrictEqual([...IDS]);
  });

  test("returns a new array and keeps the input", () => {
    const items = [...IDS];

    const result = reorder(items, identity, move(["d"], "a", "before"));

    expect(items).toStrictEqual([...IDS]);
    expect(result).not.toBe(items);
  });
});
