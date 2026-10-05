import type { LabelMode, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";
import DownloadIcon from "~icons/lucide/download";

import { Button } from "../ui/Button";
import { Select, type SelectOption } from "../ui/Select";
import { ToggleButton, ToggleButtonGroup } from "../ui/ToggleButtonGroup";
import { LABEL_MODES, TREE_VERSIONS, X_SCALES } from "../workspace/search";

const SCALE_LABELS: Record<Scale, string> = { div: "Divergence", depth: "Cladogram" };

const VERSION_LABELS: Record<TreeVersion, string> = { input: "Input", resolved: "Resolved", imputed: "Imputed" };

const LABEL_MODE_OPTIONS: SelectOption<LabelMode>[] = LABEL_MODES.map((mode) => ({
  id: mode,
  label: { auto: "Automatic", on: "On", off: "Off" }[mode],
}));

const FIGURE_BUTTON_LABEL = "Download figure (SVG)";

export function ScaleToggle({ value, onChange }: ChoiceProps<Scale>) {
  return (
    <ChoiceGroup label="Branch scale" choices={X_SCALES} labels={SCALE_LABELS} value={value} onChange={onChange} />
  );
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

export function FigureButton({ onDownload }: FigureButtonProps) {
  return (
    <Button
      variant="quiet"
      size="sm"
      icon={DownloadIcon}
      isDisabled={onDownload === undefined}
      {...(onDownload === undefined ? undefined : { onPress: onDownload })}
    >
      {FIGURE_BUTTON_LABEL}
    </Button>
  );
}

export interface FigureButtonProps {
  onDownload?: (() => void) | undefined;
}

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
