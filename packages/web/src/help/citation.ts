const HTML_ESCAPES = new Map([
  ["&", "&amp;"],
  ["<", "&lt;"],
  [">", "&gt;"],
  ['"', "&quot;"],
]);

const DOI = "10.1371/journal.pcbi.1010394";

const DOI_URL = `https://doi.org/${DOI}`;

const JOURNAL = "PLOS Computational Biology";

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

export const REFERENCE_SEGMENTS: readonly ReferenceSegment[] = [
  {
    text: 'Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." ',
  },
  { text: JOURNAL, italic: true },
  { text: " 18:e1010394. " },
  { text: DOI_URL, href: DOI_URL },
];

export const REFERENCE_TEXT = REFERENCE_SEGMENTS.map((segment) => segment.text).join("");

export const REFERENCE_HTML = REFERENCE_SEGMENTS.map(segmentHtml).join("");

export const BIBTEX_ENTRY = `@article{barratcharlaix2022treeknit,
  author = {Barrat-Charlaix, Pierre and Vaughan, Timothy G. and Neher, Richard A.},
  title = {{TreeKnit}: Inferring ancestral reassortment graphs of influenza viruses},
  journal = {${JOURNAL}},
  year = {2022},
  volume = {18},
  number = {8},
  pages = {e1010394},
  doi = {${DOI}},
  url = {${DOI_URL}},
}`;

export const PAPER_LINKS: readonly PaperLink[] = [
  {
    id: "article",
    label: "Article",
    description: `Read the article in ${JOURNAL} (DOI ${DOI})`,
    href: DOI_URL,
  },
  {
    id: "pdf",
    label: "PDF",
    description: `Open the PDF of the article from ${JOURNAL}`,
    href: `https://journals.plos.org/ploscompbiol/article/file?id=${DOI}&type=printable`,
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
];

export function escapeHtml(text: string): string {
  return text.replaceAll(/[&<>"]/gu, (character) => HTML_ESCAPES.get(character) ?? character);
}

function segmentHtml({ text, italic = false, href }: ReferenceSegment): string {
  const styled = italic ? `<i>${escapeHtml(text)}</i>` : escapeHtml(text);

  return href === undefined ? styled : `<a href="${escapeHtml(href)}">${styled}</a>`;
}
