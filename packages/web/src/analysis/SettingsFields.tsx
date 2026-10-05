import type { ResolveMode } from "@neherlab/treeknit-wasm";
import { useCallback } from "react";
import type { Key } from "react-aria-components";
import { type FieldPathByValue, useController } from "react-hook-form";

import { NumberField } from "../ui/NumberField";
import { Select, type SelectOption } from "../ui/Select";
import type { AnalysisForm } from "./request";

const RESOLVE_OPTIONS = {
  matched: { id: "matched", name: "matched" },
  strict: { id: "strict", name: "strict" },
  liberal: { id: "liberal", name: "liberal" },
  none: { id: "none", name: "none" },
} as const satisfies { [M in ResolveMode]: SelectOption<M> };

const RESOLVE_LIST = Object.values(RESOLVE_OPTIONS);

const INTEGER_FORMAT: Intl.NumberFormatOptions = { maximumFractionDigits: 0, useGrouping: false };

const DECIMAL_FORMAT: Intl.NumberFormatOptions = { maximumFractionDigits: 6, useGrouping: false };

export function SettingsFields() {
  return (
    <fieldset className="flex flex-col gap-4">
      <legend className="mb-3 text-base font-semibold">Settings</legend>
      <div className="flex flex-wrap gap-x-6 gap-y-4">
        <NumberSetting name="settings.gamma" label="γ, cost of a reassortment" format={DECIMAL_FORMAT} />
        <ResolveSetting />
        <NumberSetting name="settings.seed" label="Seed" format={INTEGER_FORMAT} />
      </div>
    </fieldset>
  );
}

function NumberSetting({ name, label, format }: NumberSettingProps) {
  const { field } = useController<AnalysisForm, NumberPath>({ name });

  return (
    <NumberField
      label={label}
      value={field.value ?? Number.NaN}
      onChange={field.onChange}
      onBlur={field.onBlur}
      formatOptions={format}
      minValue={0}
      isRequired
    />
  );
}

interface NumberSettingProps {
  name: NumberPath;
  label: string;
  format: Intl.NumberFormatOptions;
}

type NumberPath = FieldPathByValue<AnalysisForm, number | undefined>;

function ResolveSetting() {
  const { field } = useController<AnalysisForm, "settings.resolve">({ name: "settings.resolve" });
  const { onChange } = field;

  const select = useCallback(
    (key: Key | null) => {
      const mode = RESOLVE_LIST.find((option) => option.id === key)?.id;

      if (mode !== undefined) {
        onChange(mode);
      }
    },
    [onChange],
  );

  return (
    <Select
      label="Resolution"
      options={RESOLVE_LIST}
      value={field.value ?? RESOLVE_OPTIONS.matched.id}
      onChange={select}
    />
  );
}
