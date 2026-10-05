import { cn } from "cn";
import {
  Button as AriaButton,
  Group,
  Input,
  NumberField as AriaNumberField,
  type NumberFieldProps as AriaNumberFieldProps,
} from "react-aria-components";
import DecreaseIcon from "~icons/lucide/chevron-down";
import IncreaseIcon from "~icons/lucide/chevron-up";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import type { IconComponent } from "./icon";
import { fieldStyle, inputStyle } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";

export function NumberField({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
  placeholder,
  className,
  ...props
}: NumberFieldProps) {
  return (
    <AriaNumberField
      {...props}
      isInvalid={props.isInvalid ?? errorMessage !== undefined}
      className={cn(fieldStyle, className)}
    >
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      <Group className={cn(inputStyle(), "flex items-stretch overflow-hidden px-0")}>
        <Input
          placeholder={placeholder ?? ""}
          className="placeholder:text-ink-muted min-w-0 flex-1 bg-transparent px-2.5 tabular-nums outline-hidden"
        />
        <div className="border-rule flex w-6 shrink-0 flex-col border-l">
          <StepButton slot="increment" label="Increase" icon={IncreaseIcon} />
          <StepButton slot="decrement" label="Decrease" icon={DecreaseIcon} />
        </div>
      </Group>
      <FieldDescription>{description}</FieldDescription>
      <FieldErrorMessage>{errorMessage}</FieldErrorMessage>
    </AriaNumberField>
  );
}

export interface NumberFieldProps extends Omit<AriaNumberFieldProps, "className" | "children">, FieldProps {
  placeholder?: string;
  className?: string;
}

function StepButton({ slot, label, icon: Icon }: StepButtonProps) {
  return (
    <TooltipTrigger tooltip={label} placement="right">
      <AriaButton
        slot={slot}
        className={cn(
          "text-ink-muted flex flex-1 cursor-default items-center justify-center text-xs outline-hidden",
          "data-hovered:bg-ink/8 data-hovered:text-ink data-pressed:bg-ink/14",
          "data-disabled:text-ink-muted/50 data-disabled:cursor-not-allowed",
        )}
      >
        <Icon aria-hidden />
      </AriaButton>
    </TooltipTrigger>
  );
}

interface StepButtonProps {
  slot: "increment" | "decrement";
  label: string;
  icon: IconComponent;
}
