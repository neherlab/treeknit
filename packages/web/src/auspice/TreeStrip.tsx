import type { PairSummary } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";

import { ToggleButton, ToggleButtonGroup } from "../ui/ToggleButtonGroup";
import { type ShownTrees, type StripChoice, shownFromStrip, shownTreeIndices } from "./strip";

const AT_MOST_TWO = "At most two trees: hide one to show another";

const LONG_LABEL_CHARS = 20;

export function TreeStrip({ labels, pairs, shown, onChange }: TreeStripProps) {
  const pair = pairs[shown.pair];
  const selected = useMemo(() => (pair === undefined ? [] : shownTreeIndices(pair, shown.trees)), [pair, shown.trees]);
  const selectedKeys = useMemo(() => selected.map(String), [selected]);

  const change = useCallback(
    (keys: Set<Key>) => {
      const next = shownFromStrip(
        [...keys].flatMap((key) => (labels[Number(key)] === undefined ? [] : [Number(key)])),
        pairs,
        shown,
      );

      if (next !== null) {
        onChange(next);
      }
    },
    [labels, pairs, shown, onChange],
  );

  if (labels.length < 2) {
    return null;
  }

  return (
    <ToggleButtonGroup
      aria-label="Shown trees"
      selectionMode="multiple"
      selectedKeys={selectedKeys}
      onSelectionChange={change}
    >
      {labels.map((label, index) => {
        const unavailable = selected.length === 2 && !selected.includes(index);

        return (
          <ToggleButton
            key={label}
            id={String(index)}
            label={label}
            className="max-w-40"
            unavailableReason={unavailable ? AT_MOST_TWO : undefined}
            tooltip={label.length > LONG_LABEL_CHARS ? label : undefined}
          />
        );
      })}
    </ToggleButtonGroup>
  );
}

export interface TreeStripProps {
  labels: readonly string[];
  pairs: readonly PairSummary[];
  shown: ShownTrees;
  onChange: (choice: StripChoice) => void;
}
