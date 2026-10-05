import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import { characterRange, codeLines, type TextRange } from "../codeLines";

const NEWICK = "((A,B),(C,(D,X)));";

describe("codeLines", () => {
  test("keeps every line unmarked without a range", () => {
    expect(codeLines("(A,B);\n(A,C);", undefined)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
      { number: 2, before: "(A,C);", marked: undefined, after: "" },
    ]);
  });

  test("marks the character at a 1-based line and column", () => {
    const range = characterRange({ line: 1, column: 4 });

    expect(codeLines(NEWICK, range)).toStrictEqual([
      { number: 1, before: "((A", marked: ",", after: "B),(C,(D,X)));" },
    ]);
  });

  test("counts columns in Unicode code points, not UTF-16 units", () => {
    const range = characterRange({ line: 1, column: 3 });

    expect(codeLines("(\u{1D49C},B);", range)).toStrictEqual([
      { number: 1, before: "(\u{1D49C}", marked: ",", after: "B);" },
    ]);
  });

  test("marks an empty caret after the last character for a column past the end of the line", () => {
    const range = characterRange({ line: 1, column: 7 });

    expect(codeLines("(A,B)\n", range)).toStrictEqual([{ number: 1, before: "(A,B)", marked: "", after: "" }]);
  });

  test("drops the empty line after a final line end", () => {
    expect(codeLines("(A,B);\n", undefined)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
    ]);
    expect(codeLines("(A,B);\r\n", undefined)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
    ]);
  });

  test("keeps the empty line after a final line end when the range is on it", () => {
    const range = characterRange({ line: 2, column: 1 });

    expect(codeLines("(A,B);\n", range)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
      { number: 2, before: "", marked: "", after: "" },
    ]);
  });

  test("keeps only the last of several empty lines at the end", () => {
    expect(codeLines("(A,B);\n\n", undefined)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
      { number: 2, before: "", marked: undefined, after: "" },
    ]);
  });

  test("marks a range across lines, with the end column exclusive", () => {
    const range: TextRange = { start: { line: 1, column: 3 }, end: { line: 3, column: 2 } };

    expect(codeLines("abcd\nefgh\nijkl\nmnop", range)).toStrictEqual([
      { number: 1, before: "ab", marked: "cd", after: "" },
      { number: 2, before: "", marked: "efgh", after: "" },
      { number: 3, before: "", marked: "i", after: "jkl" },
      { number: 4, before: "mnop", marked: undefined, after: "" },
    ]);
  });

  test("leaves the end line unmarked when the range ends at its first column", () => {
    const range: TextRange = { start: { line: 1, column: 2 }, end: { line: 2, column: 1 } };

    expect(codeLines("abc\ndef", range)).toStrictEqual([
      { number: 1, before: "a", marked: "bc", after: "" },
      { number: 2, before: "def", marked: undefined, after: "" },
    ]);
  });

  test("marks an empty caret at the start of a range whose end precedes it", () => {
    const range: TextRange = { start: { line: 2, column: 3 }, end: { line: 1, column: 1 } };

    expect(codeLines("abc\ndef", range)).toStrictEqual([
      { number: 1, before: "abc", marked: undefined, after: "" },
      { number: 2, before: "de", marked: "", after: "f" },
    ]);
  });

  test("marks nothing for a range below the last line", () => {
    const range = characterRange({ line: 3, column: 1 });

    expect(codeLines("abc\ndef", range)).toStrictEqual([
      { number: 1, before: "abc", marked: undefined, after: "" },
      { number: 2, before: "def", marked: undefined, after: "" },
    ]);
  });

  test("drops the carriage return of CRLF line ends", () => {
    expect(codeLines("(A,B);\r\n(A,C);", undefined)).toStrictEqual([
      { number: 1, before: "(A,B);", marked: undefined, after: "" },
      { number: 2, before: "(A,C);", marked: undefined, after: "" },
    ]);
  });
});

describe("codeLines properties", () => {
  const position = fc.record({ line: fc.integer({ min: -1, max: 6 }), column: fc.integer({ min: -1, max: 12 }) });
  const range = fc.option(fc.record({ start: position, end: position }), { nil: undefined });

  const code = fc
    .array(fc.string({ unit: "grapheme", maxLength: 10 }), { maxLength: 5 })
    .map((lines) => lines.join("\n"));

  test("splits the code into its lines and preserves the text of each line", () => {
    fc.assert(
      fc.property(code, range, (text, highlight) => {
        const rebuilt = codeLines(text, highlight).map(({ before, marked, after }) => before + (marked ?? "") + after);

        const lines = text.split("\n");
        const lastLineMarked = codeLines(`${text}x`, highlight).at(-1)?.marked !== undefined;
        const dropsLastLine = lines.length > 1 && lines.at(-1) === "" && !lastLineMarked;

        expect(rebuilt).toStrictEqual(dropsLastLine ? lines.slice(0, -1) : lines);
      }),
    );
  });

  test("marks a contiguous block of lines that starts at the start line, clamped to the first line", () => {
    fc.assert(
      fc.property(code, range, (text, highlight) => {
        const marked = codeLines(text, highlight)
          .filter((line) => line.marked !== undefined)
          .map((line) => line.number);

        const expected = marked.length === 0 ? [] : [Math.max(highlight?.start.line ?? 1, 1)];

        expect(marked.slice(0, 1)).toStrictEqual(expected);
        expect(marked).toStrictEqual(marked.map((_, index) => (marked[0] ?? 0) + index));
      }),
    );
  });
});
