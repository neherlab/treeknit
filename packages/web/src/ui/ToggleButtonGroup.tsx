import { cn } from "cn";
import type { ComponentType, SVGProps } from "react";
import {
  composeRenderProps,
  ToggleButton as AriaToggleButton,
  ToggleButtonGroup as AriaToggleButtonGroup,
  type Key,
  type ToggleButtonProps as AriaToggleButtonProps,
  type ToggleButtonGroupProps as AriaToggleButtonGroupProps,
} from "react-aria-components";

import { focusRing } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";

export function ToggleButtonGroup({
  selectionMode = "single",
  disallowEmptySelection = true,
  className,
  ...props
}: AriaToggleButtonGroupProps) {
  return (
    <AriaToggleButtonGroup
      {...props}
      selectionMode={selectionMode}
      disallowEmptySelection={disallowEmptySelection}
      className={composeRenderProps(className, (custom) =>
        cn(
          "rounded-control border-rule bg-ground inline-flex w-fit items-center gap-0.5 border p-0.5",
          "data-[orientation=vertical]:flex-col data-[orientation=vertical]:items-stretch",
          "data-disabled:opacity-50",
          custom,
        ),
      )}
    />
  );
}

export function ToggleButton({ label, icon: Icon, iconOnly = false, ...props }: ToggleButtonProps) {
  const button = (
    <AriaToggleButton
      {...props}
      aria-label={label}
      className={cn(
        "text-ink-muted rounded-inner inline-flex h-7 shrink-0 cursor-pointer items-center justify-center gap-1.5 text-sm whitespace-nowrap select-none",
        "transition-colors duration-150 motion-reduce:transition-none [&_svg]:shrink-0",
        "data-hovered:not-data-selected:bg-pane data-hovered:not-data-selected:text-ink data-pressed:not-data-selected:bg-pane",
        "data-selected:bg-ink data-selected:text-ground data-selected:font-semibold",
        "data-disabled:text-ink-muted data-disabled:cursor-not-allowed",
        iconOnly ? "w-7" : "px-2.5",
        focusRing,
      )}
    >
      {Icon === undefined ? null : <Icon aria-hidden />}
      {iconOnly ? null : label}
    </AriaToggleButton>
  );

  if (!iconOnly) {
    return button;
  }

  return (
    <TooltipTrigger tooltip={label} repeatsName>
      {button}
    </TooltipTrigger>
  );
}

export interface ToggleButtonProps extends Pick<AriaToggleButtonProps, "isDisabled"> {
  id: Key;
  label: string;
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  iconOnly?: boolean;
}
