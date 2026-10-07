import { useCallback, useState } from "react";

import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { TextField } from "../ui/TextField";

const NEWICK_ROWS = 10;

export function PasteTreeDialog({ isOpen, onOpenChange, onAdd }: PasteTreeDialogProps) {
  const [newick, setNewick] = useState("");
  const [label, setLabel] = useState("");

  const close = useCallback(() => {
    setNewick("");
    setLabel("");
    onOpenChange(false);
  }, [onOpenChange]);

  const changeOpen = useCallback(
    (open: boolean) => {
      if (open) {
        onOpenChange(true);
      } else {
        close();
      }
    },
    [close, onOpenChange],
  );

  const add = useCallback(() => {
    onAdd(newick, label);
    close();
  }, [close, label, newick, onAdd]);

  return (
    <Dialog
      title="Paste tree"
      isOpen={isOpen}
      onOpenChange={changeOpen}
      className="max-w-2xl"
      footer={
        <>
          <Button onPress={close}>Cancel</Button>
          <Button variant="primary" isDisabled={newick.trim() === ""} onPress={add}>
            Add tree
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4 p-4">
        <TextField
          label="Newick"
          multiline
          mono
          rows={NEWICK_ROWS}
          value={newick}
          onChange={setNewick}
          placeholder="((A,B),(C,D));"
        />
        <TextField label="Label" value={label} onChange={setLabel} />
      </div>
    </Dialog>
  );
}

export interface PasteTreeDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onAdd: (newick: string, label: string) => void;
}
