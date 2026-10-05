import { cn } from "cn";
import type { ReactNode } from "react";
import { SwitchButton, SwitchField, type SwitchFieldProps } from "react-aria-components";

import { FieldDescription } from "./Field";
import { InfoButton } from "./InfoButton";
import { groupFocusRing } from "./styles";

export function Switch({ label, info, description, className, ...props }: SwitchProps) {
  return (
    <SwitchField {...props} className={cn("flex min-w-0 flex-col gap-1", className)}>
      <div className="flex items-center gap-1">
        <SwitchButton className="group text-ink data-disabled:text-ink-muted flex cursor-default items-center gap-2 text-sm outline-hidden data-disabled:cursor-not-allowed">
          <span
            className={cn(
              "rounded-control border-ink-muted bg-ground inline-flex h-4.5 w-8 shrink-0 items-center border p-0.5",
              "transition-colors duration-150 motion-reduce:transition-none",
              "group-data-hovered:border-ink group-data-selected:border-focus group-data-selected:bg-focus",
              "group-data-disabled:opacity-50",
              groupFocusRing,
            )}
          >
            <span
              className={cn(
                "bg-ink-muted rounded-inner size-3 transition-transform duration-150 motion-reduce:transition-none",
                "group-data-selected:bg-ground group-data-selected:translate-x-3.5",
              )}
            />
          </span>
          {label}
        </SwitchButton>
        {info === undefined ? null : <InfoButton topic={label}>{info}</InfoButton>}
      </div>
      {description === undefined || description === null ? null : (
        <div className="pl-10">
          <FieldDescription>{description}</FieldDescription>
        </div>
      )}
    </SwitchField>
  );
}

export interface SwitchProps extends Omit<SwitchFieldProps, "className" | "children"> {
  label: string;
  info?: ReactNode;
  description?: ReactNode;
  className?: string;
}
