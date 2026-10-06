import * as z from "zod";

export type QueryValue = string | true;

export type QueryRecord = Record<string, QueryValue | QueryValue[]>;

const queryItemSchema = z.union([z.string(), z.literal(true)]);

export const queryValueSchema = z.union([queryItemSchema, z.array(queryItemSchema)]);

export interface QueryEntry {
  key: string;
  value: QueryValue;
}

const ESCAPED_IN_VALUES = new Set(["%", "&", "#", "+", " ", '"', "<", ">"]);

export function readQuery(query: string): QueryEntry[] {
  return query
    .split("&")
    .filter((part) => part !== "")
    .map((part) => {
      const [[key, value] = ["", ""]] = new URLSearchParams(part);

      return { key, value: part.includes("=") ? value : true };
    });
}

export function writeQuery(entries: readonly QueryEntry[]): string {
  return entries
    .map(({ key, value }) =>
      value === true ? escapeComponent(key, true) : `${escapeComponent(key, true)}=${escapeComponent(value, false)}`,
    )
    .join("&");
}

export function queryRecord(entries: readonly QueryEntry[]) {
  return Object.fromEntries(
    Array.from(
      Map.groupBy(entries, ({ key }) => key),
      ([key, group]) => {
        const values = group.map(({ value }) => value);
        const [only, ...rest] = values;

        return [key, only !== undefined && rest.length === 0 ? only : values];
      },
    ),
  );
}

export function recordEntries(record: Readonly<QueryRecord>): QueryEntry[] {
  return Object.entries(record).flatMap(([key, value]) =>
    (Array.isArray(value) ? value : [value]).map((item: QueryValue) => ({ key, value: item })),
  );
}

function escapeComponent(text: string, key: boolean): string {
  return Array.from(text, (char) => (mustEscape(char, key) ? encodeURIComponent(char) : char)).join("");
}

function mustEscape(char: string, key: boolean): boolean {
  const code = char.codePointAt(0) ?? 0;

  return ESCAPED_IN_VALUES.has(char) || (key && char === "=") || code < 0x20 || code >= 0x7f;
}
