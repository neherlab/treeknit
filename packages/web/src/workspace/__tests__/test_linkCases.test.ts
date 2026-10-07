import { describe, expect, test } from "vitest";

import { linkEntries } from "../../launch/entries";
import { type QueryEntry, readQuery, writeQuery } from "../searchQuery";
import table from "./__fixtures__/link_cases.json";

describe("the link grammar of the web app", () => {
  test.each(table.written)(
    "writes $query as the Rust reference does, and reads it back",
    ({ pairs, query, readBack }) => {
      expect({ written: writeQuery(pairs.map(entry)), read: readQuery(query).map(pair) }).toStrictEqual({
        written: query,
        read: readBack,
      });
    },
  );

  test.each(table.read)("reads the query '$query' as the Rust reference does", ({ query, pairs }) => {
    expect(readQuery(query).map(pair)).toStrictEqual(pairs);
  });

  test.each(table.links)("reads the link $url as the Rust reference does", ({ url, pairs }) => {
    expect(linkEntries(addressParts(url)).map(pair)).toStrictEqual(pairs);
  });
});

function entry([key = "", value = ""]: readonly string[]): QueryEntry {
  return { key, value: value === "" ? true : value };
}

function pair({ key, value }: QueryEntry): string[] {
  return [key, value === true ? "" : value];
}

function addressParts(url: string) {
  const [beforeHash = "", ...afterHash] = url.split("#");
  const [, ...afterQuestion] = beforeHash.split("?");
  const query = afterQuestion.join("?");
  const fragment = afterHash.join("#");

  return { search: query === "" ? "" : `?${query}`, hash: fragment === "" ? "" : `#${fragment}` };
}
