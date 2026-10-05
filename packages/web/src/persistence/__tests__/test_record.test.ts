import { describe, expect, test } from "vitest";

import { nextGeneration, persistenceMessageSchema, storedRecordSchema } from "../record";

describe("stored workspace record", () => {
  test("accepts a workspace with every tree source kind", () => {
    const record = {
      kind: "workspace",
      version: 1,
      generation: 3,
      sessionFile: "{}",
      sources: [
        { kind: "file", name: "ha.nwk" },
        { kind: "paste" },
        { kind: "example", name: "na.nwk" },
        { kind: "session" },
      ],
    };

    expect(storedRecordSchema.parse(record)).toStrictEqual(record);
  });

  test("accepts an off marker without trees", () => {
    expect(storedRecordSchema.parse({ kind: "off", version: 1, generation: 0 })).toStrictEqual({
      kind: "off",
      version: 1,
      generation: 0,
    });
  });

  test("rejects records with a missing or other version, a missing or invalid generation, an unknown source, or an unknown kind", () => {
    const invalid = [
      { kind: "off", generation: 1 },
      { kind: "off", version: 2, generation: 1 },
      { kind: "off", version: 1 },
      { kind: "off", version: 1, generation: -1 },
      { kind: "off", version: 1, generation: 1.5 },
      { kind: "workspace", version: 1, generation: 1, sessionFile: "{}", sources: [{ kind: "url", name: "x" }] },
      { kind: "workspace", version: 1, generation: 1, sessionFile: "{}", sources: [{ kind: "file" }] },
      { kind: "workspace", version: 1, generation: 1, sources: [] },
      { kind: "deleted", version: 1, generation: 1 },
      "a string",
    ];

    expect(invalid.map((value) => storedRecordSchema.safeParse(value).success)).toStrictEqual(invalid.map(() => false));
  });

  test("accepts channel messages for on and off with a generation only", () => {
    expect(
      [
        { state: "on", generation: 2 },
        { state: "off", generation: 3 },
        { state: "toggle", generation: 1 },
        { state: "on" },
      ].map((value) => persistenceMessageSchema.safeParse(value).success),
    ).toStrictEqual([true, true, false, false]);
  });

  test("numbers the next generation after the stored one, starting at 1", () => {
    expect([nextGeneration(undefined), nextGeneration({ kind: "off", version: 1, generation: 4 })]).toStrictEqual([
      1, 5,
    ]);
  });
});
