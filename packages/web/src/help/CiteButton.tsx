import { cn } from "cn";
import { Dialog, DialogTrigger, Heading, Popover } from "react-aria-components";
import QuoteIcon from "~icons/lucide/quote";

import { Button, type ButtonProps } from "../ui/Button";
import { popoverStyle } from "../ui/styles";
import { Citation } from "./Citation";

const POPOVER_OFFSET_PX = 6;

export function CiteButton({ variant, size, placement }: CiteButtonProps) {
  return (
    <DialogTrigger>
      <Button variant={variant} size={size} icon={QuoteIcon}>
        Cite
      </Button>
      <Popover
        placement={placement}
        offset={POPOVER_OFFSET_PX}
        className={cn(popoverStyle, "w-[min(36rem,calc(100vw-2rem))] overflow-y-auto")}
      >
        <Dialog className="flex flex-col gap-3 p-4 outline-hidden">
          <Heading slot="title" className="text-base font-semibold">
            Cite TreeKnit
          </Heading>
          <p className="text-sm">If you use TreeKnit in your work, please cite:</p>
          <Citation />
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}

export interface CiteButtonProps {
  variant?: ButtonProps["variant"];
  size?: ButtonProps["size"];
  placement: "bottom end" | "bottom start";
}
