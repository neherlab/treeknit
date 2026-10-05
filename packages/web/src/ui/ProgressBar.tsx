import { cn } from "cn";
import { useId } from "react";
import {
  Label,
  ProgressBar as AriaProgressBar,
  type ProgressBarProps as AriaProgressBarProps,
} from "react-aria-components";

export function ProgressBar({ label, labelHidden = false, className, ...props }: ProgressBarProps) {
  const hatchId = useId();

  return (
    <AriaProgressBar {...props} className={cn("flex min-w-0 flex-col gap-1.5", className)}>
      {({ percentage, valueText, isIndeterminate }) => (
        <>
          <div className={cn("flex items-baseline gap-3", labelHidden ? "justify-end" : "justify-between")}>
            <Label className={cn("text-ink truncate text-sm", labelHidden && "sr-only")}>{label}</Label>
            <span className="text-ink-muted shrink-0 text-xs tabular-nums">{valueText}</span>
          </div>
          <svg aria-hidden className="rounded-control block h-1.5 w-full overflow-hidden" width="100%" height="6">
            <defs>
              <pattern id={hatchId} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                <rect width="3" height="6" className="fill-ink-muted" />
              </pattern>
            </defs>
            <rect width="100%" height="100%" className="fill-rule" />
            {isIndeterminate ? (
              <rect width="100%" height="100%" fill={`url(#${hatchId})`} />
            ) : (
              <rect width={`${percentage ?? 0}%`} height="100%" className="fill-ink" />
            )}
          </svg>
        </>
      )}
    </AriaProgressBar>
  );
}

export interface ProgressBarProps extends Omit<AriaProgressBarProps, "className" | "children"> {
  label: string;
  labelHidden?: boolean;
  className?: string;
}
