import { cn } from "cn";
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
} from "react-aria-components";

import { focusRing } from "./styles";

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
  return (
    <AriaTabList
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn(
          "border-rule flex shrink-0 gap-1",
          "data-[orientation=horizontal]:border-b data-[orientation=vertical]:flex-col data-[orientation=vertical]:border-r",
          "[scrollbar-width:thin] data-[orientation=horizontal]:overflow-x-auto data-[orientation=horizontal]:overflow-y-hidden",
          custom,
        ),
      )}
    />
  );
}

export type TabListProps<T extends object> = AriaTabListProps<T>;

export function Tab({ className, ...props }: TabProps) {
  return (
    <AriaTab
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn(
          "text-ink-muted relative flex h-10 cursor-default items-center gap-1.5 px-2.5 text-sm whitespace-nowrap select-none",
          "transition-colors duration-150 motion-reduce:transition-none [&_svg]:shrink-0",
          "after:absolute after:inset-x-2.5 after:bottom-0 after:h-0.5 after:bg-transparent",
          "data-hovered:text-ink data-selected:text-ink data-selected:after:bg-ink",
          "data-disabled:text-ink-muted/60 data-disabled:cursor-not-allowed",
          "data-focus-visible:outline-focus outline-hidden -outline-offset-2 data-focus-visible:outline-2",
          custom,
        ),
      )}
    />
  );
}

export type TabProps = AriaTabProps;

export function TabPanel({ className, ...props }: TabPanelProps) {
  return (
    <AriaTabPanel
      {...props}
      className={composeRenderProps(className, (custom) => cn("min-h-0 min-w-0 flex-1", focusRing, custom))}
    />
  );
}

export type TabPanelProps = AriaTabPanelProps;
