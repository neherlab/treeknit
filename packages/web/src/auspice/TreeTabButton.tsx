import type { ComponentType, ReactNode, SVGProps } from "react";
import { Button, type ButtonProps } from "react-aria-components";

import { TooltipTrigger } from "../ui/TooltipTrigger";

const TREE_TAB_BUTTON_STYLE = [
  "flex h-[22px] cursor-pointer items-center gap-1 rounded-b-[3px] border border-t-0 border-[#ccc] bg-white px-1.5",
  "text-[12px] leading-[15px] font-normal text-[#333] uppercase select-none",
  "[&_svg]:size-[11px] [&_svg]:shrink-0 [&_svg]:text-[#888]",
  "data-hovered:bg-[#f5f5f5] data-pressed:bg-[#f5f5f5] data-disabled:cursor-not-allowed data-disabled:text-[#bbb]",
  "outline-hidden data-focus-visible:outline-2 data-focus-visible:outline-offset-1 data-focus-visible:outline-[#5097ba]",
].join(" ");

export function TreeTabButton({ icon: Icon, label, tooltip, iconOnly = false, ...props }: TreeTabButtonProps) {
  const button = (
    <Button {...props} {...(iconOnly ? { "aria-label": label } : {})} className={TREE_TAB_BUTTON_STYLE}>
      {Icon === undefined ? null : <Icon aria-hidden />}
      {iconOnly ? null : label}
    </Button>
  );

  if (tooltip === undefined && !iconOnly) {
    return button;
  }

  return (
    <TooltipTrigger tooltip={tooltip ?? label} placement="bottom" repeatsName={tooltip === undefined}>
      {button}
    </TooltipTrigger>
  );
}

export interface TreeTabButtonProps extends Pick<ButtonProps, "isDisabled" | "onPress"> {
  label: string;
  tooltip?: ReactNode;
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  iconOnly?: boolean;
}

export function TreeTabButtonGroup({ children }: { children: ReactNode }) {
  return <div className="flex shrink-0 gap-1 self-start">{children}</div>;
}
