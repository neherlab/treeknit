export const LEAF_SEARCH_LIMIT = 50;

export interface LeafMatches {
  shown: string[];
  total: number;
}

export function matchingLeaves(
  names: readonly string[],
  query: string,
  contains: (text: string, substring: string) => boolean,
  limit = LEAF_SEARCH_LIMIT,
): LeafMatches {
  const trimmed = query.trim();

  if (trimmed === "") {
    return { shown: [], total: 0 };
  }

  const matches = names.filter((name) => contains(name, trimmed));

  return { shown: matches.slice(0, limit), total: matches.length };
}
