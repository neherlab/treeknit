import { cn } from "cn";
import type { ReactNode } from "react";
import {
  RadioButton,
  RadioField,
  type RadioFieldProps,
  RadioGroup as AriaRadioGroup,
  type RadioGroupProps as AriaRadioGroupProps,
  Text,
} from "react-aria-components";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import { descriptionStyle, groupFocusRing } from "./styles";

export function RadioGroup({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
  children,
  className,
  ...props
}: RadioGroupProps) {
  return (
    <AriaRadioGroup {...props} className={cn("group/radios flex min-w-0 flex-col gap-2", className)}>
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      <div className="flex flex-col gap-x-4 gap-y-1.5 group-data-[orientation=horizontal]/radios:flex-row group-data-[orientation=horizontal]/radios:flex-wrap">
        {children}
      </div>
      <FieldDescription>{description}</FieldDescription>
      <FieldErrorMessage>{errorMessage}</FieldErrorMessage>
    </AriaRadioGroup>
  );
}

export interface RadioGroupProps extends Omit<AriaRadioGroupProps, "className" | "children">, FieldProps {
  children: ReactNode;
  className?: string;
}

export function Radio({ description, children, ...props }: RadioProps) {
  return (
    <RadioField {...props} className="flex flex-col gap-0.5">
      <RadioButton className="group text-ink data-disabled:text-ink-muted flex w-fit cursor-default items-center gap-2 text-sm outline-hidden data-disabled:cursor-not-allowed">
        <span
          className={cn(
            "border-ink-muted bg-ground size-4 shrink-0 rounded-full border",
            "transition-[border-color,border-width] duration-150 motion-reduce:transition-none",
            "group-data-hovered:border-ink group-data-selected:border-ink group-data-selected:border-[5px]",
            "group-data-invalid:border-danger group-data-disabled:opacity-50",
            groupFocusRing,
          )}
        />
        {children}
      </RadioButton>
      {description === undefined ? null : (
        <Text slot="description" className={cn(descriptionStyle, "pl-6")}>
          {description}
        </Text>
      )}
    </RadioField>
  );
}

export interface RadioProps extends Pick<RadioFieldProps, "value" | "isDisabled"> {
  description?: ReactNode;
  children: ReactNode;
}
