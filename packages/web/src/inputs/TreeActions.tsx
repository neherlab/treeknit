import { useCallback, useMemo, useState } from "react";
import { FileTrigger, type Key } from "react-aria-components";
import PasteIcon from "~icons/lucide/clipboard-paste";
import FilePlusIcon from "~icons/lucide/file-plus";
import FlaskIcon from "~icons/lucide/flask-conical";

import { EXAMPLE_GROUPS } from "../analysis/example";
import { Button } from "../ui/Button";
import { Menu, MenuItem, MenuSection } from "../ui/Menu";
import { PasteTreeDialog } from "./PasteTreeDialog";
import { TREE_FILE_TYPES } from "./treeFiles";
import type { TreeInput } from "./useTreeInput";

const EXAMPLES = new Map(EXAMPLE_GROUPS.flatMap(({ examples }) => examples.map((example) => [example.id, example])));

export function TreeActions({ input, variant = "secondary", size = "sm" }: TreeActionsProps) {
  const [pasting, setPasting] = useState(false);

  const selectFiles = useCallback(
    (files: FileList | null) => {
      void input.addFiles(files === null ? [] : [...files]);
    },
    [input],
  );

  const openPaste = useCallback(() => {
    setPasting(true);
  }, []);

  const addPasted = useCallback(
    (newick: string, label: string) => {
      void input.addPasted(newick, label);
    },
    [input],
  );

  const addExample = useCallback(
    (key: Key) => {
      const example = EXAMPLES.get(String(key));

      if (example !== undefined) {
        void input.addExample(example);
      }
    },
    [input],
  );

  const examplesTrigger = useMemo(
    () => (
      <Button variant={variant} size={size} icon={FlaskIcon} tooltip="Add an example dataset">
        Examples
      </Button>
    ),
    [size, variant],
  );

  return (
    <div className="flex flex-wrap items-center gap-2">
      <FileTrigger acceptedFileTypes={TREE_FILE_TYPES} allowsMultiple onSelect={selectFiles}>
        <Button variant={variant} size={size} icon={FilePlusIcon}>
          Add files
        </Button>
      </FileTrigger>
      <Button variant={variant} size={size} icon={PasteIcon} onPress={openPaste}>
        Paste tree
      </Button>
      <Menu aria-label="Examples" onAction={addExample} trigger={examplesTrigger}>
        {EXAMPLE_GROUPS.map((group) => (
          <MenuSection key={group.id} title={group.name}>
            {group.examples.map((example) => (
              <MenuItem key={example.id} id={example.id}>
                {example.name}
              </MenuItem>
            ))}
          </MenuSection>
        ))}
      </Menu>
      <PasteTreeDialog isOpen={pasting} onOpenChange={setPasting} onAdd={addPasted} />
    </div>
  );
}

export interface TreeActionsProps {
  input: TreeInput;
  variant?: "primary" | "secondary" | "quiet";
  size?: "md" | "sm";
}
