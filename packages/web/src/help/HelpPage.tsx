import type { ReactNode } from "react";

import { useSettingsSchema, useVersion } from "../analysis/queries";
import { KEEP_WORKSPACE } from "../persistence/PersistenceSwitch";
import { ExternalLink } from "../ui/ExternalLink";
import { useWorkspace } from "../workspace/context";

const PAPER_DOI = "https://doi.org/10.1371/journal.pcbi.1010394";

const GLOSSARY: readonly { term: string; definition: string }[] = [
  {
    term: "MCC",
    definition:
      "A maximally compatible clade: a largest set of leaves whose subtrees have the same topology in both trees of a pair. Leaves of one MCC share their history in both segments.",
  },
  {
    term: "Reassortment",
    definition:
      "The exchange of segments between viruses that infect the same cell, so that the segments of one genome have different histories. In a pair of trees, a reassortment separates two MCCs.",
  },
  {
    term: "Polytomy",
    definition:
      "An internal node with more than two children. A polytomy often marks a part of a tree that the data do not resolve.",
  },
  {
    term: "Resolution",
    definition:
      "Adding to a tree the splits of another tree that do not conflict with it, so that polytomies become binary nodes.",
  },
  {
    term: "Imputation",
    definition:
      "Placing into a tree the leaves that only other trees contain, at the position that their MCC gives them.",
  },
  {
    term: "ARG",
    definition:
      "The ancestral reassortment graph of two trees: one network that holds the histories of both segments, with a hybrid node for each reassortment.",
  },
  {
    term: "γ",
    definition:
      "The cost of a reassortment in the inference. A higher γ makes each reassortment more costly, so TreeKnit infers fewer of them.",
  },
];

const ICYTREE_CHARACTERS = ", ( ) : ; [ ] '";

export function HelpPage() {
  const treeCount = useWorkspace((state) => state.trees.length);
  const settings = useWorkspace((state) => state.settings);
  const { data: schema } = useSettingsSchema(treeCount, settings);
  const { data: version } = useVersion();

  return (
    <main className="min-h-0 flex-1 overflow-y-auto">
      <article className="flex max-w-[75ch] flex-col gap-10 px-6 py-8 text-base">
        <h1 className="text-lg font-semibold">Help</h1>
        <HelpSection id="help-about" title="What TreeKnit does">
          <p>
            TreeKnit infers reassortment from the trees of the segments of a genome. For each pair of trees, it finds
            the MCCs, the groups of leaves that the two trees place in the same way. Where the MCCs change, the segments
            have different histories. TreeKnit also resolves polytomies with the splits of the other trees, and for two
            trees it builds the ancestral reassortment graph.
          </p>
          <ol className="flex list-decimal flex-col gap-1.5 pl-6">
            <li>Add at least two Newick trees: drop files, add files, paste a tree, or start with an example.</li>
            <li>Check the trees: their labels, leaf counts, parse errors, and the leaves that the trees share.</li>
            <li>Set the settings. Each setting has an info button that explains it.</li>
            <li>Run TreeKnit with the button or with Ctrl+Enter (Cmd+Enter on a Mac).</li>
            <li>Read the overview, then open the tanglegram of a pair or the ARG.</li>
            <li>Download the files, the figures, and a session file that restores the trees and settings.</li>
          </ol>
        </HelpSection>
        <HelpSection id="help-resolution" title="Resolution modes">
          {schema === undefined ? null : (
            <>
              <dl className="flex flex-col gap-3">
                {schema.modes.map(({ mode, name, effect }) => (
                  <div key={mode}>
                    <dt className="font-semibold">{name}</dt>
                    <dd>{effect}</dd>
                  </div>
                ))}
              </dl>
              <p>{schema.treeOrderHelp}</p>
            </>
          )}
        </HelpSection>
        <HelpSection id="help-glossary" title="Glossary">
          <dl className="flex flex-col gap-3">
            {GLOSSARY.map(({ term, definition }) => (
              <div key={term}>
                <dt className="font-semibold">{term}</dt>
                <dd>{definition}</dd>
              </div>
            ))}
          </dl>
        </HelpSection>
        <HelpSection id="help-privacy" title="Privacy">
          <p>
            Your trees stay in this browser. TreeKnit sends nothing to a server. They are saved between visits only when
            you turn on {KEEP_WORKSPACE}.
          </p>
        </HelpSection>
        <HelpSection id="help-other-tools" title="Open the results in other tools">
          <p>
            Open <ExternalLink href="https://auspice.us">auspice.us</ExternalLink> and drop{" "}
            <code className="font-mono text-sm">auspice_&lt;a&gt;.json</code> and{" "}
            <code className="font-mono text-sm">auspice_&lt;b&gt;.json</code> on the page together. Auspice shows the
            two trees side by side, with lines between the copies of each leaf.
          </p>
          <p>
            Open <code className="font-mono text-sm">ARG/arg.nwk</code> in{" "}
            <ExternalLink href="https://icytree.org">IcyTree</ExternalLink> to see the ARG. IcyTree cannot read the file
            when leaf names contain spaces or any of the characters{" "}
            <code className="font-mono text-sm">{ICYTREE_CHARACTERS}</code>.
          </p>
        </HelpSection>
        <HelpSection id="help-cite" title="How to cite">
          <p>
            Barrat-Charlaix, Vaughan &amp; Neher, PLoS Comput Biol 18(8): e1010394 (2022),{" "}
            <ExternalLink href={PAPER_DOI}>{PAPER_DOI}</ExternalLink>
          </p>
        </HelpSection>
        <HelpSection id="help-version" title="Version">
          {version === undefined ? null : (
            <p>
              TreeKnit {version.version}. Source code:{" "}
              <ExternalLink href={version.repository}>{version.repository}</ExternalLink>
            </p>
          )}
        </HelpSection>
      </article>
    </main>
  );
}

function HelpSection({ id, title, children }: HelpSectionProps) {
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h2 id={id} className="text-base font-semibold">
        {title}
      </h2>
      {children}
    </section>
  );
}

interface HelpSectionProps {
  id: string;
  title: string;
  children: ReactNode;
}
