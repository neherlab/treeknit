import type { ReactNode } from "react";

import { useResultNames, useSettingsSchema, useVersion } from "../analysis/queries";
import { KEEP_WORKSPACE } from "../persistence/PersistenceSwitch";
import { PLACEHOLDER_LABELS } from "../results/fileRows";
import { ModeList } from "../settings/ModeList";
import { Header } from "../shell/Header";
import { ExternalLink } from "../ui/ExternalLink";
import { useWorkspace } from "../workspace/context";
import { CITE_REQUEST, Citation } from "./Citation";
import { LinkHelp, MessageHelp } from "./LinkHelp";

const AUSPICE_URL = "https://docs.nextstrain.org/projects/auspice";

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
  const { data: names } = useResultNames(...PLACEHOLDER_LABELS);

  return (
    <>
      <Header />
      <main className="min-h-0 flex-1 overflow-y-auto">
        <article className="flex max-w-[75ch] flex-col gap-10 px-6 py-8 text-base">
          <h1 className="text-lg font-semibold">Help</h1>
          <HelpSection id="help-about" title="What TreeKnit does">
            <p>
              TreeKnit infers reassortment from the trees of the segments of a genome. For each pair of trees, it finds
              the MCCs, the groups of leaves that the two trees place in the same way. Where the MCCs change, the
              segments have different histories. TreeKnit also resolves polytomies with the splits of the other trees,
              and for two trees it builds the ancestral reassortment graph.
            </p>
            <ol className="flex list-decimal flex-col gap-1.5 pl-6">
              <li>Add at least two Newick trees: drop files, add files, paste a tree, or start with an example.</li>
              <li>Check the trees: their labels, leaf counts, parse errors, and the leaves that the trees share.</li>
              <li>Set the settings. Each setting has an info button that explains it.</li>
              <li>Run TreeKnit with the button or with Ctrl+Enter (Cmd+Enter on a Mac).</li>
              <li>
                Hide or show the trees and settings with Ctrl+B (Cmd+B on a Mac), and the details with Ctrl+Alt+B
                (Cmd+Option+B), or with the tab on the edge of each pane.
              </li>
              <li>
                Read the result in the Auspice view, which opens when the run finishes, or in the overview, the
                tanglegram of a pair, or the ARG.
              </li>
              <li>Download the files, the figures, and a session file that restores the trees and settings.</li>
            </ol>
          </HelpSection>
          <HelpSection id="help-resolution" title="Resolution modes">
            {schema === undefined ? null : (
              <>
                <ModeList modes={schema.modes} />
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
              Your trees stay in this browser. TreeKnit sends nothing to a server. They are saved between visits only
              when you turn on {KEEP_WORKSPACE}.
            </p>
          </HelpSection>
          <HelpSection id="help-auspice" title="Auspice view">
            <p>
              The Auspice view draws the two trees of a pair facing each other with{" "}
              <ExternalLink href={AUSPICE_URL}>Auspice</ExternalLink>, the tree viewer of Nextstrain, with a line
              between the two copies of each leaf. Its sidebar colors and filters the trees by MCC, labels the branches
              where an MCC starts, and downloads the drawing as SVG. Choosing a leaf or, with Shift, a branch shows its
              details in the inspector.
            </p>
            <p>
              Auspice is made by the Nextstrain team and is licensed under the GNU Affero General Public License v3.0
              (AGPL-3.0). The TreeKnit source code is MIT-licensed; this web app contains Auspice, so the app as served
              is distributed under the AGPL-3.0. Its complete source, including the patch it applies to Auspice, is in
              the TreeKnit repository
              {version === undefined ? (
                "."
              ) : (
                <>
                  : <ExternalLink href={version.repository}>{version.repository}</ExternalLink>
                </>
              )}
            </p>
          </HelpSection>
          <HelpSection id="help-other-tools" title="Open the results in other tools">
            {names === undefined ? null : (
              <>
                <p>
                  Open <ExternalLink href="https://auspice.us">auspice.us</ExternalLink> and drop{" "}
                  <code className="font-mono text-sm">{names.auspiceFiles[0]}</code> and{" "}
                  <code className="font-mono text-sm">{names.auspiceFiles[1]}</code> on the page together. Auspice shows
                  the two trees side by side, with lines between the copies of each leaf.
                </p>
                <p>
                  Open <code className="font-mono text-sm">{names.argNewick}</code> in{" "}
                  <ExternalLink href="https://icytree.org">IcyTree</ExternalLink> to see the ARG. IcyTree cannot read
                  the file when leaf names contain spaces or any of the characters{" "}
                  <code className="font-mono text-sm">{ICYTREE_CHARACTERS}</code>.
                </p>
              </>
            )}
          </HelpSection>
          <HelpSection id="help-links" title="Open TreeKnit from a link">
            <LinkHelp />
          </HelpSection>
          <HelpSection id="help-messages" title="Send trees from another page">
            <MessageHelp />
          </HelpSection>
          <HelpSection id="help-cite" title="How to cite">
            <p>{CITE_REQUEST}</p>
            <Citation />
          </HelpSection>
          <HelpSection id="help-version" title="Version">
            {version === undefined ? null : (
              <p>
                TreeKnit {version.version}. Source code:{" "}
                <ExternalLink href={version.repository}>{version.repository}</ExternalLink>. Command-line releases:{" "}
                <ExternalLink href={version.releases}>{version.releases}</ExternalLink>
              </p>
            )}
          </HelpSection>
        </article>
      </main>
    </>
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
