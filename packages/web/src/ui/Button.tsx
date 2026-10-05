import type { VariantProps } from "class-variance-authority";
import { cn } from "cn";
import { Button as AriaButton, type ButtonProps as AriaButtonProps, composeRenderProps } from "react-aria-components";

import type { IconComponent } from "./icon";
import { buttonStyle } from "./styles";

export function Button({ variant, size, icon: Icon, className, children, ...props }: ButtonProps) {
  return (
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
}

export interface ButtonProps extends AriaButtonProps, Omit<VariantProps<typeof buttonStyle>, "iconOnly"> {
  icon?: IconComponent;
}
