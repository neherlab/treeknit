export const TREEKNIT_PUBLICATION = {
  author: "Barrat-Charlaix et al",
  authors: [
    { family: "Barrat-Charlaix", given: "Pierre", initials: "P" },
    { family: "Vaughan", given: "Timothy G.", initials: "TG" },
    { family: "Neher", given: "Richard A.", initials: "RA" },
  ],
  title: "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses",
  titleWord: "TreeKnit",
  journal: "PLOS Comput Biol",
  journalName: "PLOS Computational Biology",
  year: "2022",
  volume: "18",
  issue: "8",
  article: "e1010394",
  doi: "10.1371/journal.pcbi.1010394",
  href: "https://doi.org/10.1371/journal.pcbi.1010394",
} as const;

export type Publication = typeof TREEKNIT_PUBLICATION;

export type CitationFormat = "text" | "bibtex";

export const CITATION_FORMATS: readonly { format: CitationFormat; label: string }[] = [
  { format: "text", label: "Text" },
  { format: "bibtex", label: "BibTeX" },
];

const FORMATTERS: Record<CitationFormat, (publication: Publication) => string> = {
  text: textReference,
  bibtex: bibtexEntry,
};

export function formatCitation(publication: Publication, format: CitationFormat): string {
  return FORMATTERS[format](publication);
}

export function shortAttribution(publication: Publication): string {
  const families = publication.authors.map(({ family }) => family);
  const last = families.at(-1) ?? "";
  const authors = families.length > 1 ? `${families.slice(0, -1).join(", ")} & ${last}` : last;

  return `${authors}, ${publication.journal} ${publication.year}`;
}

function textReference(publication: Publication): string {
  const authors = publication.authors.map(({ family, initials }) => `${family} ${initials}`).join(", ");
  const source = `${publication.year};${publication.volume}(${publication.issue}):${publication.article}`;

  return `${authors}. ${publication.title}. ${publication.journalName}. ${source}. ${publication.href}`;
}

function bibtexEntry(publication: Publication): string {
  const fields: [string, string][] = [
    ["author", publication.authors.map(({ family, given }) => `${family}, ${given}`).join(" and ")],
    ["title", publication.title.replace(publication.titleWord, `{${publication.titleWord}}`)],
    ["journal", publication.journalName],
    ["year", publication.year],
    ["volume", publication.volume],
    ["number", publication.issue],
    ["pages", publication.article],
    ["doi", publication.doi],
    ["url", publication.href],
  ];

  const body = fields.map(([name, value]) => `  ${name} = {${value}},`).join("\n");

  return `@article{${bibtexKey(publication)},\n${body}\n}`;
}

function bibtexKey(publication: Publication): string {
  const family = publication.authors[0].family;

  return `${keyPart(family)}${publication.year}${keyPart(publication.titleWord)}`;
}

function keyPart(text: string): string {
  return text.toLowerCase().replaceAll(/[^a-z]/gu, "");
}
