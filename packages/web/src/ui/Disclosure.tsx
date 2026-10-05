import { cn } from "cn";
import type { ReactNode } from "react";
import {
  Button as AriaButton,
  Disclosure as AriaDisclosure,
  DisclosurePanel,
  type DisclosureProps as AriaDisclosureProps,
  Heading,
} from "react-aria-components";
import ChevronIcon from "~icons/lucide/chevron-right";

import { InfoButton } from "./InfoButton";
import { focusRing } from "./styles";

export function Disclosure({ title, info, className, children, ...props }: DisclosureProps) {
  return (
    <AriaDisclosure {...props} className={cn("group/disclosure flex flex-col", className)}>
      <Heading className="flex items-center gap-1">
        <AriaButton
          slot="trigger"
          className={cn(
            "rounded-control text-ink -mx-1 flex h-8 cursor-default items-center gap-1 px-1 text-sm font-semibold select-none",
            "data-hovered:bg-ink/8 data-pressed:bg-ink/14 transition-colors duration-150 motion-reduce:transition-none",
            focusRing,
          )}
        >
          <ChevronIcon
            aria-hidden
            className="text-ink-muted shrink-0 transition-transform duration-150 group-data-expanded/disclosure:rotate-90 motion-reduce:transition-none"
          />
          {title}
        </AriaButton>
        {info === undefined ? null : <InfoButton topic={title}>{info}</InfoButton>}
      </Heading>
      <DisclosurePanel className="flex flex-col gap-4 group-data-expanded/disclosure:pt-2">{children}</DisclosurePanel>
    </AriaDisclosure>
  );
}

export interface DisclosureProps extends Omit<AriaDisclosureProps, "children" | "className"> {
  title: string;
  info?: ReactNode;
  className?: string;
  children: ReactNode;
}
