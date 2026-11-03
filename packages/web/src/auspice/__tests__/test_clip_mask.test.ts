import {
  type ClipMaskSelection,
  type ClipMaskTree,
  setClipMask,
} from "auspice/src/components/tree/phyloTree/renderers";
import { describe, expect, test } from "vitest";

const TREE_TOP = 40;

const TREE_BOTTOM = 400;

describe("auspice tree clip mask", () => {
  test.each<{ name: string; tree: TreePlacement; expected: ClipRect }>([
    {
      name: "a tree without branch labels in the margin keeps 5 px before the x range and 15 px after it",
      tree: { range: [60, 500], branchLabels: 0, mirrored: false },
      expected: { x: 55, y: 25, width: 460, height: 380 },
    },
    {
      name: "a tree with branch labels in the root margin widens to the left and keeps its outermost tips",
      tree: { range: [60, 500], branchLabels: 40, mirrored: false },
      expected: { x: 15, y: 25, width: 500, height: 380 },
    },
    {
      name: "a mirrored tree with branch labels in the root margin widens to the right and keeps its outermost tips",
      tree: { range: [500, 60], branchLabels: 40, mirrored: true },
      expected: { x: 55, y: 25, width: 500, height: 380 },
    },
  ])("$name", ({ tree, expected }) => {
    const svg = new RecordingSelection("svg");

    setClipMask.call(clipMaskTree(tree, svg));

    expect(svg.find("rect")?.attributes).toStrictEqual(expected);
  });

  test("a redraw after the branch labels move into the root margin widens the existing mask", () => {
    const svg = new RecordingSelection("svg");
    const before = clipMaskTree({ range: [60, 500], branchLabels: 0, mirrored: false }, svg);

    setClipMask.call(before);
    setClipMask.call({ ...before, margins: { branchLabels: 40 } });

    expect(svg.find("rect")?.attributes).toStrictEqual({ x: 15, y: 25, width: 500, height: 380 });
  });
});

class RecordingSelection implements ClipMaskSelection {
  readonly attributes: Record<string, number | string> = {};

  readonly #name: string;

  readonly #children: RecordingSelection[] = [];

  constructor(name: string) {
    this.#name = name;
  }

  append(name: string): RecordingSelection {
    const child = new RecordingSelection(name);

    this.#children.push(child);

    return child;
  }

  select(selector: string): RecordingSelection {
    const found = this.find(selector);

    if (found === undefined) {
      throw new Error(`no ${selector} in ${this.#name}`);
    }

    return found;
  }

  attr(name: string, value: number | string): RecordingSelection {
    this.attributes[name] = value;

    return this;
  }

  find(name: string): RecordingSelection | undefined {
    for (const child of this.#children) {
      const found = child.#name === name ? child : child.find(name);

      if (found !== undefined) {
        return found;
      }
    }

    return undefined;
  }
}

function clipMaskTree({ range, branchLabels, mirrored }: TreePlacement, svg: RecordingSelection): ClipMaskTree {
  const groups: ClipMaskTree["groups"] = {};

  return {
    id: "1",
    xScale: { range: () => [...range] },
    yScale: { range: () => [TREE_TOP, TREE_BOTTOM] },
    margins: { branchLabels },
    params: { orientation: mirrored ? [-1, 1] : [1, 1] },
    svg,
    groups,
  };
}

interface TreePlacement {
  range: readonly [number, number];
  branchLabels: number;
  mirrored: boolean;
}

interface ClipRect {
  x: number;
  y: number;
  width: number;
  height: number;
}
