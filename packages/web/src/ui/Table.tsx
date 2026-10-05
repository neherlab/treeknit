import { cn } from "cn";
import {
  Cell as AriaCell,
  type CellProps as AriaCellProps,
  Column as AriaColumn,
  type ColumnProps as AriaColumnProps,
  composeRenderProps,
  Row as AriaRow,
  type RowProps as AriaRowProps,
  Table as AriaTable,
  TableBody as AriaTableBody,
  type TableBodyProps as AriaTableBodyProps,
  TableHeader as AriaTableHeader,
  type TableHeaderProps as AriaTableHeaderProps,
  type TableProps as AriaTableProps,
} from "react-aria-components";
import ArrowDownIcon from "~icons/lucide/arrow-down";
import ArrowUpIcon from "~icons/lucide/arrow-up";

const cellFocus = "-outline-offset-2 outline-hidden data-focus-visible:outline-2 data-focus-visible:outline-focus";

export const tableStyle = "text-ink font-condensed w-full border-separate border-spacing-0 text-sm";

export const tableHeaderStyle = "bg-ground sticky top-0 z-10";

export const rowHoverStyle = "data-hovered:bg-ink/4 data-selected:bg-pane";

export const nativeRowStyle = "group/row hover:bg-ink/4 data-selected:bg-pane";

export function columnStyle(align: CellAlign): string {
  return cn(
    "border-rule text-ink-muted h-8 border-b px-2 text-xs font-normal whitespace-nowrap",
    align === "end" ? "text-right" : "text-left",
  );
}

export function cellStyle(align: CellAlign): string {
  return cn(
    "border-rule h-8 border-b px-2 align-middle group-last/row:border-b-0",
    align === "end" ? "text-right" : "text-left",
  );
}

export type CellAlign = "start" | "end";

export function Table({ className, ...props }: TableProps) {
  return (
    <AriaTable {...props} className={composeRenderProps(className, (custom) => cn(tableStyle, cellFocus, custom))} />
  );
}

export type TableProps = AriaTableProps;

export function TableHeader<T extends object>({ className, ...props }: TableHeaderProps<T>) {
  return (
    <AriaTableHeader {...props} className={composeRenderProps(className, (custom) => cn(tableHeaderStyle, custom))} />
  );
}

export type TableHeaderProps<T extends object> = AriaTableHeaderProps<T>;

export function Column({ align = "start", className, children, ...props }: ColumnProps) {
  return (
    <AriaColumn
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn(columnStyle(align), "data-hovered:text-ink data-allows-sorting:cursor-default", cellFocus, custom),
      )}
    >
      {composeRenderProps(children, (content, { allowsSorting, sortDirection }) => (
        <span className={cn("inline-flex items-center gap-1", align === "end" && "flex-row-reverse")}>
          {content}
          {allowsSorting ? <SortIndicator direction={sortDirection} /> : null}
        </span>
      ))}
    </AriaColumn>
  );
}

export interface ColumnProps extends AriaColumnProps {
  align?: CellAlign;
}

export function TableBody<T extends object>({ className, ...props }: TableBodyProps<T>) {
  return (
    <AriaTableBody
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn("data-empty:text-ink-muted data-empty:h-16 data-empty:text-center", custom),
      )}
    />
  );
}

export type TableBodyProps<T extends object> = AriaTableBodyProps<T>;

export function Row<T extends object>({ className, ...props }: RowProps<T>) {
  return (
    <AriaRow
      {...props}
      className={composeRenderProps(className, (custom) =>
        cn("group/row cursor-default data-href:cursor-pointer", rowHoverStyle, cellFocus, custom),
      )}
    />
  );
}

export type RowProps<T extends object> = AriaRowProps<T>;

export function Cell({ align = "start", className, ...props }: CellProps) {
  return (
    <AriaCell
      {...props}
      className={composeRenderProps(className, (custom) => cn(cellStyle(align), cellFocus, custom))}
    />
  );
}

export interface CellProps extends AriaCellProps {
  align?: CellAlign;
}

export function SortIndicator({ direction }: { direction: "ascending" | "descending" | undefined }) {
  if (direction === undefined) {
    return <ArrowDownIcon aria-hidden className="invisible size-3.5" />;
  }

  const Icon = direction === "ascending" ? ArrowUpIcon : ArrowDownIcon;

  return <Icon aria-hidden className="text-ink size-3.5" />;
}
