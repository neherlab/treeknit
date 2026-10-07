import { cn } from "cn";
import type { ComponentType, SVGProps } from "react";
import {
  Button as AriaButton,
  ButtonContext,
  Group,
  Input,
  NumberField as AriaNumberField,
  type NumberFieldProps as AriaNumberFieldProps,
  useSlottedContext,
} from "react-aria-components";
import DecreaseIcon from "~icons/lucide/chevron-down";
import IncreaseIcon from "~icons/lucide/chevron-up";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import { fieldValidity } from "./fieldValidity";
import { fieldStyle, inputStyle } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";

export function NumberField({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
  isInvalid,
  placeholder,
  className,
  ...props
}: NumberFieldProps) {
  return (
    <AriaNumberField {...props} {...fieldValidity(isInvalid, errorMessage)} className={cn(fieldStyle, className)}>
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      <Group className={cn(inputStyle(), "flex items-stretch overflow-hidden px-0")}>
        <Input
          {...(placeholder === undefined ? undefined : { placeholder })}
          className="placeholder:text-ink-muted min-w-0 flex-1 bg-transparent px-2.5 tabular-nums outline-hidden"
        />
        <div className="border-rule flex w-6 shrink-0 flex-col border-l">
          <StepButton slot="increment" icon={IncreaseIcon} />
          <StepButton slot="decrement" icon={DecreaseIcon} />
        </div>
      </Group>
      <FieldDescription>{description}</FieldDescription>
      <FieldErrorMessage>{errorMessage}</FieldErrorMessage>
    </AriaNumberField>
  );
}

export type NumberFieldProps = Omit<AriaNumberFieldProps, "className" | "children"> &
  FieldProps & {
    placeholder?: string;
    className?: string;
  };

function StepButton({ slot, icon: Icon }: StepButtonProps) {
  const ariaLabel = useSlottedContext(ButtonContext, slot)?.["aria-label"];
  const label = ariaLabel === undefined || ariaLabel === "" ? null : ariaLabel;

  const button = (
    <AriaButton
      slot={slot}
      className={cn(
        "text-ink-muted flex flex-1 cursor-pointer items-center justify-center text-xs outline-hidden",
        "data-hovered:bg-ink/8 data-hovered:text-ink data-pressed:bg-ink/14",
        "data-disabled:text-ink-muted/50 data-disabled:cursor-not-allowed",
      )}
    >
      <Icon aria-hidden />
    </AriaButton>
  );

  return label === null ? (
    button
  ) : (
    <TooltipTrigger tooltip={label} repeatsName placement="right">
      {button}
    </TooltipTrigger>
  );
}

interface StepButtonProps {
  slot: "increment" | "decrement";
  icon: ComponentType<SVGProps<SVGSVGElement>>;
}
