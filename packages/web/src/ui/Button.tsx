import type { VariantProps } from "class-variance-authority";
import { cn } from "cn";
import type { ComponentType, ReactNode, SVGProps } from "react";
import { Button as AriaButton, type ButtonProps as AriaButtonProps, composeRenderProps } from "react-aria-components";

import { buttonStyle } from "./styles";
import { TooltipTrigger, type TooltipTriggerProps } from "./TooltipTrigger";

export function Button({
  variant,
  size,
  icon: Icon,
  tooltip,
  tooltipPlacement,
  className,
  children,
  ...props
}: ButtonProps) {
  const button = (
    <AriaButton
      {...props}
      className={composeRenderProps(className, (custom) => cn(buttonStyle({ variant, size }), custom))}
    >
      {composeRenderProps(children, (content) => (
        <>
          {Icon === undefined ? null : <Icon aria-hidden />}
          {content}
        </>
      ))}
    </AriaButton>
  );

  if (tooltip === undefined) {
    return button;
  }

  return (
    <TooltipTrigger tooltip={tooltip} placement={tooltipPlacement}>
      {button}
    </TooltipTrigger>
  );
}

export interface ButtonProps extends AriaButtonProps, Omit<VariantProps<typeof buttonStyle>, "iconOnly"> {
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  tooltip?: ReactNode;
  tooltipPlacement?: TooltipTriggerProps["placement"];
}
