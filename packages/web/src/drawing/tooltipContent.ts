export interface TooltipContent {
  text: string;
  style: Partial<CSSStyleDeclaration>;
}

const TOOLTIP_STYLE: Partial<CSSStyleDeclaration> = {
  backgroundColor: "var(--color-ink)",
  color: "var(--color-ground)",
  fontFamily: "var(--font-sans)",
  fontSize: "12px",
  lineHeight: "16px",
  padding: "4px 8px",
  borderRadius: "4px",
  maxWidth: "256px",
  whiteSpace: "pre-line",
  overflowWrap: "anywhere",
};

export function tooltipContent(lines: readonly string[]): TooltipContent | null {
  return lines.length === 0 ? null : { text: lines.join("\n"), style: TOOLTIP_STYLE };
}
