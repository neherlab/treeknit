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

  test("keeps a carriage return that ends no line, as Rust counts it as a character", () => {
    expect(codeLines("a\n\r", undefined)).toStrictEqual([
      { number: 1, before: "a", marked: undefined, after: "" },
      { number: 2, before: "\r", marked: undefined, after: "" },
    ]);
    expect(codeLines("a\rb", characterRange({ line: 1, column: 3 }))).toStrictEqual([
      { number: 1, before: "a\r", marked: "b", after: "" },
    ]);
  });

  test("gives one empty line for empty code", () => {
    expect(codeLines("", undefined)).toStrictEqual([{ number: 1, before: "", marked: undefined, after: "" }]);
  });

  test("marks an empty caret in empty code", () => {
    expect(codeLines("", characterRange({ line: 1, column: 1 }))).toStrictEqual([
      { number: 1, before: "", marked: "", after: "" },
    ]);
  });

  test.each([0, -1])("marks an empty caret at the line start for column %i", (column) => {
    expect(codeLines("abc", characterRange({ line: 1, column }))).toStrictEqual([
      { number: 1, before: "", marked: "", after: "abc" },
    ]);
  });

  test.each([0, -1])("marks nothing for a range on line %i", (line) => {
    expect(codeLines("abc\ndef", characterRange({ line, column: 1 }))).toStrictEqual([
      { number: 1, before: "abc", marked: undefined, after: "" },
      { number: 2, before: "def", marked: undefined, after: "" },
    ]);
  });
});

describe("codeLines with the positions of Rust newick::line_column", () => {
  const UTF8 = new TextEncoder();
  const UTF8_DECODER = new TextDecoder();

  test.each([
    { text: "(A,B);", offset: 0, line: 1, column: 1 },
    { text: "(A,B);", offset: 3, line: 1, column: 4 },
    { text: "(A,\nB);", offset: 4, line: 2, column: 1 },
    { text: "(A,\r\nB);", offset: 7, line: 2, column: 3 },
    { text: "(é,ü);", offset: 4, line: 1, column: 4 },
    { text: "(é,ü);", offset: 7, line: 1, column: 6 },
    { text: "(A,\nB)", offset: 100, line: 2, column: 3 },
    { text: "(A,\n(B,C)D\n;", offset: 11, line: 3, column: 1 },
  ])("marks the character at byte $offset of $text", ({ text, offset, line, column }) => {
    const bytes = UTF8.encode(text);
    const before = UTF8_DECODER.decode(bytes.slice(0, offset));
    const after = UTF8_DECODER.decode(bytes.slice(offset));
    const next = Array.from(after)[0] ?? "";

    const marked = codeLines(text, characterRange({ line, column })).find((entry) => entry.marked !== undefined);

    expect(marked?.number).toBe(line);
    expect(marked?.before).toBe(before.split("\n").at(-1));
    expect(marked?.marked).toBe(next === "\n" || next === "\r" ? "" : next);
  });
});

describe("codeLines properties", () => {
  const position = fc.record({ line: fc.integer({ min: -1, max: 6 }), column: fc.integer({ min: -1, max: 12 }) });
  const range = fc.option(fc.record({ start: position, end: position }), { nil: undefined });
  const unit = fc.constantFrom("a", "é", "𝔸", ",", " ");
  const lineText = fc.string({ unit, maxLength: 8 });
  const lastLineText = fc.string({ unit, minLength: 1, maxLength: 8 });

  const code = fc
    .tuple(fc.array(lineText, { maxLength: 4 }), lastLineText)
    .map(([lines, last]) => [...lines, last].join("\n"));

  test("marks exactly the code points from the start up to the end of the range, as a cell model of the text", () => {
    fc.assert(
      fc.property(code, range, (text, highlight) => {
        expect(codeLines(text, highlight)).toStrictEqual(cellModel(text, highlight));
      }),
    );
  });
});

type Cell = readonly [line: number, column: number];

function cellModel(text: string, range: TextRange | undefined) {
  const lines = text.split("\n").map((line) => Array.from(line));
  const at = (line: number, column: number): Cell => clampCell(lines, line, column);
  const from = range === undefined ? undefined : at(range.start.line, range.start.column);
  const to = range === undefined ? undefined : at(range.end.line, range.end.column);

  return lines.map((characters, index) => {
    const number = index + 1;
    const lineEndCell: Cell = [number, characters.length + 1];

    const covered =
      from !== undefined &&
      to !== undefined &&
      (number === range?.start.line || (!isBeforeCell(lineEndCell, from) && isBeforeCell([number, 1], to)));

    if (!covered) {
      return { number, before: characters.join(""), marked: undefined, after: "" };
    }

    const cells = characters.map((character, column): [string, Cell] => [character, [number, column + 1]]);

    const pick = (keep: (cell: Cell) => boolean) =>
      cells.flatMap(([character, cell]) => (keep(cell) ? [character] : []));

    return {
      number,
      before: pick((cell) => isBeforeCell(cell, from)).join(""),
      marked: pick((cell) => !isBeforeCell(cell, from) && isBeforeCell(cell, to)).join(""),
      after: pick((cell) => !isBeforeCell(cell, from) && !isBeforeCell(cell, to)).join(""),
    };
  });
}

function clampCell(lines: readonly (readonly string[])[], line: number, column: number): Cell {
  if (line < 1) {
    return [0, 0];
  }

  const characters = lines[line - 1];

  return characters === undefined
    ? [lines.length + 1, 0]
    : [line, Math.min(Math.max(column, 1), characters.length + 1)];
}

function isBeforeCell([line, column]: Cell, [otherLine, otherColumn]: Cell): boolean {
  return line < otherLine || (line === otherLine && column < otherColumn);
}
