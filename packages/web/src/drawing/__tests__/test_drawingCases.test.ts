import { describe, expect, test } from "vitest";

import { argColumn } from "../../arg/geometry";
import { labelsVisible } from "../../canvas/labels";
import type { Column } from "../../canvas/projection";
import { tanglegramColumns } from "../../tanglegram/columns";
import { ribbonsShown } from "../../tanglegram/layers";
import table from "./__fixtures__/drawing_cases.json";
import { exampleDrawingRules } from "./fixtures";

const RULES = exampleDrawingRules();

describe("the drawing formulas of the web app", () => {
  test("apply the rules of the table of the Rust reference", () => {
    expect(table.rules).toStrictEqual(RULES);
  });

  test.each(table.columns)(
    "lay out $widthPx px with a $longestLabelPx px label as the Rust reference does",
    ({ widthPx, longestLabelPx, tanglegram, argColumn: arg }) => {
      const columns = tanglegramColumns(widthPx, longestLabelPx, RULES);

      expect({
        tanglegram: {
          left: span(columns.left),
          leftLabels: span(columns.leftLabels),
          links: span(columns.links),
          rightLabels: span(columns.rightLabels),
          right: span(columns.right),
        },
        argColumn: span(argColumn(widthPx, longestLabelPx, RULES)),
      }).toStrictEqual({ tanglegram, argColumn: arg });
    },
  );

  test.each(table.rows)(
    "at $rowPx px per row show labels and ribbons as the Rust reference does",
    ({ rowPx, labelsShown, ribbonsShown: ribbons }) => {
      expect({
        labelsShown: labelsVisible("auto", rowPx, RULES),
        ribbonsShown: ribbonsShown(rowPx, RULES),
      }).toStrictEqual({ labelsShown, ribbonsShown: ribbons });
    },
  );

  test("mirror the right tree of a tanglegram only", () => {
    const columns = tanglegramColumns(1232, 80, RULES);

    expect(
      [columns.left, columns.leftLabels, columns.links, columns.rightLabels, columns.right].map(
        ({ mirrored }) => mirrored,
      ),
    ).toStrictEqual([false, false, false, false, true]);
  });
});

function span({ start, end }: Column): Pick<Column, "start" | "end"> {
  return { start, end };
}
