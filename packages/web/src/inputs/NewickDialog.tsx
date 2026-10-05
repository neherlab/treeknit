import { CodeBlock } from "../ui/CodeBlock";
import type { TextRange } from "../ui/codeLines";
import { Dialog } from "../ui/Dialog";

export function NewickDialog({ label, newick, errorRange, isOpen, onOpenChange }: NewickDialogProps) {
  return (
    <Dialog title={label} isOpen={isOpen} onOpenChange={onOpenChange} className="max-w-3xl">
      <div className="p-4">
        <CodeBlock code={newick} label="Newick" {...(errorRange === undefined ? undefined : { errorRange })} />
      </div>
    </Dialog>
  );
}

export interface NewickDialogProps {
  label: string;
  newick: string;
  errorRange: TextRange | undefined;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
}
