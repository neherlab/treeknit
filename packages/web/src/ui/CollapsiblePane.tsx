import { cva } from "class-variance-authority";
import { cn } from "cn";
import { type ReactNode, useCallback, useId, useLayoutEffect, useRef, useState } from "react";
import { Button } from "react-aria-components";
import ChevronLeftIcon from "~icons/lucide/chevron-left";
import ChevronRightIcon from "~icons/lucide/chevron-right";

import { focusRing } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";
import { useEscapeKey } from "./useEscapeKey";

export type PaneWidthPx = 260 | 320 | 336;

const PANE_WIDTHS: Record<PaneWidthPx, string> = {
  260: "w-[260px]",
  320: "w-[320px]",
  336: "w-[336px]",
};

const railStyle = cva(
  [
    "absolute top-0 bottom-0 max-w-[calc(100cqw-40px)]",
    "transition-transform duration-300 ease-out will-change-transform motion-reduce:transition-none",
  ],
  {
    variants: {
      side: { left: "left-0", right: "right-0" },
      isOpen: { true: "translate-x-0", false: "" },
    },
    compoundVariants: [
      { side: "left", isOpen: false, className: "-translate-x-full" },
      { side: "right", isOpen: false, className: "translate-x-full" },
    ],
  },
);

const paneStyle = cva("h-full overflow-x-hidden overflow-y-auto", {
  variants: {
    look: {
      app: "bg-pane border-rule",
      auspice: "bg-[#f2f2f2]",
    },
    side: { left: "", right: "" },
    isOverlay: { true: "", false: "" },
  },
  compoundVariants: [
    { look: "app", side: "left", className: "border-r" },
    { look: "app", side: "right", className: "border-l" },
    { look: "app", isOverlay: true, className: "shadow-lg" },
    { look: "auspice", side: "left", isOverlay: false, className: "shadow-[inset_-3px_0_3px_-3px_rgba(0,0,0,0.2)]" },
    { look: "auspice", side: "right", isOverlay: false, className: "shadow-[inset_3px_0_3px_-3px_rgba(0,0,0,0.2)]" },
    { look: "auspice", side: "left", isOverlay: true, className: "shadow-[2px_0_8px_rgba(0,0,0,0.2)]" },
    { look: "auspice", side: "right", isOverlay: true, className: "shadow-[-2px_0_8px_rgba(0,0,0,0.2)]" },
  ],
});

const tabStyle = cva(["absolute flex h-11 cursor-pointer items-center justify-center text-[10px]", focusRing], {
  variants: {
    look: {
      app: "bg-pane border-rule text-ink-muted data-hovered:text-ink top-1/2 w-[13px] -translate-y-1/2 border",
      auspice:
        "top-0 w-[15px] bg-[#f2f2f2] text-[#333] data-focus-visible:outline-[#5097ba] data-hovered:text-[#5097ba]",
    },
    side: { left: "", right: "" },
    isOverlay: { true: "", false: "" },
  },
  compoundVariants: [
    { look: "app", side: "left", className: "left-[calc(100%-1px)] rounded-r-[6px] border-l-0" },
    { look: "app", side: "right", className: "right-[calc(100%-1px)] rounded-l-[6px] border-r-0" },
    {
      look: "auspice",
      side: "left",
      className: "left-[calc(100%-3px)] rounded-r-[6px] pl-[3px] [clip-path:inset(-12px_-12px_-12px_0)]",
    },
    {
      look: "auspice",
      side: "right",
      className: "right-[calc(100%-3px)] rounded-l-[6px] pr-[3px] [clip-path:inset(-12px_0_-12px_-12px)]",
    },
    {
      look: "auspice",
      side: "left",
      isOverlay: false,
      className:
        "shadow-[inset_-3px_0_3px_-3px_rgba(0,0,0,0.2),inset_0_3px_3px_-3px_rgba(0,0,0,0.2),inset_0_-3px_3px_-3px_rgba(0,0,0,0.2)]",
    },
    {
      look: "auspice",
      side: "right",
      isOverlay: false,
      className:
        "shadow-[inset_3px_0_3px_-3px_rgba(0,0,0,0.2),inset_0_3px_3px_-3px_rgba(0,0,0,0.2),inset_0_-3px_3px_-3px_rgba(0,0,0,0.2)]",
    },
    { look: "auspice", side: "left", isOverlay: true, className: "shadow-[2px_0_8px_rgba(0,0,0,0.2)]" },
    { look: "auspice", side: "right", isOverlay: true, className: "shadow-[-2px_0_8px_rgba(0,0,0,0.2)]" },
    { look: "app", isOverlay: true, className: "shadow-lg" },
  ],
});

