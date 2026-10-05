import { cn } from "cn";
import {
  Button as AriaButton,
  composeRenderProps,
  type Key,
  DropIndicator,
  GridList as AriaGridList,
  GridListItem as AriaGridListItem,
  type GridListItemProps as AriaGridListItemProps,
  type GridListProps as AriaGridListProps,
  useDragAndDrop,
} from "react-aria-components";
import GripIcon from "~icons/lucide/grip-vertical";

import type { ReorderMove } from "./reorder";
import { focusRing } from "./styles";
import { TooltipTrigger } from "./TooltipTrigger";

export function GridList<T extends object>({ onReorder, dragLabel, className, ...props }: GridListProps<T>) {
  const { dragAndDropHooks } = useDragAndDrop<T>({
    isDisabled: onReorder === undefined,
    getItems: (keys) => [...keys].map((key) => ({ "text/plain": dragLabel?.(key) ?? String(key) })),
    onReorder(event) {
      if (event.target.dropPosition !== "on") {
        onReorder?.({ keys: event.keys, target: event.target.key, position: event.target.dropPosition });
      }
    },
    renderDropIndicator: (target) => (
      <DropIndicator
        target={target}
        className="data-drop-target:bg-focus -my-px h-0.5 rounded-full bg-transparent outline-hidden"
      />
    ),
  });

  return (
    <AriaGridList
      {...props}
      {...(onReorder === undefined ? {} : { dragAndDropHooks })}
      className={composeRenderProps(className, (custom) =>
        cn(
          "border-rule flex flex-col border-y outline-hidden",
          "data-focus-visible:outline-focus data-focus-visible:outline-2",
          custom,
        ),
      )}
    />
  );
}

export interface GridListProps<T extends object> extends Omit<AriaGridListProps<T>, "dragAndDropHooks"> {
  onReorder?: (move: ReorderMove) => void;
  dragLabel?: (key: Key) => string;
}

export function GridListItem({ className, children, ...props }: GridListItemProps) {
  return (
    <AriaGridListItem
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn(
          "border-rule text-ink relative flex cursor-default items-start gap-2 border-b px-2 py-2 text-sm select-none last:border-b-0",
          "data-hovered:bg-ink/4 data-selected:bg-pane data-dragging:opacity-50",
          "data-focus-visible:outline-focus outline-hidden -outline-offset-2 data-focus-visible:outline-2",
          custom,
        ),
      )}
    >
      {composeRenderProps(children, (content, { allowsDragging }) => (
        <>
          {allowsDragging === true ? (
            <TooltipTrigger tooltip="Drag to reorder">
              <AriaButton
                slot="drag"
                className={cn(
                  "rounded-control text-ink-muted flex h-8 w-5 shrink-0 cursor-grab items-center justify-center",
                  "data-hovered:text-ink data-pressed:cursor-grabbing",
                  focusRing,
                )}
              >
                <GripIcon aria-hidden />
              </AriaButton>
            </TooltipTrigger>
          ) : null}
          <div className="flex min-w-0 flex-1 flex-col gap-1">{content}</div>
        </>
      ))}
    </AriaGridListItem>
  );
}

export type GridListItemProps = AriaGridListItemProps;
