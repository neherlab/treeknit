import { describe, expect, test } from "vitest";

import { escapeHtml, REFERENCE_HTML, REFERENCE_TEXT } from "../citation";

const DOI_URL = "https://doi.org/10.1371/journal.pcbi.1010394";

describe("citation of the TreeKnit paper", () => {
  test("writes the text reference in Chicago author-date style with the DOI link", () => {
    expect(REFERENCE_TEXT).toBe(
      `Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." PLOS Computational Biology 18:e1010394. ${DOI_URL}`,
    );
  });

  test("writes the rich reference as HTML with the journal in italics and escaped quotes", () => {
    expect(REFERENCE_HTML).toBe(
      `Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. &quot;TreeKnit: Inferring ancestral reassortment graphs of influenza viruses.&quot; <i>PLOS Computational Biology</i> 18:e1010394. <a href="${DOI_URL}">${DOI_URL}</a>`,
    );
  });

  test("escapes markup characters in HTML text", () => {
    expect(escapeHtml('Trees <and> "graphs" & more')).toBe("Trees &lt;and&gt; &quot;graphs&quot; &amp; more");
  });
});
