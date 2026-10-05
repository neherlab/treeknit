import { describe, expect, test } from "vitest";

import { addFailure, isSessionFileName, readFailure, sessionFailure } from "../treeFiles";

describe("tree files", () => {
  test("routes JSON files to the session file reader and everything else to the trees", () => {
    expect(
      ["treeknit_request.json", "SESSION.JSON", "ha.nwk", "na.tree", "json.txt"].map(isSessionFileName),
    ).toStrictEqual([true, true, false, false, false]);
  });

  test("says what failed and what to do", () => {
    expect({
      read: readFailure("ha.nwk", new Error("permission denied")),
      session: sessionFailure("notes.json", new Error("missing field `trees`")),
      add: addFailure("worker stopped"),
    }).toStrictEqual({
      add: "The trees could not be added: worker stopped. Try again.",
      read: "ha.nwk could not be read: permission denied. Check the file and add it again.",
      session: "notes.json is not a TreeKnit session file: missing field `trees`. Open a treeknit_request.json file.",
    });
  });
});