export function CollapsiblePane({ children, className, ...pane }: CollapsiblePaneProps) {
  const { side, isOverlay, isOpen, onOpenChange } = pane;

  const close = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  const sidePane = <SidePane {...pane} />;

  return (
    <div className={cn("@container relative isolate flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}>
      {side === "left" ? sidePane : null}
      {isOverlay && isOpen ? (
        <div aria-hidden role="presentation" className="absolute inset-0 z-2" onPointerDown={close} />
      ) : null}
      <div className="relative z-0 flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">{children}</div>
      {side === "right" ? sidePane : null}
    </div>
  );
}

export interface CollapsiblePaneProps extends SidePaneProps {
  children: ReactNode;
  className?: string;
}

export function SidePane({
  side,
  width,
  title,
  name,
  shortcut,
  isOpen,
  onOpenChange,
  isOverlay,
  look,
  pane,
  className,
}: SidePaneProps & { className?: string }) {
  const paneId = useId();
  const railRef = useRef<HTMLDivElement>(null);
  const paneRef = useRef<HTMLElement>(null);
  const tabRef = useRef<HTMLButtonElement>(null);
  const reserved = useReservedSpace(railRef, isOpen);
  const label = `${isOpen ? "Hide" : "Show"} ${name}`;
  const tooltip = shortcut === undefined ? label : `${label} (${shortcut})`;
  const Chevron = isOpen === (side === "left") ? ChevronLeftIcon : ChevronRightIcon;

  useLayoutEffect(() => {
    if (!isOpen && paneRef.current?.contains(document.activeElement) === true) {
      tabRef.current?.focus({ preventScroll: true });
    }
  }, [isOpen]);

  const toggle = useCallback(() => {
    onOpenChange(!isOpen);
  }, [isOpen, onOpenChange]);

  const close = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  const escapeProps = useEscapeKey(isOverlay && isOpen ? close : null);

  return (
    <div className={cn("relative z-3 shrink-0", !isOverlay && reserved ? PANE_WIDTHS[width] : "w-0", className)}>
      <div ref={railRef} {...escapeProps} className={cn(railStyle({ side, isOpen }), PANE_WIDTHS[width])}>
        <aside
          ref={paneRef}
          id={paneId}
          aria-label={title}
          inert={!isOpen}
          className={paneStyle({ look, side, isOverlay })}
        >
          {pane}
        </aside>
        <TooltipTrigger tooltip={tooltip} placement={side === "left" ? "right" : "left"}>
          <Button
            ref={tabRef}
            aria-label={label}
            aria-expanded={isOpen}
            aria-controls={paneId}
            onPress={toggle}
            className={tabStyle({ look, side, isOverlay })}
          >
            <Chevron aria-hidden />
          </Button>
        </TooltipTrigger>
      </div>
    </div>
  );
}

export interface SidePaneProps {
  side: "left" | "right";
  width: PaneWidthPx;
  title: string;
  name: string;
  shortcut?: string | undefined;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  isOverlay: boolean;
  look: "app" | "auspice";
  pane: ReactNode;
}

function useReservedSpace(rail: { current: HTMLElement | null }, isOpen: boolean): boolean {
  const [reserved, setReserved] = useState(isOpen);

  useLayoutEffect(() => {
    let current = true;
    const animations = rail.current?.getAnimations() ?? [];

    Promise.all(animations.map(async (animation) => animation.finished))
      .then(() => {
        if (current) {
          setReserved(isOpen);
        }

        return undefined;
      })
      .catch((cause: unknown) => {
        if (!(cause instanceof DOMException && cause.name === "AbortError")) {
          throw cause;
        }
      });

    return () => {
      current = false;
    };
  }, [rail, isOpen]);

  return reserved;
}
