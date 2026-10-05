export interface TextPosition {
  line: number;
  column: number;
}

export interface TextRange {
  start: TextPosition;
  end: TextPosition;
}

export interface CodeLine {
  number: number;
  before: string;
  marked: string | undefined;
  after: string;
}

export function characterRange(position: TextPosition): TextRange {
  return { start: position, end: { line: position.line, column: position.column + 1 } };
}

export function codeLines(code: string, range: TextRange | undefined): CodeLine[] {
  return code.split("\n").map((text, index) => {
    const number = index + 1;
    const line = text.endsWith("\r") ? text.slice(0, -1) : text;

    if (range === undefined) {
      return { number, before: line, marked: undefined, after: "" };
    }

    return markLine(number, line, range);
  });
}

function markLine(number: number, line: string, range: TextRange): CodeLine {
  const { start } = range;
  const end = isBefore(range.end, start) ? start : range.end;
  const endsOnLineStart = end.line > start.line && end.column <= 1;
  const lastLine = endsOnLineStart ? end.line - 1 : end.line;

  if (number < start.line || number > Math.max(start.line, lastLine)) {
    return { number, before: line, marked: undefined, after: "" };
  }

  const characters = Array.from(line);
  const from = number === start.line ? clampColumn(start.column, characters.length) : 0;
  const to = number === end.line && !endsOnLineStart ? clampColumn(end.column, characters.length) : characters.length;
  const until = Math.max(from, to);

  return {
    number,
    before: characters.slice(0, from).join(""),
    marked: characters.slice(from, until).join(""),
    after: characters.slice(until).join(""),
  };
}

function isBefore(position: TextPosition, other: TextPosition): boolean {
  return position.line < other.line || (position.line === other.line && position.column < other.column);
}

function clampColumn(column: number, length: number): number {
  return Math.min(Math.max(column - 1, 0), length);
}
