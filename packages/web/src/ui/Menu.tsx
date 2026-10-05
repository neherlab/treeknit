import { cn } from "cn";
import type { ComponentType, ReactNode, SVGProps } from "react";
import {
  composeRenderProps,
  Header,
  Menu as AriaMenu,
  MenuItem as AriaMenuItem,
  type MenuItemProps as AriaMenuItemProps,
  type MenuProps as AriaMenuProps,
  MenuSection as AriaMenuSection,
  type MenuSectionProps as AriaMenuSectionProps,
  MenuTrigger,
  type MenuTriggerProps as AriaMenuTriggerProps,
  Popover,
  type PopoverProps,
  Separator,
} from "react-aria-components";
import CheckIcon from "~icons/lucide/check";

import { listBoxStyle, popoverStyle } from "./styles";

const POPOVER_OFFSET_PX = 4;

export function Menu<T extends object>({ trigger, placement = "bottom start", className, ...props }: MenuProps<T>) {
  return (
    <MenuTrigger>
      {trigger}
      <Popover placement={placement} offset={POPOVER_OFFSET_PX} className={cn(popoverStyle, "min-w-48")}>
        <AriaMenu {...props} className={composeRenderProps(className, (custom) => cn(listBoxStyle, custom))} />
      </Popover>
    </MenuTrigger>
  );
}

export interface MenuProps<T extends object> extends AriaMenuProps<T> {
  trigger: AriaMenuTriggerProps["children"];
  placement?: PopoverProps["placement"];
}

export function MenuItem({ icon: Icon, tone = "default", className, children, ...props }: MenuItemProps) {
  return (
    <AriaMenuItem
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn(
          "rounded-control flex cursor-pointer items-center gap-2 px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:shrink-0",
          tone === "danger" ? "text-danger" : "text-ink",
          "data-focused:bg-pane data-pressed:bg-pane data-open:bg-pane data-disabled:text-ink-muted data-disabled:cursor-not-allowed",
          custom,
        ),
      )}
    >
      {composeRenderProps(children, (content, { selectionMode, isSelected }) => (
        <>
          {selectionMode === "none" ? null : <CheckIcon aria-hidden className={cn(!isSelected && "invisible")} />}
          {Icon === undefined ? null : <Icon aria-hidden className={tone === "danger" ? "" : "text-ink-muted"} />}
          <span className="flex-1 truncate">{content}</span>
        </>
      ))}
    </AriaMenuItem>
  );
}

export interface MenuItemProps extends AriaMenuItemProps {
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  tone?: "default" | "danger";
}

export function MenuSection<T extends object>({ title, className, children, ...props }: MenuSectionProps<T>) {
  return (
    <AriaMenuSection
      {...props}
      className={cn("not-first:border-rule not-first:mt-1 not-first:border-t not-first:pt-1", className)}
    >
      {title === undefined ? null : <Header className="text-ink-muted px-2 py-1 text-xs">{title}</Header>}
      {children}
    </AriaMenuSection>
  );
}

export interface MenuSectionProps<T extends object> extends Omit<AriaMenuSectionProps<T>, "children" | "items"> {
  title?: string;
  children: ReactNode;
}

export function MenuSeparator() {
  return <Separator className="bg-rule mx-1 my-1 h-px" />;
}
