import { cva } from "class-variance-authority";
import { cn } from "cn";
import type { ReactNode } from "react";
import { Dialog as AriaDialog, Heading, Modal, ModalOverlay, type ModalOverlayProps } from "react-aria-components";
import CloseIcon from "~icons/lucide/x";

import { IconButton } from "./IconButton";

const overlayStyle = cva("bg-ink/30 dark:bg-ground/70 fixed inset-0 z-50 flex", {
  variants: {
    placement: {
      center: "items-center justify-center p-4",
      left: "justify-start",
      right: "justify-end",
    },
  },
});

const panelStyle = cva("bg-ground text-ink flex max-h-full min-h-0 flex-col outline-hidden", {
  variants: {
    placement: {
      center: "rounded-control border-rule w-full max-w-lg border shadow-xl",
      left: "border-rule h-full w-[min(336px,100vw-48px)] border-r shadow-xl",
      right: "border-rule h-full w-[min(320px,100vw-48px)] border-l shadow-xl",
    },
  },
});

export type DialogPlacement = "center" | "left" | "right";

export function Dialog({ title, placement = "center", footer, className, children, ...props }: DialogProps) {
  return (
    <ModalOverlay {...props} isDismissable className={overlayStyle({ placement })}>
      <Modal className={cn(panelStyle({ placement }), className)}>
        <AriaDialog className="flex min-h-0 flex-1 flex-col outline-hidden">
          {({ close }) => (
            <>
              <header className="border-rule flex h-12 shrink-0 items-center gap-2 border-b pr-2 pl-4">
                <Heading slot="title" className="flex-1 truncate text-base font-semibold">
                  {title}
                </Heading>
                <IconButton label="Close" icon={CloseIcon} onPress={close} tooltipPlacement="bottom" />
              </header>
              <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
              {footer === undefined ? null : (
                <footer className="border-rule bg-pane flex shrink-0 items-center justify-end gap-2 border-t px-4 py-3">
                  {footer}
                </footer>
              )}
            </>
          )}
        </AriaDialog>
      </Modal>
    </ModalOverlay>
  );
}

export interface DialogProps extends Omit<ModalOverlayProps, "children" | "className" | "isDismissable"> {
  title: string;
  placement?: DialogPlacement;
  footer?: ReactNode;
  className?: string;
  children: ReactNode;
}
