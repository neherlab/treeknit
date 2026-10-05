import type { TreeText } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { FileTrigger, Header, Menu, MenuItem, MenuSection, MenuTrigger, Popover } from "react-aria-components";
import { useFieldArray } from "react-hook-form";
import ChevronIcon from "~icons/lucide/chevron-down";

import { Button } from "../ui/Button";
import { optionClassName, popoverClassName } from "../ui/fieldStyles";
import { EXAMPLE_GROUPS, type Example } from "./example";
import { type AnalysisForm, readTrees } from "./request";

const NEWICK_FILE_TYPES = [".nwk", ".newick", ".tre", ".tree", ".txt"];

export function TreeInputs() {
  const { fields, replace } = useFieldArray<AnalysisForm, "trees">({ name: "trees" });
  const { mutate, error } = useMutation<TreeText[], Error, TreeSource>({ mutationFn: async (read) => read() });

  const load = useCallback(
    (read: TreeSource) => {
      mutate(read, {
        onSuccess: (trees) => {
          replace(trees);
        },
      });
    },
    [mutate, replace],
  );

  const selectFiles = useCallback(
    (list: FileList | null) => {
      const files = Array.from(list ?? []);
      load(async () => readTrees(files));
    },
    [load],
  );

  return (
    <fieldset className="flex flex-col gap-3">
      <legend className="mb-3 text-base font-semibold">Trees</legend>
      <p className="text-ink-muted text-sm">Newick files, one tree per segment.</p>
      <div className="flex gap-2">
        <FileTrigger allowsMultiple acceptedFileTypes={NEWICK_FILE_TYPES} onSelect={selectFiles}>
          <Button>Choose files</Button>
        </FileTrigger>
        <ExampleMenu onSelect={load} />
      </div>
      {error === null ? null : (
        <p role="alert" className="text-danger text-sm">
          {error.message}
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

type TreeSource = () => Promise<TreeText[]>;

function ExampleMenu({ onSelect }: ExampleMenuProps) {
  return (
    <MenuTrigger>
      <Button variant="quiet">
        Load example
        <ChevronIcon aria-hidden />
      </Button>
      <Popover className={popoverClassName}>
        <Menu className="max-h-[inherit] overflow-auto outline-none">
          {EXAMPLE_GROUPS.map((group) => (
            <MenuSection key={group.id} id={group.id}>
              <Header className="text-ink-muted px-3 pt-2 pb-1 text-xs font-semibold">{group.name}</Header>
              {group.examples.map((example) => (
                <ExampleItem key={example.id} example={example} onSelect={onSelect} />
              ))}
            </MenuSection>
          ))}
        </Menu>
      </Popover>
    </MenuTrigger>
  );
}

interface ExampleMenuProps {
  onSelect: (read: TreeSource) => void;
}

function ExampleItem({ example, onSelect }: ExampleItemProps) {
  const select = useCallback(() => {
    onSelect(example.load);
  }, [example, onSelect]);

  return (
    <MenuItem id={example.id} onAction={select} className={optionClassName}>
      {example.name}
    </MenuItem>
  );
}

interface ExampleItemProps {
  example: Example;
  onSelect: (read: TreeSource) => void;
}
