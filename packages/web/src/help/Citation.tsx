import { useCallback, useMemo, useState } from "react";
import type { Key } from "react-aria-components";

import { CodeBlock } from "../ui/CodeBlock";
import { ExternalLink } from "../ui/ExternalLink";
import { ToggleButton, ToggleButtonGroup } from "../ui/ToggleButtonGroup";
import { CITATION_FORMATS, type CitationFormat, formatCitation, TREEKNIT_PUBLICATION } from "./citation";

export function Citation() {
  const [format, setFormat] = useState<CitationFormat>("text");
  const selected = useMemo(() => [format], [format]);
  const label = CITATION_FORMATS.find((choice) => choice.format === format)?.label ?? format;

  const select = useCallback((keys: Set<Key>) => {
    const next = CITATION_FORMATS.find((choice) => keys.has(choice.format));

    if (next !== undefined) {
      setFormat(next.format);
    }
  }, []);

  return (
    <div className="flex min-w-0 flex-col gap-3">
      <ToggleButtonGroup aria-label="Citation format" selectedKeys={selected} onSelectionChange={select}>
        {CITATION_FORMATS.map((choice) => (
          <ToggleButton key={choice.format} id={choice.format} label={choice.label} />
        ))}
      </ToggleButtonGroup>
      <CodeBlock code={formatCitation(TREEKNIT_PUBLICATION, format)} label={label} lineNumbers={false} />
      <p className="text-sm">
        DOI: <ExternalLink href={TREEKNIT_PUBLICATION.href}>{TREEKNIT_PUBLICATION.doi}</ExternalLink>
      </p>
    </div>
  );
}
