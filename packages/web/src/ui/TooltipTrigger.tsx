import type { ReactNode } from "react";
import {
  OverlayArrow,
  Tooltip,
  TooltipTrigger as AriaTooltipTrigger,
  type TooltipProps,
  type TooltipTriggerComponentProps,
} from "react-aria-components";

const TOOLTIP_DELAY_MS = 600;

const TOOLTIP_OFFSET_PX = 6;

export function TooltipTrigger({
  tooltip,
  placement = "top",
  repeatsName = false,
  children,
  ...props
}: TooltipTriggerProps) {
  return (
    <AriaTooltipTrigger delay={TOOLTIP_DELAY_MS} {...props}>
      {children}
      <Tooltip
        placement={placement}
        offset={TOOLTIP_OFFSET_PX}
        className="rounded-control bg-ink text-ground max-w-64 px-2 py-1 text-xs shadow-md"
      >
        <OverlayArrow className="text-ink data-[placement=bottom]:rotate-180 data-[placement=left]:-rotate-90 data-[placement=right]:rotate-90">
          <svg width={8} height={4} viewBox="0 0 8 4" className="block fill-current">
            <path d="M0 0 L4 4 L8 0" />
          </svg>
        </OverlayArrow>
        {repeatsName ? <span aria-hidden>{tooltip}</span> : tooltip}
      </Tooltip>
    </AriaTooltipTrigger>
  );
}

export interface TooltipTriggerProps extends TooltipTriggerComponentProps {
  tooltip: ReactNode;
  placement?: TooltipProps["placement"];
  repeatsName?: boolean;
}
