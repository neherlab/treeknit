import { describe, expect, test } from "vitest";

import { addFailure, batchRejection, isSessionFileName, readFailures, sessionFailure } from "../treeFiles";

describe("tree files", () => {
  test("routes JSON files to the session file reader and everything else to the trees", () => {
    expect(
      ["treeknit_request.json", "SESSION.JSON", "ha.nwk", "na.tree", "json.txt"].map(isSessionFileName),
    ).toStrictEqual([true, true, false, false, false]);
  });

  test("says what failed and what to do", () => {
    expect({
      read: readFailures([{ name: "ha.nwk", cause: new Error("permission denied") }], 0),
      session: sessionFailure("notes.json", new Error("missing field `trees`")),
      add: addFailure("worker stopped"),
    }).toStrictEqual({
      add: "The trees could not be added: worker stopped. Try again.",
      read: "ha.nwk could not be read: permission denied. Check the file and add it again.",
      session: "notes.json is not a TreeKnit session file: missing field `trees`. Open a treeknit_request.json file.",
    });
  });

  test("accepts trees alone or one session file alone", () => {
    expect({
      trees: batchRejection([], 3),
      session: batchRejection(["treeknit_request.json"], 0),
    }).toStrictEqual({ trees: null, session: null });
  });

  test("rejects a session file together with trees, naming the session file", () => {
    expect(batchRejection(["treeknit_request.json"], 2)).toBe(
      "treeknit_request.json was not opened together with trees, because a session file replaces the trees. Nothing was added. Open the session file on its own, or add only the trees.",
    );
  });

  test("rejects several session files, naming each of them", () => {
    expect(batchRejection(["a.json", "b.json"], 0)).toBe(
      "a.json, b.json were not opened: open one session file at a time. Nothing was added.",
    );
  });

  test("reports nothing when every file was read", () => {
    expect(readFailures([], 3)).toBeNull();
  });

  test("names every file that could not be read, with its reason", () => {
    expect(
      readFailures(
        [
          { name: "ha.nwk", cause: new Error("permission denied") },
          { name: "na.nwk", cause: "file was removed" },
        ],
        0,
      ),
    ).toBe(
      "2 files could not be read: ha.nwk (permission denied); na.nwk (file was removed). Check the files and add them again.",
    );
  });

  test("states how many of the other files were added", () => {
    const failure = [{ name: "ha.nwk", cause: new Error("permission denied") }];

    expect({ one: readFailures(failure, 1), three: readFailures(failure, 3) }).toStrictEqual({
      one: "ha.nwk could not be read: permission denied. Check the file and add it again. The other file was added.",
      three:
        "ha.nwk could not be read: permission denied. Check the file and add it again. The other 3 files were added.",
    });
  });
});
