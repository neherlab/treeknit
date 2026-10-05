import { describe, expect, test } from "vitest";

import { referenceHtml, TREEKNIT_PUBLICATION } from "../../help/citation";
import { figureCaption } from "../figure";

describe("figureCaption", () => {
  test("names the trees, the shown leaves, and the publications, with markup in labels escaped", () => {
    const caption = figureCaption({
      title: "TreeKnit: <ha> and na",
      summary: "Showing 3 of 5 leaves.",
      publications: [
        {
          author: "Hadfield et al",
          title: "Nextstrain",
          journal: "Bioinformatics",
          year: "2018",
          href: "https://doi.org/x?a=1&b=2",
        },
      ],
    });

    expect(caption).toStrictEqual([
      "TreeKnit: &lt;ha&gt; and na",
      "Showing 3 of 5 leaves.",
      "",
      "Relevant publications:",
      referenceHtml(TREEKNIT_PUBLICATION),
      '<a href="https://doi.org/x?a=1&amp;b=2">Hadfield et al, Nextstrain, Bioinformatics (2018)</a>',
    ]);
  });
});
