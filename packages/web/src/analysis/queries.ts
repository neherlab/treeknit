import type {
  AnalysisRequest,
  ArgView,
  DrawingRules,
  Palette,
  Settings,
  TreeInspection,
} from "@neherlab/treeknit-wasm";
import {
  keepPreviousData,
  type QueryKey,
  queryOptions,
  skipToken,
  type SkipToken,
  useQueries,
  useQuery,
  useSuspenseQuery,
  type UseQueryResult,
} from "@tanstack/react-query";
import { useMemo } from "react";
import { isDeepEqual } from "remeda";

import type { AnalysisClient } from "./client";
import { useAnalysisClient } from "./context";
import type { SessionArgs, SessionResult, StatelessArgs, StatelessResult } from "./protocol";

type Answer<Result> = UseQueryResult<Awaited<Result>>;

export interface IdentifiedText {
  textId: number;
  newick: string;
}

const INPUT_QUERY_GC_MS = 5000;

const NO_SESSION = ["session", null] as const;

const UNLABELLED = "";

function pairViewScope(sessionId: number, pair: SessionArgs<"pairView">[0]) {
  return ["session", sessionId, "pairView", pair] as const;
}

function auspiceViewScope(sessionId: number, pair: SessionArgs<"auspiceView">[0]) {
  return ["session", sessionId, "auspiceView", pair] as const;
}

function argViewScope(sessionId: number) {
  return ["session", sessionId, "argView"] as const;
}

export const analysisKeys = {
  defaultSettings: () => ["defaultSettings"] as const,
  palette: () => ["palette"] as const,
  drawingRules: () => ["drawingRules"] as const,
  version: () => ["version"] as const,
  inspectTree: (textId: number) => ["inspectTree", textId] as const,
  overlap: (textIds: readonly number[]) => ["overlap", textIds] as const,
  validate: (textIds: readonly number[], labels: readonly string[], settings: Settings | null | undefined) =>
    ["validate", textIds, labels, settings ?? null] as const,
  settingsSchema: (...args: StatelessArgs<"settingsSchema">) => ["settingsSchema", ...args] as const,
  session: (sessionId: number) => ["session", sessionId] as const,
  files: (sessionId: number) => ["session", sessionId, "files"] as const,
  commandLine: (sessionId: number) => ["session", sessionId, "commandLine"] as const,
  pairViewScope,
  pairView: (sessionId: number, ...[pair, ...rest]: SessionArgs<"pairView">) =>
    [...pairViewScope(sessionId, pair), ...rest] as const,
  auspiceViewScope,
  auspiceView: (sessionId: number, ...[pair, ...rest]: SessionArgs<"auspiceView">) =>
    [...auspiceViewScope(sessionId, pair), ...rest] as const,
  argViewScope,
  argView: (sessionId: number, ...args: SessionArgs<"argView">) => [...argViewScope(sessionId), ...args] as const,
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

export function useSuspensePalette(): Palette {
  const client = useAnalysisClient();

  return useSuspenseQuery({ queryKey: analysisKeys.palette(), queryFn: async () => client.palette() }).data;
}

export function useDrawingRules(): DrawingRules {
  const client = useAnalysisClient();

  return useSuspenseQuery({ queryKey: analysisKeys.drawingRules(), queryFn: async () => client.drawingRules() }).data;
}

export function useVersion(): Answer<StatelessResult<"version">> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.version(), queryFn: async () => client.version() });
}

export function useInspectTrees(trees: readonly IdentifiedText[]): (TreeInspection | undefined)[] {
  const client = useAnalysisClient();

  return useQueries({
    queries: trees.map(({ textId, newick }) => ({
      queryKey: analysisKeys.inspectTree(textId),
      queryFn: async () => client.inspectTree(UNLABELLED, newick),
      gcTime: INPUT_QUERY_GC_MS,
    })),
    combine: inspectionData,
  });
}

export function useOverlap(trees: readonly IdentifiedText[]): Answer<StatelessResult<"overlap">> {
  const client = useAnalysisClient();
  const textIds = useMemo(() => trees.map(({ textId }) => textId), [trees]);

  return useQuery({
    queryKey: analysisKeys.overlap(textIds),
    queryFn: async () => client.overlap(trees.map(({ newick }) => ({ label: UNLABELLED, newick }))),
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function validationQuery(client: AnalysisClient, request: AnalysisRequest, textIds: readonly number[]) {
  return queryOptions({
    queryKey: analysisKeys.validate(
      textIds,
      request.trees.map(({ label }) => label),
      request.settings,
    ),
    queryFn: async () => client.validate(request),
    gcTime: INPUT_QUERY_GC_MS,
  });
}

export function useValidation(
  request: AnalysisRequest,
  textIds: readonly number[],
): Answer<StatelessResult<"validate">> {
  return useQuery({
    ...validationQuery(useAnalysisClient(), request, textIds),
    placeholderData: (previous, previousQuery) => (sameTexts(previousQuery?.queryKey, textIds) ? previous : undefined),
  });
}

export function sameTexts(
  previous: ReturnType<typeof analysisKeys.validate> | undefined,
  textIds: readonly number[],
): boolean {
  return previous !== undefined && isDeepEqual(previous[1], textIds);
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
      sharesScope(
        previousQuery?.queryKey,
        sessionKey(sessionId, (id) => analysisKeys.pairViewScope(id, args[0])),
      )
        ? previous
        : undefined,
  });
}

export function useAuspiceView(
  sessionId: number | null,
  ...args: SessionArgs<"auspiceView">
): Answer<SessionResult<"auspiceView">> {
  const client = useAnalysisClient();
  const queryKey = sessionKey(sessionId, (id) => analysisKeys.auspiceView(id, ...args));

  return useQuery({
    queryKey,
    queryFn: sessionQuery(sessionId, async (id) => client.auspiceView(id, ...args)),
    placeholderData: (previous, previousQuery) =>
      sharesScope(
        previousQuery?.queryKey,
        sessionKey(sessionId, (id) => analysisKeys.auspiceViewScope(id, args[0])),
      )
        ? previous
        : undefined,
  });
}

export function useArgView(sessionId: number | null, ...args: SessionArgs<"argView">): UseQueryResult<ArgView | null> {
  const client = useAnalysisClient();
  const queryKey = sessionKey(sessionId, (id) => analysisKeys.argView(id, ...args));

  return useQuery({
    queryKey,
    queryFn: sessionQuery(sessionId, async (id) => (await client.argView(id, ...args)) ?? null),
    placeholderData: (previous, previousQuery) =>
      sharesScope(previousQuery?.queryKey, sessionKey(sessionId, analysisKeys.argViewScope)) ? previous : undefined,
  });
}

export function useConstellation(sessionId: number | null): Answer<SessionResult<"constellation">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: sessionKey(sessionId, analysisKeys.constellation),
    queryFn: sessionQuery(sessionId, async (id) => client.constellation(id)),
  });
}

export function currentData<T>({
  data,
  isPlaceholderData,
}: Pick<UseQueryResult<T>, "data" | "isPlaceholderData">): T | undefined {
  return isPlaceholderData ? undefined : data;
}

export function sharesScope(previous: QueryKey | undefined, scope: QueryKey): boolean {
  return previous !== undefined && isDeepEqual(previous.slice(0, scope.length), scope);
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
