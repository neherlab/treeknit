import type { LaunchKeyInfo } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useLaunchKeys } from "../analysis/queries";
import { ExternalLink } from "../ui/ExternalLink";
import { VIEW_KEYS } from "../workspace/search";

const CODE = "font-mono text-sm";

const LINK_EXAMPLES: readonly LinkExample[] = [
  {
    query: "?example=h3n2-2017&run&view=tanglegram",
    effect: "Runs a built-in example and shows the tanglegram.",
    opens: true,
  },
  {
    query:
      "?tree=https://raw.githubusercontent.com/neherlab/treeknit-rs/main/data/h3n2-2017/ha.nwk&tree=https://raw.githubusercontent.com/neherlab/treeknit-rs/main/data/h3n2-2017/na.nwk&resolve=strict&gamma=3",
    effect: "Loads two trees from their addresses with strict resolution and γ = 3, ready to run.",
    opens: true,
  },
  {
    query:
      "?tree=HA=https://github.com/org/repo/blob/main/seg4.nwk&tree=NA=https://github.com/org/repo/blob/main/seg6.nwk&seq-lengths=1701,1410&run",
    effect: "Labels the trees HA and NA. The address of a GitHub file page works as well as its raw file.",
    opens: false,
  },
  {
    query: "?tree=ha=data:,((A,B),(C,(D,X)));&tree=na=data:,((A,(B,X)),(C,D));&run",
    effect: "Holds two small trees in the link itself.",
    opens: true,
  },
  {
    query: "?session=https://zenodo.org/records/123/files/treeknit_session.json&seed=7&run&view=mccs",
    effect: "Runs a session file from Zenodo with another seed and shows the MCC table.",
    opens: false,
  },
  {
    query: "?example=h3n2-2017&run&view=auspice&show=ha&auspice=legend=open%26l=radial",
    effect: "Shows one tree of the pair in the Auspice view, with the settings of Auspice in the auspice key.",
    opens: true,
  },
  {
    query: "?run&view=tanglegram&pair=ha:na&mcc=4#session=data:application/gzip;base64,H4sIAAAA...",
    effect:
      "The form of Copy link for trees without an address: the trees travel in the part after #, which no server receives.",
    opens: false,
  },
];

const SENDER_EXAMPLE = `const app = window.open(TREEKNIT + "?from=opener&view=tanglegram");

window.addEventListener("message", (event) => {
  if (event.source === app && event.data?.type === "treeknit:ready") {
    app.postMessage(
      { type: "treeknit:open", session: sessionFileText, run: true },
      event.origin,
    );
  }
});`;

export function LinkHelp() {
  const { data: keys } = useLaunchKeys();
  const base = useMemo(() => new URL("./", window.location.href).href, []);

  return (
    <>
      <p>
        A link to <code className={CODE}>{base}</code> can name trees, settings, a run, and a view. The address bar then
        describes what the page shows, so copying it shares the result: the same trees, settings, and seed give the same
        result. Copy link in the run bar gives a link also for trees from files.
      </p>
      <ul className="flex flex-col gap-2">
        {LINK_EXAMPLES.map(({ query, effect, opens }) => (
          <li key={query} className="flex flex-col gap-0.5">
            {opens ? (
              <ExternalLink href={`./${query}`} className="font-mono text-sm break-all">
                {query}
              </ExternalLink>
            ) : (
              <code className={`${CODE} break-all`}>{query}</code>
            )}
            <span>{effect}</span>
          </li>
        ))}
      </ul>
      <p>
        Write <code className={CODE}>&amp;</code> inside an address as <code className={CODE}>%26</code>, and the number
        of a key as the interface shows it. The command line runs a link with{" "}
        <code className={CODE}>treeknit --link</code> and prints one with <code className={CODE}>--print-link</code>.
      </p>
      {keys === undefined ? null : <KeyList keys={keys} />}
      <p>
        The keys of the view: {VIEW_KEYS.join(", ")}. A remote file must allow other sites to read it (CORS), which
        GitHub, Zenodo, and Nextstrain do.
      </p>
    </>
  );
}

export function MessageHelp() {
  return (
    <>
      <p>
        A page can open TreeKnit with <code className={CODE}>?from=opener</code>, or embed it in a frame with{" "}
        <code className={CODE}>?from=parent</code>, and send it a session file of any size. TreeKnit posts{" "}
        <code className={CODE}>{'{ type: "treeknit:ready", protocol: 1 }'}</code> to that page every 250 ms for 10
        seconds, and opens the first <code className={CODE}>treeknit:open</code> message that the page sends back:
      </p>
      <pre className="bg-pane rounded-control overflow-x-auto p-3 font-mono text-sm">{SENDER_EXAMPLE}</pre>
      <p>
        A page served with <code className={CODE}>Cross-Origin-Opener-Policy: same-origin</code> loses its link to the
        window it opens, so it cannot send TreeKnit a session file this way.
      </p>
    </>
  );
}

function KeyList({ keys }: { keys: readonly LaunchKeyInfo[] }) {
  return (
    <dl className="flex flex-col gap-2">
      {keys.map(({ key, opposite, value, description }) => (
        <div key={key}>
          <dt className={CODE}>
            {value === null ? key : `${key}=${value}`}
            {opposite === null ? null : ` (opposite: ${opposite})`}
          </dt>
          <dd>{description}</dd>
        </div>
      ))}
    </dl>
  );
}

interface LinkExample {
  query: string;
  effect: string;
  opens: boolean;
}
