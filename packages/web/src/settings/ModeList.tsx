import type { ModeInfo } from "@neherlab/treeknit-wasm";
import { cn } from "cn";

export function ModeList({ modes, dense = false }: ModeListProps) {
  return (
    <dl className={cn("flex flex-col", dense ? "gap-1.5" : "gap-3")}>
      {modes.map(({ mode, name, effect }) => (
        <div key={mode}>
          <dt className="font-semibold">{name}</dt>
          <dd>{effect}</dd>
        </div>
      ))}
    </dl>
  );
}

export interface ModeListProps {
  modes: readonly ModeInfo[];
  dense?: boolean;
}
