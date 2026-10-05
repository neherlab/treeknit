import { describe, expect, test } from "vitest";

import { analysisKeys, currentData, sameTexts, sharesScope } from "../queries";

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

  test("key stateless queries by their inputs, with tree texts by their text ids", () => {
    expect([
      analysisKeys.inspectTree(4),
      analysisKeys.overlap([4, 7]),
      analysisKeys.settingsSchema(2, { gamma: 2 }),
      analysisKeys.validate([4, 7], ["ha", "na"], { gamma: 2 }),
      analysisKeys.validate([], [], undefined),
    ]).toStrictEqual([
      ["inspectTree", 4],
      ["overlap", [4, 7]],
      ["settingsSchema", 2, { gamma: 2 }],
      ["validate", [4, 7], ["ha", "na"], { gamma: 2 }],
      ["validate", [], [], null],
    ]);
  });
});

describe("view placeholders", () => {
  test("keep the previous pair view only within the same session and pair", () => {
    const scope = analysisKeys.pairViewScope(1, 0);

    expect([
      sharesScope(analysisKeys.pairView(1, 0, "input", "div"), scope),
      sharesScope(analysisKeys.pairView(1, 0, "resolved", "depth"), scope),
      sharesScope(analysisKeys.pairView(1, 1, "resolved", "depth"), scope),
      sharesScope(analysisKeys.pairView(2, 0, "resolved", "depth"), scope),
      sharesScope(analysisKeys.argView(1, "div"), scope),
      sharesScope(undefined, scope),
    ]).toStrictEqual([true, true, false, false, false, false]);
  });

  test("keep the previous ARG view only within the same session", () => {
    const scope = analysisKeys.argViewScope(1);

    expect([
      sharesScope(analysisKeys.argView(1, "div"), scope),
      sharesScope(analysisKeys.argView(1, "depth"), scope),
      sharesScope(analysisKeys.argView(2, "div"), scope),
      sharesScope(analysisKeys.pairView(1, 0, "resolved", "div"), scope),
    ]).toStrictEqual([true, true, false, false]);
  });
});

describe("validation placeholders", () => {
  const previous = analysisKeys.validate([1, 2], ["ha", "na"], { gamma: 2 });

  test("keep the previous errors only while each index holds the same tree text", () => {
    expect({
      renamed: sameTexts(analysisKeys.validate([1, 2], ["segment 4", "na"], { gamma: 2 }), [1, 2]),
      settings: sameTexts(analysisKeys.validate([1, 2], ["ha", "na"], { gamma: 3 }), [1, 2]),
      reordered: sameTexts(previous, [2, 1]),
      removed: sameTexts(previous, [1]),
      edited: sameTexts(previous, [1, 3]),
      none: sameTexts(undefined, [1, 2]),
    }).toStrictEqual({ renamed: true, settings: true, reordered: false, removed: false, edited: false, none: false });
  });
});

describe("currentData", () => {
  test("hides the data of the previous key while the next key loads", () => {
    expect([
      currentData({ data: "previous pair", isPlaceholderData: true }),
      currentData({ data: "this pair", isPlaceholderData: false }),
    ]).toStrictEqual([undefined, "this pair"]);
  });
});
