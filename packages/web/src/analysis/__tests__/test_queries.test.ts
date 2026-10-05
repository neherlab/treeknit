import { describe, expect, test } from "vitest";

import { analysisKeys, sameTreeTexts, sharesScope } from "../queries";

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

    expect(new Set(keys.map((key) => JSON.stringify(key.slice(0, session.length))))).toStrictEqual(
      new Set([JSON.stringify(session)]),
    );
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
      analysisKeys.inspectTree("(A,B);"),
      analysisKeys.overlap(["(A,B);", "(A,C);"]),
      analysisKeys.settingsSchema(2, { gamma: 2 }),
      analysisKeys.validate({ trees: [] }),
    ]).toStrictEqual([
      ["inspectTree", "(A,B);"],
      ["overlap", ["(A,B);", "(A,C);"]],
      ["settingsSchema", 2, { gamma: 2 }],
      ["validate", { trees: [] }],
    ]);
  });
});

describe("view placeholders", () => {
  test("keep the previous pair view only within the same session and pair", () => {
    const next = analysisKeys.pairView(1, 0, "resolved", "depth");

    expect([
      sharesScope(analysisKeys.pairView(1, 0, "input", "div"), next, 4),
      sharesScope(analysisKeys.pairView(1, 1, "resolved", "depth"), next, 4),
      sharesScope(analysisKeys.pairView(2, 0, "resolved", "depth"), next, 4),
      sharesScope(undefined, next, 4),
    ]).toStrictEqual([true, false, false, false]);
  });

  test("keep the previous ARG view only within the same session", () => {
    const next = analysisKeys.argView(1, "depth");

    expect([
      sharesScope(analysisKeys.argView(1, "div"), next, 3),
      sharesScope(analysisKeys.argView(2, "div"), next, 3),
    ]).toStrictEqual([true, false]);
  });
});

describe("validation placeholders", () => {
  const HA = { label: "ha", newick: "(A,B);" };
  const NA = { label: "na", newick: "(A,C);" };

  test("keep the previous errors only while each index holds the same tree text", () => {
    expect({
      renamed: sameTreeTexts([HA, NA], [{ ...HA, label: "segment 4" }, NA]),
      same: sameTreeTexts([HA, NA], [HA, NA]),
      reordered: sameTreeTexts([HA, NA], [NA, HA]),
      removed: sameTreeTexts([HA, NA], [HA]),
      edited: sameTreeTexts([HA, NA], [HA, { ...NA, newick: "(A,D);" }]),
    }).toStrictEqual({ renamed: true, same: true, reordered: false, removed: false, edited: false });
  });
});
