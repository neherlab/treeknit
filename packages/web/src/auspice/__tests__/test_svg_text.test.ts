import {
  htmlToSvg,
  type SvgSourceElement,
  type SvgSourceRect,
  type SvgSourceText,
} from "auspice/src/components/download/domToSvg";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

const CHAR_WIDTH = 8;

const LINE_HEIGHT = 20;

const FONT_SIZE = 14;

const ELEMENT_NODE = 1;

const TEXT_NODE = 3;

const STYLE = {
  display: "block",
  visibility: "visible",
  opacity: "1",
  color: "rgb(0, 0, 0)",
  fontFamily: "lato",
  fontSize: `${FONT_SIZE}px`,
  fontWeight: "400",
  fontStyle: "normal",
};

const CITATION: readonly Inline[] = [
  { text: "Cite " },
  { tag: "i", text: "Journal" },
  { text: " 18:e1010394. Published 2022. " },
  { tag: "a", text: "https://doi.org/x" },
];

const TEXT_TAG = /<text x="([^"]+)" y="([^"]+)"[^>]*>([^<]*)<\/text>/gu;

describe("auspice SVG text of a caption line", () => {
  beforeEach(() => {
    vi.stubGlobal("Element", FakeElement);
    vi.stubGlobal("Node", { ELEMENT_NODE, TEXT_NODE });
    vi.stubGlobal("window", { getComputedStyle: () => STYLE });
    vi.stubGlobal("document", { createRange: () => new FakeRange() });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test.each<{ name: string; columns: number; expected: SvgText[] }>([
    {
      name: "the text after an inline element starts at its first visible character",
      columns: 100,
      expected: [
        { x: 0, y: 11.2, text: "Cite " },
        { x: 5 * CHAR_WIDTH, y: 11.2, text: "Journal" },
        { x: 13 * CHAR_WIDTH, y: 11.2, text: "18:e1010394. Published 2022. " },
        { x: 42 * CHAR_WIDTH, y: 11.2, text: "https://doi.org/x" },
      ],
    },
    {
      name: "the first of several lines after an inline element starts at its first visible character",
      columns: 30,
      expected: [
        { x: 0, y: 11.2, text: "Cite " },
        { x: 5 * CHAR_WIDTH, y: 11.2, text: "Journal" },
        { x: 13 * CHAR_WIDTH, y: 11.2, text: "18:e1010394. " },
        { x: 0, y: 31.2, text: "Published 2022. " },
        { x: 0, y: 51.2, text: "https://doi.org/x" },
      ],
    },
  ])("$name", ({ columns, expected }) => {
    const { markup } = htmlToSvg(paragraph(CITATION, columns));

    expect(svgTexts(markup)).toStrictEqual(expected);
  });
});

class FakeElement implements SvgSourceElement {
  readonly nodeType = ELEMENT_NODE;

  readonly childNodes: (FakeElement | FakeText)[] = [];

  readonly tagName: string;

  readonly #parent: FakeElement | undefined;

  readonly #href: string | undefined;

  constructor(tag: string, parent?: FakeElement, href?: string) {
    this.tagName = tag.toUpperCase();
    this.#parent = parent;
    this.#href = href;
    parent?.childNodes.push(this);
  }

  getBoundingClientRect(): SvgSourceRect {
    return boundingRect(this.#boxes());
  }

  closest(selector: string): { getAttribute(name: string): string | null } | null {
    if (selector === "a[href]" && this.#href !== undefined) {
      const href = this.#href;

      return { getAttribute: (name) => (name === "href" ? href : null) };
    }

    return this.#parent?.closest(selector) ?? null;
  }

  #boxes(): SvgSourceRect[] {
    return this.childNodes.flatMap((child) => (child instanceof FakeText ? child.boxes : child.#boxes()));
  }
}

class FakeText implements SvgSourceText {
  readonly nodeType = TEXT_NODE;

  readonly nodeValue: string;

  readonly parentElement: FakeElement;

  readonly boxes: readonly SvgSourceRect[];

  constructor(text: string, parent: FakeElement, boxes: readonly SvgSourceRect[]) {
    this.nodeValue = text;
    this.parentElement = parent;
    this.boxes = boxes;
  }
}

class FakeRange {
  #node: FakeText | undefined;

  #start = 0;

  #end = 0;

  selectNodeContents(node: FakeText): void {
    this.#node = node;
    this.#start = 0;
    this.#end = node.nodeValue.length;
  }

  setStart(node: FakeText, offset: number): void {
    this.#node = node;
    this.#start = offset;
  }

  setEnd(node: FakeText, offset: number): void {
    this.#node = node;
    this.#end = offset;
  }

  getClientRects(): SvgSourceRect[] {
    const boxes = this.#node?.boxes.slice(this.#start, this.#end) ?? [];
    const lines = Map.groupBy(boxes, (box) => box.top);

    return [...lines.values()].map(boundingRect);
  }
}

function paragraph(inlines: readonly Inline[], columns: number): FakeElement {
  const root = new FakeElement("div");
  const p = new FakeElement("p", root);
  const cursor = { column: 0, row: 0 };

  for (const { tag, text } of inlines) {
    const parent = tag === undefined ? p : new FakeElement(tag, p, tag === "a" ? text : undefined);

    parent.childNodes.push(new FakeText(text, parent, layOut(text, columns, cursor)));
  }

  return root;
}

function layOut(text: string, columns: number, cursor: { column: number; row: number }): SvgSourceRect[] {
  return [...text.matchAll(/\S+|\s+/gu)].flatMap(([run]) => {
    if (/\S/u.test(run) && cursor.column > 0 && cursor.column + run.length > columns) {
      cursor.column = 0;
      cursor.row += 1;
    }

    return Array.from({ length: run.length }, () => {
      const box = {
        left: cursor.column * CHAR_WIDTH,
        top: cursor.row * LINE_HEIGHT,
        width: CHAR_WIDTH,
        height: LINE_HEIGHT,
      };

      cursor.column += 1;

      return box;
    });
  });
}

function boundingRect(boxes: readonly SvgSourceRect[]): SvgSourceRect {
  const left = Math.min(...boxes.map((box) => box.left));
  const top = Math.min(...boxes.map((box) => box.top));
  const right = Math.max(...boxes.map((box) => box.left + box.width));
  const bottom = Math.max(...boxes.map((box) => box.top + box.height));

  return { left, top, width: right - left, height: bottom - top };
}

function svgTexts(markup: string): SvgText[] {
  return [...markup.matchAll(TEXT_TAG)].map(([, x, y, text]) => ({ x: Number(x), y: Number(y), text: text ?? "" }));
}

interface Inline {
  tag?: "a" | "i";
  text: string;
}

interface SvgText {
  x: number;
  y: number;
  text: string;
}
