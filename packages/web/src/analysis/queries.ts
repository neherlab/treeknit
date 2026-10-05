import type { ArgView, TreeInspection, TreeText } from "@neherlab/treeknit-wasm";
import { keepPreviousData, useQueries, useQuery, type UseQueryResult } from "@tanstack/react-query";

import { useAnalysisClient } from "./context";
import type { SessionArgs, SessionResult, StatelessArgs, StatelessResult } from "./protocol";

type Answer<Result> = UseQueryResult<Awaited<Result>>;

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
  });
}

export function useInspectTrees(trees: readonly TreeText[]): (TreeInspection | undefined)[] {
  const client = useAnalysisClient();

  return useQueries({
    queries: trees.map(({ label, newick }) => ({
      queryKey: analysisKeys.inspectTree(label, newick),
      queryFn: async () => client.inspectTree(label, newick),
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
  });
}

export function useValidation(...args: StatelessArgs<"validate">): Answer<StatelessResult<"validate">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.validate(...args),
    queryFn: async () => client.validate(...args),
    placeholderData: keepPreviousData,
  });
}

export function useSettingsSchema(...args: StatelessArgs<"settingsSchema">): Answer<StatelessResult<"settingsSchema">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.settingsSchema(...args),
    queryFn: async () => client.settingsSchema(...args),
    placeholderData: keepPreviousData,
  });
}

export function useSessionFiles(sessionId: number): Answer<SessionResult<"files">> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.files(sessionId), queryFn: async () => client.files(sessionId) });
}

export function useCommandLine(sessionId: number): Answer<SessionResult<"commandLine">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.commandLine(sessionId),
    queryFn: async () => client.commandLine(sessionId),
  });
}

export function usePairView(sessionId: number, ...args: SessionArgs<"pairView">): Answer<SessionResult<"pairView">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.pairView(sessionId, ...args),
    queryFn: async () => client.pairView(sessionId, ...args),
  });
}

export function useArgView(sessionId: number, ...args: SessionArgs<"argView">): UseQueryResult<ArgView | null> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.argView(sessionId, ...args),
    queryFn: async () => (await client.argView(sessionId, ...args)) ?? null,
  });
}

export function useConstellation(sessionId: number): Answer<SessionResult<"constellation">> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.constellation(sessionId),
    queryFn: async () => client.constellation(sessionId),
  });
}

function inspectionData(results: readonly UseQueryResult<TreeInspection>[]): (TreeInspection | undefined)[] {
  return results.map(({ data }) => data);
}
