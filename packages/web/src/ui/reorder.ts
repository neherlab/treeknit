import type { Key } from "react-aria-components";

export interface ReorderMove {
  keys: ReadonlySet<Key>;
  target: Key;
  position: "before" | "after";
}

export function reorder<T>(items: readonly T[], keyOf: (item: T) => Key, move: ReorderMove): T[] {
  if (move.keys.has(move.target)) {
    return [...items];
  }

  const moved = items.filter((item) => move.keys.has(keyOf(item)));
  const rest = items.filter((item) => !move.keys.has(keyOf(item)));
  const targetIndex = rest.findIndex((item) => keyOf(item) === move.target);

  if (targetIndex === -1 || moved.length === 0) {
    return [...items];
  }

  const insertAt = move.position === "before" ? targetIndex : targetIndex + 1;

  return [...rest.slice(0, insertAt), ...moved, ...rest.slice(insertAt)];
}
