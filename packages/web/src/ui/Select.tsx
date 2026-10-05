import { cn } from "cn";
import { useCallback } from "react";
import {
  Button as AriaButton,
  type Key,
  ListBox,
  ListBoxItem,
  Popover,
  Select as AriaSelect,
  type SelectProps as AriaSelectProps,
  SelectValue,
} from "react-aria-components";
import CheckIcon from "~icons/lucide/check";
import ChevronIcon from "~icons/lucide/chevron-down";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import { fieldStyle, inputStyle, listBoxStyle, optionStyle, popoverStyle } from "./styles";

const POPOVER_OFFSET_PX = 4;

export function Select<K extends string>({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
  options,
  value,
  onChange,
  className,
  ...props
}: SelectProps<K>) {
  const select = useCallback(
    (key: Key | null) => {
      const option = options.find((candidate) => candidate.id === key);

      if (option !== undefined) {
        onChange(option.id);
      }
    },
    [options, onChange],
  );

  return (
    <AriaSelect
      {...props}
      isInvalid={props.isInvalid ?? errorMessage !== undefined}
      value={value}
      onChange={select}
      className={cn(fieldStyle, "group/select", className)}
    >
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      <AriaButton
        className={cn(
          inputStyle(),
          "group-data-invalid/select:border-danger flex cursor-default items-center justify-between gap-2 text-left",
        )}
      >
        <SelectValue className="data-placeholder:text-ink-muted truncate" />
        <ChevronIcon aria-hidden className="text-ink-muted shrink-0" />
      </AriaButton>
      <FieldDescription>{description}</FieldDescription>
      <FieldErrorMessage>{errorMessage}</FieldErrorMessage>
      <Popover offset={POPOVER_OFFSET_PX} className={popoverStyle}>
        <ListBox items={options} className={listBoxStyle}>
          {renderOption}
        </ListBox>
      </Popover>
    </AriaSelect>
  );
}

export type SelectProps<K extends string> = Omit<
  AriaSelectProps,
  "className" | "children" | "value" | "onChange" | "selectionMode"
> &
  FieldProps & {
    options: readonly SelectOption<K>[];
    value: K;
    onChange: (value: K) => void;
    className?: string;
  };

export interface SelectOption<K extends string> {
  id: K;
  label: string;
}

function renderOption<K extends string>(option: SelectOption<K>) {
  return (
    <ListBoxItem id={option.id} textValue={option.label} className={optionStyle}>
      {({ isSelected }) => (
        <>
          <span className="flex-1 truncate">{option.label}</span>
          <CheckIcon aria-hidden className={cn("shrink-0", !isSelected && "invisible")} />
        </>
      )}
    </ListBoxItem>
  );
}
