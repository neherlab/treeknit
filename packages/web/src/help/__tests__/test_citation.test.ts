import { describe, expect, test } from "vitest";

import {
  type Author,
  bibtexEntry,
  inlineAuthors,
  paperLinks,
  type Publication,
  referenceHtml,
  referenceSegments,
  referenceText,
  TREEKNIT_PUBLICATION,
} from "../citation";

const DOI_URL = "https://doi.org/10.1371/journal.pcbi.1010394";

describe("citation of the TreeKnit paper", () => {
  test("writes the text reference in Chicago author-date style with the DOI link", () => {
    expect(referenceText(TREEKNIT_PUBLICATION)).toBe(
      `Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." PLOS Computational Biology 18:e1010394. ${DOI_URL}`,
    );
  });

  test("marks the journal as italic and the DOI URL as a link", () => {
    const segments = referenceSegments(TREEKNIT_PUBLICATION);

    expect(segments.filter((segment) => segment.italic === true).map((segment) => segment.text)).toStrictEqual([
      "PLOS Computational Biology",
    ]);
    expect(segments.filter((segment) => segment.href !== undefined)).toStrictEqual([{ text: DOI_URL, href: DOI_URL }]);
  });

  test("writes the rich reference as HTML with the journal in italics and escaped quotes", () => {
    expect(referenceHtml(TREEKNIT_PUBLICATION)).toBe(
      `Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. &quot;TreeKnit: Inferring ancestral reassortment graphs of influenza viruses.&quot; <i>PLOS Computational Biology</i> 18:e1010394. <a href="${DOI_URL}">${DOI_URL}</a>`,
    );
  });

  test("escapes markup characters in the HTML reference", () => {
    const publication = { ...TREEKNIT_PUBLICATION, title: "Trees <and> graphs & more" };

    expect(referenceHtml(publication)).toContain("&quot;Trees &lt;and&gt; graphs &amp; more.&quot;");
  });

  test("writes a BibTeX entry that keeps the capitals of the tool name", () => {
    expect(bibtexEntry(TREEKNIT_PUBLICATION)).toBe(
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
        `  url = {${DOI_URL}},`,
        "}",
      ].join("\n"),
    );
  });

  test("links the open-access article through its DOI, then the PDF, PubMed Central, and PubMed", () => {
    expect(paperLinks(TREEKNIT_PUBLICATION)).toStrictEqual([
      {
        id: "article",
        label: "Article",
        description: "Read the article in PLOS Computational Biology (DOI 10.1371/journal.pcbi.1010394)",
        href: DOI_URL,
      },
      {
        id: "pdf",
        label: "PDF",
        description: "Open the PDF of the article from PLOS Computational Biology",
        href: "https://journals.plos.org/ploscompbiol/article/file?id=10.1371/journal.pcbi.1010394&type=printable",
      },
      {
        id: "pmc",
        label: "PubMed Central",
        description: "Read the full text in PubMed Central (PMC9447925)",
        href: "https://pmc.ncbi.nlm.nih.gov/articles/PMC9447925/",
      },
      {
        id: "pubmed",
        label: "PubMed",
        description: "Open the PubMed record (PMID 35984845)",
        href: "https://pubmed.ncbi.nlm.nih.gov/35984845/",
      },
    ]);
  });
});

describe("author list of the Chicago reference", () => {
  test.each([
    {
      case: "one author whose name ends in an initial",
      authors: [author("Vaughan", "Timothy G.")],
      start: 'Vaughan, Timothy G. 2022. "',
    },
    { case: "two authors", authors: [author("Lee"), author("Kim", "Bo")], start: "Lee, Ann and Bo Kim. 2022." },
    {
      case: "six authors, all listed",
      authors: [author("A"), author("B"), author("C"), author("D"), author("E"), author("F")],
      start: "A, Ann, Ann B, Ann C, Ann D, Ann E, and Ann F. 2022.",
    },
    {
      case: "seven authors, shortened to three and et al.",
      authors: [author("A"), author("B"), author("C"), author("D"), author("E"), author("F"), author("G")],
      start: "A, Ann, Ann B, Ann C, et al. 2022.",
    },
  ] satisfies { case: string; authors: Author[]; start: string }[])("writes $case", ({ authors, start }) => {
    expect(referenceText(withAuthors(authors)).slice(0, start.length)).toBe(start);
  });
});

describe("inline author form", () => {
  test.each([
    { case: "one author by family name", authors: [author("Lee")], expected: "Lee" },
    { case: "two authors joined with and", authors: [author("Lee"), author("Kim")], expected: "Lee and Kim" },
    {
      case: "three or more authors shortened to et al.",
      authors: [author("Lee"), author("Kim"), author("Ng")],
      expected: "Lee et al.",
    },
  ] satisfies { case: string; authors: Author[]; expected: string }[])("names $case", ({ authors, expected }) => {
    expect(inlineAuthors(withAuthors(authors))).toBe(expected);
  });
});

function withAuthors([first, ...others]: readonly Author[]): Publication {
  if (first === undefined) {
    throw new Error("A publication needs at least one author");
  }

  return { ...TREEKNIT_PUBLICATION, authors: [first, ...others] };
}

function author(family: string, given = "Ann"): Author {
  return { family, given };
}
