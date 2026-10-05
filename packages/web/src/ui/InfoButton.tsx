import { cn } from "cn";
import type { ReactNode } from "react";
import { Dialog, DialogTrigger, Popover } from "react-aria-components";
import InfoIcon from "~icons/lucide/info";

import { IconButton } from "./IconButton";
import { popoverStyle } from "./styles";

const POPOVER_OFFSET_PX = 6;

export function InfoButton({ topic, children }: InfoButtonProps) {
  const label = `About ${topic}`;

  return (
    <DialogTrigger>
      <IconButton
        slot={null}
        label={label}
        icon={InfoIcon}
        size="xs"
        className="text-ink-muted data-hovered:text-ink data-pressed:text-ink"
      />
      <Popover placement="bottom start" offset={POPOVER_OFFSET_PX} className={cn(popoverStyle, "max-w-sm")}>
        <Dialog aria-label={label} className="text-ink flex flex-col gap-2 px-3 py-2.5 text-sm outline-hidden">
          {children}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}

export interface InfoButtonProps {
  topic: string;
  children: ReactNode;
}
