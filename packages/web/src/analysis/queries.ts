import type {
  AnalysisRequest,
  AppVersion,
  ArgView,
  ConstellationTable,
  FileEntry,
  Overlap,
  PairView,
  Palette,
  Scale,
  Settings,
  SettingsSchema,
  TreeInspection,
  TreeText,
  ValidationError,
  TreeVersion,
} from "@neherlab/treeknit-wasm";
import { keepPreviousData, useQuery, type UseQueryResult } from "@tanstack/react-query";

import { useAnalysisClient } from "./context";

export const analysisKeys = {
  defaultSettings: () => ["defaultSettings"] as const,
  palette: () => ["palette"] as const,
  version: () => ["version"] as const,
  inspectTree: (label: string, text: string) => ["inspectTree", label, text] as const,
  overlap: (trees: readonly TreeText[]) => ["overlap", trees] as const,
  validate: (request: AnalysisRequest) => ["validate", request] as const,
  settingsSchema: (k: number, settings: Settings) => ["settingsSchema", k, settings] as const,
  session: (sessionId: number) => ["session", sessionId] as const,
  files: (sessionId: number) => ["session", sessionId, "files"] as const,
  commandLine: (sessionId: number) => ["session", sessionId, "commandLine"] as const,
  pairView: (sessionId: number, pair: number, version: TreeVersion, scale: Scale) =>
    ["session", sessionId, "pairView", pair, version, scale] as const,
  argView: (sessionId: number, scale: Scale) => ["session", sessionId, "argView", scale] as const,
  constellation: (sessionId: number) => ["session", sessionId, "constellation"] as const,
};

export function useDefaultSettings(): UseQueryResult<Settings> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.defaultSettings(), queryFn: async () => client.defaultSettings() });
}

export function usePalette(): UseQueryResult<Palette> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.palette(), queryFn: async () => client.palette() });
}

export function useVersion(): UseQueryResult<AppVersion> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.version(), queryFn: async () => client.version() });
}

export function useInspectTree(label: string, text: string): UseQueryResult<TreeInspection> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.inspectTree(label, text),
    queryFn: async () => client.inspectTree(label, text),
    placeholderData: keepPreviousData,
  });
}

export function useOverlap(trees: readonly TreeText[]): UseQueryResult<Overlap> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.overlap(trees),
    queryFn: async () => client.overlap([...trees]),
    placeholderData: keepPreviousData,
  });
}

export function useValidation(request: AnalysisRequest): UseQueryResult<ValidationError[]> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.validate(request),
    queryFn: async () => client.validate(request),
    placeholderData: keepPreviousData,
  });
}

export function useSettingsSchema(k: number, settings: Settings): UseQueryResult<SettingsSchema> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.settingsSchema(k, settings),
    queryFn: async () => client.settingsSchema(k, settings),
    placeholderData: keepPreviousData,
  });
}

export function useSessionFiles(sessionId: number): UseQueryResult<FileEntry[]> {
  const client = useAnalysisClient();

  return useQuery({ queryKey: analysisKeys.files(sessionId), queryFn: async () => client.files(sessionId) });
}

export function useCommandLine(sessionId: number): UseQueryResult<string> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.commandLine(sessionId),
    queryFn: async () => client.commandLine(sessionId),
  });
}

export function usePairView(
  sessionId: number,
  pair: number,
  version: TreeVersion,
  scale: Scale,
): UseQueryResult<PairView> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.pairView(sessionId, pair, version, scale),
    queryFn: async () => client.pairView(sessionId, pair, version, scale),
  });
}

export function useArgView(sessionId: number, scale: Scale): UseQueryResult<ArgView | null> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.argView(sessionId, scale),
    queryFn: async () => (await client.argView(sessionId, scale)) ?? null,
  });
}

export function useConstellation(sessionId: number): UseQueryResult<ConstellationTable> {
  const client = useAnalysisClient();

  return useQuery({
    queryKey: analysisKeys.constellation(sessionId),
    queryFn: async () => client.constellation(sessionId),
  });
}
