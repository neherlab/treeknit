export function itemAt<T>(items: readonly T[], index: number, kind: string): T {
  const item = items[index];

  if (item === undefined) {
    throw new RangeError(
      `The drawing data refers to ${kind} ${String(index)}, but it has ${String(items.length)} of them`,
    );
  }

  return item;
}
