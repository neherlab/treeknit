import { describe, expect, test } from "vitest";

import {
  addFailure,
  batchProblem,
  batchRejection,
  isSessionFileName,
  readFailures,
  sessionFailure,
  sessionReadFailure,
} from "../treeFiles";

describe("tree files", () => {
  test("routes JSON files to the session file reader and everything else to the trees", () => {
    expect(
      ["treeknit_session.json", "SESSION.JSON", "ha.nwk", "na.tree", "json.txt"].map(isSessionFileName),
    ).toStrictEqual([true, true, false, false, false]);
  });

  test("says what failed and what to do", () => {
    expect({
      read: readFailures([{ name: "ha.nwk", cause: new Error("permission denied") }], 0),
      sessionRead: sessionReadFailure("s.json", new Error("permission denied")),
      session: sessionFailure(
        "notes.json",
        new Error("not a TreeKnit session file: missing field `trees` at line 1 column 2"),
      ),
      add: addFailure("worker stopped"),
    }).toStrictEqual({
      add: "The trees could not be added: worker stopped. Try again.",
      read: "ha.nwk could not be read: permission denied. Check the file and add it again.",
      sessionRead: "s.json could not be read: permission denied. Check the file and open it again.",
      session: "notes.json: not a TreeKnit session file: missing field `trees` at line 1 column 2",
    });
  });

  test("accepts trees alone or one session file alone", () => {
    expect({
      trees: batchRejection([], 3),
      session: batchRejection(["treeknit_session.json"], 0),
    }).toStrictEqual({ trees: null, session: null });
  });

  test("rejects a session file together with trees, naming the session file", () => {
    expect(batchRejection(["treeknit_session.json"], 2)).toBe(
      "treeknit_session.json was not opened together with trees, because a session file replaces the trees. Nothing was added. Open the session file on its own, or add only the trees.",
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

  test("states how many trees were added", () => {
    const failure = [{ name: "ha.nwk", cause: new Error("permission denied") }];

    expect({ one: readFailures(failure, 1), three: readFailures(failure, 3) }).toStrictEqual({
      one: "ha.nwk could not be read: permission denied. Check the file and add it again. 1 tree was added.",
      three: "ha.nwk could not be read: permission denied. Check the file and add it again. 3 trees were added.",
    });
  });

  test("joins the read failures and the add failure of one batch, and counts no tree when adding failed", () => {
    const failure = [{ name: "ha.nwk", cause: new Error("permission denied") }];
    const added = "The trees could not be added: worker stopped. Try again.";

    expect({
      none: batchProblem([], 2, null),
      read: batchProblem(failure, 2, null),
      add: batchProblem([], 2, added),
      both: batchProblem(failure, 2, added),
    }).toStrictEqual({
      none: null,
      read: "ha.nwk could not be read: permission denied. Check the file and add it again. 2 trees were added.",
      add: added,
      both: `ha.nwk could not be read: permission denied. Check the file and add it again. ${added}`,
    });
  });
});
