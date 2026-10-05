const CHICAGO_MAX_LISTED_AUTHORS = 6;

const CHICAGO_ABBREVIATED_AUTHORS = 3;

const HTML_ESCAPES = new Map([
  ["&", "&amp;"],
  ["<", "&lt;"],
  [">", "&gt;"],
  ['"', "&quot;"],
]);

export const TREEKNIT_PUBLICATION: Publication = {
  authors: [
    { family: "Barrat-Charlaix", given: "Pierre" },
    { family: "Vaughan", given: "Timothy G." },
    { family: "Neher", given: "Richard A." },
  ],
  title: "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses",
  titleWord: "TreeKnit",
  journal: "PLOS Computational Biology",
  year: "2022",
  volume: "18",
  issue: "8",
  article: "e1010394",
  doi: "10.1371/journal.pcbi.1010394",
  pmid: "35984845",
  pmcid: "PMC9447925",
  pdf: "https://journals.plos.org/ploscompbiol/article/file?id=10.1371/journal.pcbi.1010394&type=printable",
};

export interface Publication {
  authors: readonly [Author, ...Author[]];
  title: string;
  titleWord: string;
  journal: string;
  year: string;
  volume: string;
  issue: string;
  article: string;
  doi: string;
  pmid: string;
  pmcid: string;
  pdf: string;
}

export interface Author {
  family: string;
  given: string;
}

export type CitationFormat = "text" | "bibtex";

export const CITATION_FORMATS: Record<CitationFormat, CitationFormatChoice> = {
  text: {
    format: "text",
    label: "Text",
    caption: "Citation",
    description: "Reference in Chicago author-date style, for documents",
    copyTooltip: "Copy the reference; documents keep the italics and the link",
  },
  bibtex: {
    format: "bibtex",
    label: "BibTeX",
    caption: "BibTeX",
    description: "BibTeX entry, for LaTeX and reference managers",
    copyTooltip: "Copy the BibTeX entry",
  },
};

export interface CitationFormatChoice {
  format: CitationFormat;
  label: string;
  caption: string;
  description: string;
  copyTooltip: string;
}

export interface ReferenceSegment {
  text: string;
  italic?: boolean;
  href?: string;
}

export type PaperLinkId = "article" | "pdf" | "pmc" | "pubmed";

export interface PaperLink {
  id: PaperLinkId;
  label: string;
  description: string;
  href: string;
}

export function doiUrl(publication: Publication): string {
  return `https://doi.org/${publication.doi}`;
}

export function referenceSegments(publication: Publication): ReferenceSegment[] {
  const doi = doiUrl(publication);

  return [
    { text: `${sentence(chicagoAuthors(publication))} ${publication.year}. "${sentence(publication.title)}" ` },
    { text: publication.journal, italic: true },
    { text: ` ${publication.volume}:${publication.article}. ` },
    { text: doi, href: doi },
  ];
}

export function referenceText(publication: Publication): string {
  return referenceSegments(publication)
    .map((segment) => segment.text)
    .join("");
}

export function referenceHtml(publication: Publication): string {
  return referenceSegments(publication).map(segmentHtml).join("");
}

export function bibtexEntry(publication: Publication): string {
  const fields: [string, string][] = [
    ["author", publication.authors.map(({ family, given }) => `${family}, ${given}`).join(" and ")],
    ["title", publication.title.replace(publication.titleWord, `{${publication.titleWord}}`)],
    ["journal", publication.journal],
    ["year", publication.year],
    ["volume", publication.volume],
    ["number", publication.issue],
    ["pages", publication.article],
    ["doi", publication.doi],
    ["url", doiUrl(publication)],
  ];

  const body = fields.map(([name, value]) => `  ${name} = {${value}},`).join("\n");

  return `@article{${bibtexKey(publication)},\n${body}\n}`;
}

export function inlineAuthors(publication: Publication): string {
  const [first, second, ...others] = publication.authors;

  if (second === undefined) {
    return first.family;
  }

  return others.length === 0 ? `${first.family} and ${second.family}` : `${first.family} et al.`;
}

export function paperLinks(publication: Publication): PaperLink[] {
  return [
    {
      id: "article",
      label: "Article",
      description: `Read the article in ${publication.journal} (DOI ${publication.doi})`,
      href: doiUrl(publication),
    },
    {
      id: "pdf",
      label: "PDF",
      description: `Open the PDF of the article from ${publication.journal}`,
      href: publication.pdf,
    },
    {
      id: "pmc",
      label: "PubMed Central",
      description: `Read the full text in PubMed Central (${publication.pmcid})`,
      href: `https://pmc.ncbi.nlm.nih.gov/articles/${publication.pmcid}/`,
    },
    {
      id: "pubmed",
      label: "PubMed",
      description: `Open the PubMed record (PMID ${publication.pmid})`,
      href: `https://pubmed.ncbi.nlm.nih.gov/${publication.pmid}/`,
    },
  ];
}

function chicagoAuthors(publication: Publication): string {
  const names = publication.authors.map(({ family, given }, index) =>
    index === 0 ? `${family}, ${given}` : `${given} ${family}`,
  );

  if (names.length > CHICAGO_MAX_LISTED_AUTHORS) {
    return `${names.slice(0, CHICAGO_ABBREVIATED_AUTHORS).join(", ")}, et al.`;
  }

  if (names.length <= 2) {
    return names.join(" and ");
  }

  return `${names.slice(0, -1).join(", ")}, and ${names.at(-1)}`;
}

function sentence(text: string): string {
  return /[.?!]$/u.test(text) ? text : `${text}.`;
}

function segmentHtml({ text, italic = false, href }: ReferenceSegment): string {
  const styled = italic ? `<i>${escapeHtml(text)}</i>` : escapeHtml(text);

  return href === undefined ? styled : `<a href="${escapeHtml(href)}">${styled}</a>`;
}

function escapeHtml(text: string): string {
  return text.replaceAll(/[&<>"]/gu, (character) => HTML_ESCAPES.get(character) ?? character);
}

function bibtexKey(publication: Publication): string {
  const family = publication.authors[0].family;

  return `${keyPart(family)}${publication.year}${keyPart(publication.titleWord)}`;
}

function keyPart(text: string): string {
  return text.toLowerCase().replaceAll(/[^a-z]/gu, "");
}
