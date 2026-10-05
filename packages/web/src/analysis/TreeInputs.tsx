import type { TreeText } from "@neherlab/treeknit-wasm";
import { useCallback, useState } from "react";
import { FileTrigger } from "react-aria-components";
import { useFieldArray } from "react-hook-form";

import { Button } from "../ui/Button";
import { exampleTrees } from "./example";
import { type AnalysisForm, readTrees } from "./request";

const NEWICK_FILE_TYPES = [".nwk", ".newick", ".tre", ".tree", ".txt"];

export function TreeInputs() {
  const { fields, replace } = useFieldArray<AnalysisForm, "trees">({ name: "trees" });
  const [readError, setReadError] = useState<string>();

  const selectFiles = useCallback(
    (list: FileList | null) => {
      setReadError(undefined);
      void readInto(Array.from(list ?? []), replace, setReadError);
    },
    [replace],
  );

  const loadExample = useCallback(() => {
    setReadError(undefined);
    replace(exampleTrees());
  }, [replace]);

  return (
    <fieldset className="flex flex-col gap-3">
      <legend className="mb-3 text-base font-semibold">Trees</legend>
      <p className="text-ink-muted text-sm">Newick files, one tree per segment.</p>
      <div className="flex gap-2">
        <FileTrigger allowsMultiple acceptedFileTypes={NEWICK_FILE_TYPES} onSelect={selectFiles}>
          <Button>Choose files</Button>
        </FileTrigger>
        <Button variant="quiet" onPress={loadExample}>
          Load example
        </Button>
      </div>
      {readError === undefined ? null : (
        <p role="alert" className="text-danger text-sm">
          {readError}
        </p>
      )}
      {fields.length === 0 ? null : (
        <ol className="list-inside list-decimal font-mono text-sm">
          {fields.map((field) => (
            <li key={field.id}>{field.label}</li>
          ))}
        </ol>
      )}
    </fieldset>
  );
}

async function readInto(
  files: readonly File[],
  replace: (trees: TreeText[]) => void,
  setError: (message: string) => void,
): Promise<void> {
  try {
    replace(await readTrees(files));
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error));
  }
}
