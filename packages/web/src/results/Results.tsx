import type { Analysis, ArgOutcome, OutputFile, PairMccs } from "@neherlab/treeknit-wasm";
import { useCallback } from "react";
import DownloadIcon from "~icons/lucide/download";

import { Button } from "../ui/Button";
import { saveFile } from "./download";

export function Results({ analysis }: ResultsProps) {
  return (
    <div className="@container flex flex-col gap-8">
      {analysis.arg === null ? null : <ArgSummary outcome={analysis.arg} />}
      {analysis.pairs.map((pair) => (
        <PairTable key={pair.trees.join("\u0000")} pair={pair} />
      ))}
      <section aria-labelledby="files-heading" className="flex flex-col gap-3">
        <h3 id="files-heading" className="text-base font-semibold">
          Files
        </h3>
        <ul className="grid grid-cols-1 gap-2 @md:grid-cols-2">
          {analysis.files.map((file) => (
            <li key={file.name}>
              <DownloadButton file={file} />
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

interface ResultsProps {
  analysis: Analysis;
}

function ArgSummary({ outcome }: { outcome: ArgOutcome }) {
  return outcome.status === "built" ? (
    <p className="text-lg">
      Ancestral reassortment graph with <span className="font-mono font-semibold">{outcome.reassortments}</span>{" "}
      {outcome.reassortments === 1 ? "reassortment" : "reassortments"}
    </p>
  ) : (
    <p className="text-danger">No ancestral reassortment graph: {outcome.message}</p>
  );
}

function PairTable({ pair }: { pair: PairMccs }) {
  const [first, second] = pair.trees;

  return (
    <table className="w-full border-collapse text-left text-sm">
      <caption className="mb-2 text-left text-base font-semibold">
        <span className="font-mono">{first}</span> and <span className="font-mono">{second}</span>: {pair.mccs.length}{" "}
        {pair.mccs.length === 1 ? "MCC" : "MCCs"}
      </caption>
      <thead>
        <tr className="border-rule text-ink-muted border-b">
          <th scope="col" className="w-12 py-1 pr-3 font-normal">
            #
          </th>
          <th scope="col" className="py-1 font-normal">
            Leaves
          </th>
        </tr>
      </thead>
      <tbody>
        {pair.mccs.map((mcc, index) => (
          <tr key={mcc.join("\u0000")} className="border-rule border-b align-top">
            <td className="text-ink-muted py-1 pr-3 font-mono">{index + 1}</td>
            <td className="text-clade py-1 font-mono break-words">{mcc.join(", ")}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function DownloadButton({ file }: { file: OutputFile }) {
  const save = useCallback(() => {
    saveFile(file);
  }, [file]);

  return (
    <Button className="w-full justify-start font-mono" onPress={save}>
      <DownloadIcon aria-hidden />
      {file.name}
    </Button>
  );
}
