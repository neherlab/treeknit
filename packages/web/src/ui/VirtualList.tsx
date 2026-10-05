import { cn } from "cn";
import type { ReactNode } from "react";
import { type Key, ListBox, ListBoxItem, ListLayout, type ListLayoutOptions, Virtualizer } from "react-aria-components";

import { optionStyle } from "./styles";

const ROW_PX = 32;

const PADDING_PX = 4;

const LAYOUT_OPTIONS: ListLayoutOptions = { rowSize: ROW_PX, padding: PADDING_PX };

export function VirtualList<T extends { id: Key; text: string }>({
  label,
  items,
  onAction,
  children,
  className,
}: VirtualListProps<T>) {
  return (
    <Virtualizer layout={ListLayout} layoutOptions={LAYOUT_OPTIONS}>
      <ListBox
        aria-label={label}
        items={items}
        onAction={onAction}
        className={cn("border-rule bg-ground rounded-control overflow-auto border outline-hidden", className)}
      >
        {(item) => (
          <ListBoxItem id={item.id} textValue={item.text} className={cn(optionStyle, "min-w-0 overflow-hidden")}>
            {children(item)}
          </ListBoxItem>
        )}
      </ListBox>
    </Virtualizer>
  );
}

export interface VirtualListProps<T> {
  label: string;
  items: readonly T[];
  onAction: (key: Key) => void;
  children: (item: T) => ReactNode;
  className?: string;
}
