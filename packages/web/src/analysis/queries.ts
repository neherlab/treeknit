import type { ArgView, TreeInspection, TreeText } from "@neherlab/treeknit-wasm";
import {
  keepPreviousData,
  type QueryKey,
  skipToken,
  type SkipToken,
  useQueries,
  useQuery,
  type UseQueryResult,
} from "@tanstack/react-query";
import { isDeepEqual } from "remeda";

import { useAnalysisClient } from "./context";
import type { SessionArgs, SessionResult, StatelessArgs, StatelessResult } from "./protocol";

type Answer<Result> = UseQueryResult<Awaited<Result>>;

const INPUT_QUERY_GC_MS = 5000;

const PAIR_SCOPE = 4;

const SESSION_SCOPE = 3;

const NO_SESSION = ["session", null] as const;

export const analysisKeys = {
  defaultSettings: () => ["defaultSettings"] as const,
  palette: () => ["palette"] as const,
  version: () => ["version"] as const,
  inspectTree: (...args: StatelessArgs<"inspectTree">) => ["inspectTree", ...args] as const,
  overlap: (trees: readonly TreeText[]) => ["overlap", trees] as const,
  validate: (...args: StatelessArgs<"validate">) => ["validate", ...args] as const,
  settingsSchema: (...args: StatelessArgs<"settingsSchema">) => ["settingsSchema", ...args] as const,
  session: (sessionId: number) => ["session", sessionId] as const,
  files: (sessionId: number) => ["session", sessionId, "files"] as const,
  commandLine: (sessionId: number) => ["session", sessionId, "commandLine"] as const,
  pairView: (sessionId: number, ...args: SessionArgs<"pairView">) =>
    ["session", sessionId, "pairView", ...args] as const,
  argView: (sessionId: number, ...args: SessionArgs<"argView">) => ["session", sessionId, "argView", ...args] as const,
  constellation: (sessionId: number) => ["session", sessionId, "constellation"] as const,
};

export function useDefaultSettings(): Answer<StatelessResult<"defaultSettings">> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.defaultSettings(), queryFn: async () => client.defaultSettings() });
}

export function usePalette(): Answer<StatelessResult<"palette">> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.palette(), queryFn: async () => client.palette() });
}

export function useVersion(): Answer<StatelessResult<"version">> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.version(), queryFn: async () => client.version() });
}

export function useInspectTree(...args: StatelessArgs<"inspectTree">): Answer<StatelessResult<"inspectTree">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.inspectTree(...args),
    queryFn: async () => client.inspectTree(...args),
    placeholderData: keepPreviousData,
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function useInspectTrees(trees: readonly TreeText[]): (TreeInspection | undefined)[] {
  const client = useAnalysisClient();

  return useQueries({
    queries: trees.map(({ label, newick }) => ({
      queryKey: analysisKeys.inspectTree(label, newick),
      queryFn: async () => client.inspectTree(label, newick),
      gcTime: INPUT_QUERY_GC_MS,
    })),
    combine: inspectionData,
  });
}

export function useOverlap(trees: readonly TreeText[]): Answer<StatelessResult<"overlap">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.overlap(trees),
    queryFn: async () => client.overlap([...trees]),
    placeholderData: keepPreviousData,
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function useValidation(...args: StatelessArgs<"validate">): Answer<StatelessResult<"validate">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.validate(...args),
    queryFn: async () => client.validate(...args),
    placeholderData: keepPreviousData,
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function useSettingsSchema(...args: StatelessArgs<"settingsSchema">): Answer<StatelessResult<"settingsSchema">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.settingsSchema(...args),
    queryFn: async () => client.settingsSchema(...args),
    placeholderData: keepPreviousData,
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function useSessionFiles(sessionId: number | null): Answer<SessionResult<"files">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: sessionKey(sessionId, analysisKeys.files),
    queryFn: sessionQuery(sessionId, async (id) => client.files(id)),
  });
}

export function useCommandLine(sessionId: number | null): Answer<SessionResult<"commandLine">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: sessionKey(sessionId, analysisKeys.commandLine),
    queryFn: sessionQuery(sessionId, async (id) => client.commandLine(id)),
  });
}

export function usePairView(
  sessionId: number | null,
  ...args: SessionArgs<"pairView">
): Answer<SessionResult<"pairView">> {
  const client = useAnalysisClient();
  const queryKey = sessionKey(sessionId, (id) => analysisKeys.pairView(id, ...args));

  return useQuery({
    queryKey,
    queryFn: sessionQuery(sessionId, async (id) => client.pairView(id, ...args)),
    placeholderData: (previous, previousQuery) =>
      sharesScope(previousQuery?.queryKey, queryKey, PAIR_SCOPE) ? previous : undefined,
  });
}

export function useArgView(sessionId: number | null, ...args: SessionArgs<"argView">): UseQueryResult<ArgView | null> {
  const client = useAnalysisClient();
  const queryKey = sessionKey(sessionId, (id) => analysisKeys.argView(id, ...args));

  return useQuery({
    queryKey,
    queryFn: sessionQuery(sessionId, async (id) => (await client.argView(id, ...args)) ?? null),
    placeholderData: (previous, previousQuery) =>
      sharesScope(previousQuery?.queryKey, queryKey, SESSION_SCOPE) ? previous : undefined,
  });
}

export function useConstellation(sessionId: number | null): Answer<SessionResult<"constellation">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: sessionKey(sessionId, analysisKeys.constellation),
    queryFn: sessionQuery(sessionId, async (id) => client.constellation(id)),
  });
}

export function sharesScope(previous: QueryKey | undefined, next: QueryKey, depth: number): boolean {
  return previous !== undefined && isDeepEqual(previous.slice(0, depth), next.slice(0, depth));
}

function sessionKey<Key extends QueryKey>(
  sessionId: number | null,
  key: (sessionId: number) => Key,
): Key | typeof NO_SESSION {
  return sessionId === null ? NO_SESSION : key(sessionId);
}

function sessionQuery<T>(
  sessionId: number | null,
  load: (sessionId: number) => Promise<T>,
): (() => Promise<T>) | SkipToken {
  return sessionId === null ? skipToken : async () => load(sessionId);
}

function inspectionData(results: readonly UseQueryResult<TreeInspection>[]): (TreeInspection | undefined)[] {
  return results.map(({ data }) => data);
}
