import { type ComponentType, type SVGProps, useCallback, useMemo, useState } from "react";
import type { Key } from "react-aria-components";
import PubMedIcon from "~icons/lucide/book-open-text";
import PdfIcon from "~icons/lucide/file-down";
import ArticleIcon from "~icons/lucide/file-text";
import PmcIcon from "~icons/lucide/library";

import { CodeBlock } from "../ui/CodeBlock";
import { CopyPanel } from "../ui/CopyPanel";
import { ExternalButtonLink, ExternalLink } from "../ui/ExternalLink";
import { ToggleButton, ToggleButtonGroup } from "../ui/ToggleButtonGroup";
import {
  BIBTEX_ENTRY,
  CITATION_FORMATS,
  type CitationFormat,
  PAPER_LINKS,
  type PaperLinkId,
  REFERENCE_HTML,
  REFERENCE_SEGMENTS,
  REFERENCE_TEXT,
  type ReferenceSegment,
} from "./citation";

export const CITE_REQUEST = "If you use TreeKnit in your work, please cite:";

const PAPER_LINK_ICONS: Record<PaperLinkId, ComponentType<SVGProps<SVGSVGElement>>> = {
  article: ArticleIcon,
  pdf: PdfIcon,
  pmc: PmcIcon,
  pubmed: PubMedIcon,
};

const FORMAT_CHOICES = Object.values(CITATION_FORMATS);

const FORMAT_VIEWS: Record<CitationFormat, ComponentType<FormatViewProps>> = {
  text: ChicagoReference,
  bibtex: BibtexReference,
};

export function Citation() {
  const [format, setFormat] = useState<CitationFormat>("text");
  const selected = useMemo(() => [format], [format]);
  const choice = CITATION_FORMATS[format];
  const FormatView = FORMAT_VIEWS[format];

  const select = useCallback((keys: Set<Key>) => {
    const next = FORMAT_CHOICES.find((candidate) => keys.has(candidate.format));

    if (next !== undefined) {
      setFormat(next.format);
    }
  }, []);

  return (
    <div className="flex min-w-0 flex-col gap-3">
      <ToggleButtonGroup aria-label="Citation format" selectedKeys={selected} onSelectionChange={select}>
        {FORMAT_CHOICES.map((candidate) => (
          <ToggleButton
            key={candidate.format}
            id={candidate.format}
            label={candidate.label}
            tooltip={candidate.description}
          />
        ))}
      </ToggleButtonGroup>
      <FormatView caption={choice.caption} copyTooltip={choice.copyTooltip} />
      <nav aria-label="The TreeKnit paper" className="flex flex-wrap items-center gap-2">
        <span className="text-ink-muted text-sm">Open access:</span>
        {PAPER_LINKS.map((link) => (
          <ExternalButtonLink
            key={link.id}
            href={link.href}
            tooltip={link.description}
            icon={PAPER_LINK_ICONS[link.id]}
          >
            {link.label}
          </ExternalButtonLink>
        ))}
      </nav>
    </div>
  );
}

function ChicagoReference({ caption, copyTooltip }: FormatViewProps) {
  return (
    <CopyPanel label={caption} text={REFERENCE_TEXT} html={REFERENCE_HTML} copyTooltip={copyTooltip}>
      <p className="text-ink px-3 py-2.5 text-sm leading-relaxed wrap-anywhere">
        {REFERENCE_SEGMENTS.map((segment) => (
          <Segment key={segment.text} segment={segment} />
        ))}
      </p>
    </CopyPanel>
  );
}

function BibtexReference({ caption, copyTooltip }: FormatViewProps) {
  return <CodeBlock code={BIBTEX_ENTRY} label={caption} copyTooltip={copyTooltip} lineNumbers={false} />;
}

interface FormatViewProps {
  caption: string;
  copyTooltip: string;
}

function Segment({ segment: { text, italic = false, href } }: SegmentProps) {
  const styled = italic ? <i>{text}</i> : text;

  return href === undefined ? styled : <ExternalLink href={href}>{styled}</ExternalLink>;
}

interface SegmentProps {
  segment: ReferenceSegment;
}
