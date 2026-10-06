import type { LabelMode, PairSummary, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";
import DownloadIcon from "~icons/lucide/download";

import { Button } from "../ui/Button";
import { DisabledButton } from "../ui/DisabledButton";
import { Select, type SelectOption } from "../ui/Select";
import { ToggleButton, ToggleButtonGroup } from "../ui/ToggleButtonGroup";
import { LABEL_MODES, TREE_VERSIONS, SCALES } from "../workspace/search";
import { SCALE_LABELS } from "./scale";

const VERSION_LABELS: Record<TreeVersion, string> = { input: "Input", resolved: "Resolved", imputed: "Imputed" };

const LABEL_MODE_OPTIONS: SelectOption<LabelMode>[] = LABEL_MODES.map((mode) => ({
  id: mode,
  label: { auto: "Automatic", on: "On", off: "Off" }[mode],
}));

const FIGURE_BUTTON_LABEL = "Download figure (SVG)";

export function ScaleToggle({ value, onChange }: ChoiceProps<Scale>) {
  return <ChoiceGroup label="Branch scale" choices={SCALES} labels={SCALE_LABELS} value={value} onChange={onChange} />;
}

export function VersionToggle({ value, onChange }: ChoiceProps<TreeVersion>) {
  return (
    <ChoiceGroup
      label="Tree version"
      choices={TREE_VERSIONS}
      labels={VERSION_LABELS}
      value={value}
      onChange={onChange}
    />
  );
}

export function LabelModeSelect({ value, onChange }: ChoiceProps<LabelMode>) {
  return (
    <div className="flex items-center gap-2">
      <span aria-hidden className="text-ink-muted text-sm">
        Labels
      </span>
      <Select
        label="Labels"
        labelHidden
        options={LABEL_MODE_OPTIONS}
        value={value}
        onChange={onChange}
        className="w-32"
      />
    </div>
  );
}

export function PairSelect({ pairs, value, onChange }: PairSelectProps) {
  const options = useMemo<SelectOption<string>[]>(
    () => pairs.map(({ index, labels: [a, b] }) => ({ id: String(index), label: `${a} and ${b}` })),
    [pairs],
  );

  const choose = useCallback(
    (id: string) => {
      onChange(Number(id));
    },
    [onChange],
  );

  if (pairs.length < 2) {
    return null;
  }

  return <Select label="Pair" labelHidden options={options} value={String(value)} onChange={choose} className="w-48" />;
}

export interface PairSelectProps {
  pairs: readonly PairSummary[];
  value: number;
  onChange: (pair: number) => void;
}

export function FigureButton(props: FigureButtonProps) {
  if ("disabledReason" in props) {
    return (
      <DisabledButton variant="quiet" size="sm" icon={DownloadIcon} reason={props.disabledReason}>
        {FIGURE_BUTTON_LABEL}
      </DisabledButton>
    );
  }

  return (
    <Button variant="quiet" size="sm" icon={DownloadIcon} isPending={props.isPending} onPress={props.onDownload}>
      {FIGURE_BUTTON_LABEL}
    </Button>
  );
}

export type FigureButtonProps = { onDownload: () => void; isPending: boolean } | { disabledReason: string };

export interface ChoiceProps<K extends string> {
  value: K;
  onChange: (value: K) => void;
}

function ChoiceGroup<K extends string>({ label, choices, labels, value, onChange }: ChoiceGroupProps<K>) {
  const select = useCallback(
    (keys: Set<Key>) => {
      const next = choices.find((choice) => keys.has(choice));

      if (next !== undefined) {
        onChange(next);
      }
    },
    [choices, onChange],
  );

  const selected = useMemo(() => [value], [value]);

  return (
    <ToggleButtonGroup aria-label={label} selectedKeys={selected} onSelectionChange={select}>
      {choices.map((choice) => (
        <ToggleButton key={choice} id={choice} label={labels[choice]} />
      ))}
    </ToggleButtonGroup>
  );
}

interface ChoiceGroupProps<K extends string> extends ChoiceProps<K> {
  label: string;
  choices: readonly K[];
  labels: Record<K, string>;
}
