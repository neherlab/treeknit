import type {
  AnalysisRequest,
  AppVersion,
  ArgView,
  ConstellationTable,
  FigureOptions,
  FileEntry,
  OutputFile,
  Overlap,
  PairView,
  Palette,
  Progress,
  Scale,
  Settings,
  SettingsSchema,
  Summary,
  TreeInspection,
  TreeText,
  ValidationError,
  TreeVersion,
} from "@neherlab/treeknit-wasm";

export interface StatelessApi {
  defaultSettings(): Settings;
  settingsSchema(k: number, settings: Settings): SettingsSchema;
  inspectTree(label: string, text: string): TreeInspection;
  overlap(trees: TreeText[]): Overlap;
  validate(request: AnalysisRequest): ValidationError[];
  readRequest(text: string): AnalysisRequest;
  requestFile(request: AnalysisRequest): OutputFile;
  treeLabels(fileNames: string[], existingLabels: string[]): string[];
  version(): AppVersion;
  palette(): Palette;
}

export interface SessionApi {
  summary(): Summary;
  files(): FileEntry[];
  fileText(path: string): string;
  zip(): Uint8Array;
  commandLine(): string;
  pairView(pair: number, version: TreeVersion, scale: Scale): PairView;
  argView(scale: Scale): ArgView | undefined;
  constellation(): ConstellationTable;
  figure(pair: number, version: TreeVersion, options: FigureOptions): string;
  argFigure(options: FigureOptions): string;
}

export interface WorkerApi extends StatelessApi, SessionApi {
  init(module: WebAssembly.Module): void;
  run(request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>): Summary;
}
