import type { VariantProps } from "class-variance-authority";
import { cn } from "cn";
import { Button as AriaButton, type ButtonProps as AriaButtonProps, composeRenderProps } from "react-aria-components";

import type { IconComponent } from "./icon";
import { buttonStyle } from "./styles";
import { TooltipTrigger, type TooltipTriggerProps } from "./TooltipTrigger";

export function IconButton({
  label,
  icon: Icon,
  variant = "quiet",
  size,
  tooltipPlacement,
  className,
  ...props
}: IconButtonProps) {
  return (
    <TooltipTrigger tooltip={label} repeatsName placement={tooltipPlacement}>
      <AriaButton
        {...props}
        aria-label={label}
        className={composeRenderProps(className, (custom) =>
          cn(buttonStyle({ variant, size, iconOnly: true }), custom),
        )}
      >
        <Icon aria-hidden />
      </AriaButton>
    </TooltipTrigger>
  );
}

export interface IconButtonProps
  extends Omit<AriaButtonProps, "children" | "aria-label">, Omit<VariantProps<typeof buttonStyle>, "iconOnly"> {
  label: string;
  icon: IconComponent;
  tooltipPlacement?: TooltipTriggerProps["placement"];
}
