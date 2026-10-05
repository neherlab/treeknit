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

import { fieldClassName, inputClassName, labelClassName, optionClassName, popoverClassName } from "./fieldStyles";

export function Select<K extends string>({ label, options, value, onChange }: SelectProps<K>) {
  return (
    <AriaSelect value={value} onChange={onChange} className={fieldClassName}>
      <Label className={labelClassName}>{label}</Label>
      <AriaButton className={`${inputClassName} flex w-40 items-center justify-between gap-2 text-left`}>
        <SelectValue />
        <ChevronIcon aria-hidden />
      </AriaButton>
      <Popover className={popoverClassName}>
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
    <ListBoxItem id={option.id} className={optionClassName}>
      {option.name}
    </ListBoxItem>
  );
}
