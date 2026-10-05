import type { NumberSetting } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import { type FieldPathByValue, useController, useFormContext } from "react-hook-form";

import { NumberField } from "../ui/NumberField";
import { errorProps } from "../workspace/fieldErrors";
import { draftError, type SettingsDraft } from "./draft";
import { SettingInfo } from "./SettingInfo";

const INTEGER_FORMAT: Intl.NumberFormatOptions = { maximumFractionDigits: 0, useGrouping: false };

const DECIMAL_FORMAT: Intl.NumberFormatOptions = { maximumFractionDigits: 12, useGrouping: false };

export type DraftFieldName = FieldPathByValue<SettingsDraft, number>;

export function DraftNumberField({ name, label, setting, error, onCommit, className }: DraftNumberFieldProps) {
  const { control } = useFormContext<SettingsDraft>();
  const { field } = useController({ name, control });
  const { onChange } = field;

  const change = useCallback(
    (value: number) => {
      onChange(value);
      onCommit(value);
    },
    [onChange, onCommit],
  );

  const info = useMemo(() => <SettingInfo setting={setting} />, [setting]);

  return (
    <NumberField
      label={label}
      info={info}
      value={field.value}
      onChange={change}
      onBlur={field.onBlur}
      isDisabled={!setting.applies}
      isRequired
      {...errorProps(draftError(field.value) ?? error)}
      formatOptions={setting.integer ? INTEGER_FORMAT : DECIMAL_FORMAT}
      {...(setting.step === null ? undefined : { step: setting.step })}
      {...(setting.minExclusive ? undefined : { minValue: setting.min })}
      {...(setting.max === null ? undefined : { maxValue: setting.max })}
      {...(className === undefined ? undefined : { className })}
    />
  );
}

export interface DraftNumberFieldProps {
  name: DraftFieldName;
  label: string;
  setting: NumberSetting;
  error: string | undefined;
  onCommit: (value: number) => void;
  className?: string;
}
