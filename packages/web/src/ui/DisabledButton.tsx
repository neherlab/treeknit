import type { VariantProps } from "class-variance-authority";
import { cn } from "cn";
import type { ComponentType, ReactNode, SVGProps } from "react";
import { Focusable } from "react-aria-components";

import { buttonStyle, nativeFocusRing } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";

export function DisabledButton({ reason, icon: Icon, variant, size, className, children }: DisabledButtonProps) {
  return (
    <TooltipTrigger tooltip={reason}>
      <Focusable>
        <button
          type="button"
          aria-disabled
          data-disabled
          className={cn(buttonStyle({ variant, size }), nativeFocusRing, className)}
        >
          {Icon === undefined ? null : <Icon aria-hidden />}
          {children}
        </button>
      </Focusable>
    </TooltipTrigger>
  );
}

export interface DisabledButtonProps extends Omit<VariantProps<typeof buttonStyle>, "iconOnly"> {
  reason: string;
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  className?: string;
  children: ReactNode;
}
