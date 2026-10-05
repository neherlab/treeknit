import { initSync, Session } from "@neherlab/treeknit-wasm";
import * as wasm from "@neherlab/treeknit-wasm";
import type {
  AnalysisRequest,
  FigureOptions,
  Progress,
  Scale,
  Settings,
  TreeText,
  Version,
} from "@neherlab/treeknit-wasm";
import { expose, transfer } from "comlink";

import { ProgressThrottle } from "./progressThrottle";
import type { WorkerApi } from "./protocol";

class AnalysisWorker implements WorkerApi {
  #session: Session | undefined;

  init(module: WebAssembly.Module): void {
    initSync({ module });
  }

  defaultSettings() {
    return wasm.defaultSettings();
  }

  settingsSchema(k: number, settings: Settings) {
    return wasm.settingsSchema(k, settings);
  }

  inspectTree(label: string, text: string) {
    return wasm.inspectTree(label, text);
  }

  overlap(trees: TreeText[]) {
    return wasm.overlap(trees);
  }

  validate(request: AnalysisRequest) {
    return wasm.validate(request);
  }

  readRequest(text: string) {
    return wasm.readRequest(text);
  }

  requestFile(request: AnalysisRequest) {
    return wasm.requestFile(request);
  }

  treeLabels(fileNames: string[], existingLabels: string[]) {
    return wasm.treeLabels(fileNames, existingLabels);
  }

  version() {
    return wasm.version();
  }

  palette() {
    return wasm.palette();
  }

  run(request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>) {
    const throttle = new ProgressThrottle(
      (progress) => {
        onProgress(progress).catch(ignore);
      },
      () => performance.now(),
    );

    const session = Session.run(request, (progress) => {
      throttle.push(progress);
    });

    throttle.flush();
    this.#session?.free();
    this.#session = session;

    return session.summary();
  }

  summary() {
    return this.#current().summary();
  }

  files() {
    return this.#current().files();
  }

  fileText(path: string) {
    return this.#current().fileText(path);
  }

  zip() {
    const bytes = this.#current().zip();

    return transfer(bytes, [bytes.buffer]);
  }

  commandLine() {
    return this.#current().commandLine();
  }

  pairView(pair: number, version: Version, scale: Scale) {
    return this.#current().pairView(pair, version, scale);
  }

  argView(scale: Scale) {
    return this.#current().argView(scale);
  }

  constellation() {
    return this.#current().constellation();
  }

  figure(pair: number, version: Version, options: FigureOptions) {
    return this.#current().figure(pair, version, options);
  }

  argFigure(options: FigureOptions) {
    return this.#current().argFigure(options);
  }

  #current(): Session {
    if (this.#session === undefined) {
      throw new Error("This worker holds no run.");
    }

    return this.#session;
  }
}

expose(new AnalysisWorker());

function ignore(): undefined {
  return undefined;
}
