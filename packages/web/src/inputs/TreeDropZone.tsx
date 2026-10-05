import { cn } from "cn";
import { type ReactNode, useCallback } from "react";
import type { DropEvent } from "react-aria";
import { DropZone } from "react-aria-components";

import type { TreeInput } from "./useTreeInput";

export function TreeDropZone({ input, label, className, children }: TreeDropZoneProps) {
  const drop = useCallback(
    (event: DropEvent) => {
      void input.addDropped(event.items);
    },
    [input],
  );

  return (
    <DropZone
      aria-label={label}
      getDropOperation={copyDrop}
      onDrop={drop}
      className={cn(
        "rounded-control outline-hidden",
        "data-drop-target:outline-focus data-drop-target:bg-focus/5 data-drop-target:outline-2 data-drop-target:outline-dashed",
        "data-focus-visible:outline-focus data-focus-visible:outline-2 data-focus-visible:outline-offset-2",
        className,
      )}
    >
      {children}
    </DropZone>
  );
}

export interface TreeDropZoneProps {
  input: TreeInput;
  label: string;
  className?: string;
  children: ReactNode;
}

function copyDrop(): "copy" {
  return "copy";
}
