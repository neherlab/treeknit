import { cn } from "cn";
import { type ReactNode, useLayoutEffect, useRef, useState } from "react";
import { mergeProps, useFocusable } from "react-aria";
import {
  composeRenderProps,
  Tab as AriaTab,
  TabList as AriaTabList,
  type TabListProps as AriaTabListProps,
  TabPanel as AriaTabPanel,
  type TabPanelProps as AriaTabPanelProps,
  type TabProps as AriaTabProps,
  Tabs as AriaTabs,
  type TabsProps as AriaTabsProps,
  TooltipTrigger as AriaTooltipTrigger,
} from "react-aria-components";

import { focusRing } from "./styles";
import { TOOLTIP_DELAY_MS, TooltipBubble } from "./TooltipTrigger";

const TAB_STYLE = [
  "text-ink-muted relative flex h-full cursor-pointer items-center gap-1.5 px-2.5 text-sm whitespace-nowrap select-none",
  "transition-colors duration-150 motion-reduce:transition-none [&_svg]:shrink-0",
  "after:absolute after:inset-x-2.5 after:bottom-0 after:h-0.5 after:bg-transparent",
  "data-hovered:text-ink data-hovered:bg-ink/8 data-selected:text-ink data-selected:after:bg-ink data-selected:cursor-default",
  "data-disabled:text-ink-muted/60 data-disabled:cursor-not-allowed data-disabled:bg-transparent",
  "data-focus-visible:outline-focus outline-hidden -outline-offset-4 data-focus-visible:outline-2",
].join(" ");

export function Tabs({ className, ...props }: TabsProps) {
  return (
    <AriaTabs
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn("flex min-h-0 min-w-0 flex-col data-[orientation=vertical]:flex-row", custom),
      )}
    />
  );
}

export type TabsProps = AriaTabsProps;

export function TabList<T extends object>({ className, ...props }: TabListProps<T>) {
  const ref = useRef<HTMLDivElement>(null);
  const fade = useOverflowFade(ref);

  return (
    <AriaTabList
      {...props}
      ref={ref}
      className={composeRenderProps(className, (custom) =>
        cn(
          "border-rule flex shrink-0 gap-0.5",
          "data-[orientation=horizontal]:border-b data-[orientation=vertical]:flex-col data-[orientation=vertical]:border-r",
          "[scrollbar-width:none] data-[orientation=horizontal]:overflow-x-auto data-[orientation=horizontal]:overflow-y-hidden",
          fade.start &&
            fade.end &&
            "[mask-image:linear-gradient(to_right,transparent,black_24px,black_calc(100%-24px),transparent)]",
          fade.start && !fade.end && "[mask-image:linear-gradient(to_right,transparent,black_24px)]",
          !fade.start && fade.end && "[mask-image:linear-gradient(to_right,black_calc(100%-24px),transparent)]",
          custom,
        ),
      )}
    />
  );
}

export type TabListProps<T extends object> = AriaTabListProps<T>;

export function Tab({ tooltip, className, ...props }: TabProps) {
  const tabClassName = composeRenderProps(className, (custom) => cn(TAB_STYLE, custom));

  if (tooltip === undefined) {
    return <AriaTab {...props} className={tabClassName} />;
  }

  return (
    <AriaTooltipTrigger delay={TOOLTIP_DELAY_MS}>
      <TabWithTooltip {...props} className={tabClassName} tooltip={tooltip} />
    </AriaTooltipTrigger>
  );
}

export interface TabProps extends AriaTabProps {
  tooltip?: ReactNode;
}

export function TabPanel({ className, ...props }: TabPanelProps) {
  return (
    <AriaTabPanel
      {...props}
      className={composeRenderProps(className, (custom) => cn("min-h-0 min-w-0 flex-1", focusRing, custom))}
    />
  );
}

export type TabPanelProps = AriaTabPanelProps;

function TabWithTooltip({ tooltip, ...props }: AriaTabProps & { tooltip: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const { focusableProps } = useFocusable({}, ref);

  return (
    <>
      <AriaTab {...mergeProps(props, focusableProps)} ref={ref} />
      <TooltipBubble placement="bottom">{tooltip}</TooltipBubble>
    </>
  );
}

function useOverflowFade(ref: { current: HTMLElement | null }): { start: boolean; end: boolean } {
  const [fade, setFade] = useState({ start: false, end: false });

  useLayoutEffect(() => {
    const element = ref.current;

    if (element === null) {
      return undefined;
    }

    const measure = () => {
      const start = element.scrollLeft > 1;
      const end = element.scrollLeft + element.clientWidth < element.scrollWidth - 1;

      setFade((current) => (current.start === start && current.end === end ? current : { start, end }));
    };

    const observer = new ResizeObserver(measure);

    observer.observe(element);
    element.addEventListener("scroll", measure, { passive: true });
    measure();

    return () => {
      observer.disconnect();
      element.removeEventListener("scroll", measure);
    };
  }, [ref]);

  return fade;
}
