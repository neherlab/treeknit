import { describe, expect, test } from "vitest";

import { analysisKeys } from "../queries";

describe("analysis query keys", () => {
  test("nest every session query under the key of its session", () => {
    const session = analysisKeys.session(3);

    const keys = [
      analysisKeys.files(3),
      analysisKeys.commandLine(3),
      analysisKeys.pairView(3, 1, "resolved", "div"),
      analysisKeys.argView(3, "depth"),
      analysisKeys.constellation(3),
    ];

    expect(keys.map((key) => key.slice(0, session.length))).toStrictEqual(keys.map(() => [...session]));
  });

  test("tell sessions, pairs, versions, and scales apart", () => {
    const keys = [
      analysisKeys.pairView(1, 0, "resolved", "div"),
      analysisKeys.pairView(2, 0, "resolved", "div"),
      analysisKeys.pairView(1, 1, "resolved", "div"),
      analysisKeys.pairView(1, 0, "input", "div"),
      analysisKeys.pairView(1, 0, "resolved", "depth"),
    ];

    expect(new Set(keys.map((key) => JSON.stringify(key))).size).toBe(keys.length);
  });

  test("key stateless queries by their inputs", () => {
    expect([
      analysisKeys.inspectTree("ha", "(A,B);"),
      analysisKeys.settingsSchema(2, { gamma: 2 }),
      analysisKeys.validate({ trees: [] }),
    ]).toStrictEqual([
      ["inspectTree", "ha", "(A,B);"],
      ["settingsSchema", 2, { gamma: 2 }],
      ["validate", { trees: [] }],
    ]);
  });
});
