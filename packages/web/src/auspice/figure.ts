import { escapeHtml, REFERENCE_HTML } from "../help/citation";
import type { AuspicePublication } from "./state";

export function figureCaption({ title, summary, publications }: FigureCaptionInput): string[] {
  return [
    escapeHtml(title),
    escapeHtml(summary),
    "",
    "Relevant publications:",
    REFERENCE_HTML,
    ...publications.map(publicationHtml),
  ];
}

export interface FigureCaptionInput {
  title: string;
  summary: string;
  publications: readonly AuspicePublication[];
}

function publicationHtml({ author, title, journal, year, href }: AuspicePublication): string {
  const text = `${author}, ${title}, ${journal} (${year})`;

  return `<a href="${escapeHtml(href)}">${escapeHtml(text)}</a>`;
}
