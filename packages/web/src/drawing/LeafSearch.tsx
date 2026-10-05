import { cn } from "cn";
import { useCallback, useMemo, useRef, useState } from "react";
import { Autocomplete, Button, Input, type Key, Menu, Popover, SearchField, useFilter } from "react-aria-components";
import SearchIcon from "~icons/lucide/search";
import ClearIcon from "~icons/lucide/x";

import { MenuItem } from "../ui/Menu";
import { focusRing, inputStyle, listBoxStyle, popoverStyle } from "../ui/styles";
import { counted, formatCount } from "./format";
import { matchingLeaves } from "./leafSearch";

const POPOVER_OFFSET_PX = 4;

const SEARCH_LABEL = "Find a leaf";

export function LeafSearch({ names, onSelect, className }: LeafSearchProps) {
  const { contains } = useFilter({ sensitivity: "base" });
  const [query, setQuery] = useState("");
  const fieldRef = useRef<HTMLDivElement>(null);
  const matches = useMemo(() => matchingLeaves(names, query, contains), [names, query, contains]);
  const items = useMemo(() => matches.shown.map((name) => ({ id: name })), [matches]);
  const open = query.trim() !== "";

  const choose = useCallback(
    (key: Key) => {
      onSelect(String(key));
      setQuery("");
    },
    [onSelect],
  );

  const close = useCallback((isOpen: boolean) => {
    if (!isOpen) {
      setQuery("");
    }
  }, []);

  const emptyState = useCallback(() => <p className="text-ink-muted px-2 py-1.5 text-sm">No leaf matches.</p>, []);

  return (
    <Autocomplete inputValue={query} onInputChange={setQuery}>
      <SearchField
        ref={fieldRef}
        aria-label={SEARCH_LABEL}
        className={cn("group/search relative flex w-52 items-center", className)}
      >
        <SearchIcon aria-hidden className="text-ink-muted pointer-events-none absolute left-2 size-4" />
        <Input
          placeholder={SEARCH_LABEL}
          className={cn(inputStyle(), "pr-7 pl-7 [&::-webkit-search-cancel-button]:hidden")}
        />
        <Button
          aria-label="Clear the search"
          className={cn(
            "rounded-inner text-ink-muted data-hovered:text-ink absolute right-1.5 flex size-5 items-center justify-center group-data-empty/search:hidden",
            focusRing,
          )}
        >
          <ClearIcon aria-hidden className="size-3.5" />
        </Button>
      </SearchField>
      <Popover
        triggerRef={fieldRef}
        isOpen={open}
        onOpenChange={close}
        isNonModal
        placement="bottom start"
        offset={POPOVER_OFFSET_PX}
        className={cn(popoverStyle, "w-72")}
      >
        <Menu
          aria-label="Matching leaves"
          items={items}
          onAction={choose}
          renderEmptyState={emptyState}
          className={listBoxStyle}
        >
          {(item) => (
            <MenuItem id={item.id} textValue={item.id} className="font-condensed">
              {item.id}
            </MenuItem>
          )}
        </Menu>
        {matches.total > matches.shown.length ? (
          <p className="border-rule text-ink-muted border-t px-3 py-1.5 text-xs">
            {`Showing ${formatCount(matches.shown.length)} of ${counted(matches.total, "match", "matches")}`}
          </p>
        ) : null}
      </Popover>
    </Autocomplete>
  );
}

export interface LeafSearchProps {
  names: readonly string[];
  onSelect: (name: string) => void;
  className?: string;
}
