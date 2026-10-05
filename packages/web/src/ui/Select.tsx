import {
  Button as AriaButton,
  type Key,
  Label,
  ListBox,
  ListBoxItem,
  Popover,
  Select as AriaSelect,
  SelectValue,
} from "react-aria-components";
import ChevronIcon from "~icons/lucide/chevron-down";

import { fieldClassName, inputClassName, labelClassName } from "./fieldStyles";

export function Select<K extends string>({ label, options, value, onChange }: SelectProps<K>) {
  return (
    <AriaSelect value={value} onChange={onChange} className={fieldClassName}>
      <Label className={labelClassName}>{label}</Label>
      <AriaButton className={`${inputClassName} flex w-40 items-center justify-between gap-2 text-left`}>
        <SelectValue />
        <ChevronIcon aria-hidden />
      </AriaButton>
      <Popover className="border-rule bg-panel min-w-(--trigger-width) rounded-md border py-1 shadow-md">
        <ListBox items={options} className="outline-none">
          {renderOption}
        </ListBox>
      </Popover>
    </AriaSelect>
  );
}

export interface SelectProps<K extends string> {
  label: string;
  options: readonly SelectOption<K>[];
  value: K;
  onChange: (key: Key | null) => void;
}

export interface SelectOption<K extends string> {
  id: K;
  name: string;
}

function renderOption<K extends string>(option: SelectOption<K>) {
  return (
    <ListBoxItem
      id={option.id}
      className="text-ink data-focused:bg-accent data-focused:text-accent-ink cursor-default px-3 py-1 font-mono text-sm outline-none data-selected:font-medium"
    >
      {option.name}
    </ListBoxItem>
  );
}
