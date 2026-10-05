import { describe, expect, test } from "vitest";

import { formatCitation, shortAttribution, TREEKNIT_PUBLICATION } from "../citation";

describe("citation of the TreeKnit paper", () => {
  test("writes the text reference in Vancouver style", () => {
    expect(formatCitation(TREEKNIT_PUBLICATION, "text")).toBe(
      "Barrat-Charlaix P, Vaughan TG, Neher RA. TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. PLOS Computational Biology. 2022;18(8):e1010394. https://doi.org/10.1371/journal.pcbi.1010394",
    );
  });

  test("writes a BibTeX entry that keeps the capitals of the tool name", () => {
    expect(formatCitation(TREEKNIT_PUBLICATION, "bibtex")).toBe(
      [
        "@article{barratcharlaix2022treeknit,",
        "  author = {Barrat-Charlaix, Pierre and Vaughan, Timothy G. and Neher, Richard A.},",
        "  title = {{TreeKnit}: Inferring ancestral reassortment graphs of influenza viruses},",
        "  journal = {PLOS Computational Biology},",
        "  year = {2022},",
        "  volume = {18},",
        "  number = {8},",
        "  pages = {e1010394},",
        "  doi = {10.1371/journal.pcbi.1010394},",
        "  url = {https://doi.org/10.1371/journal.pcbi.1010394},",
        "}",
      ].join("\n"),
    );
  });

  test("writes a short attribution with the family names", () => {
    expect(shortAttribution(TREEKNIT_PUBLICATION)).toBe("Barrat-Charlaix, Vaughan & Neher, PLOS Comput Biol 2022");
  });
});
