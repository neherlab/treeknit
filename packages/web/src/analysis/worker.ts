import {
  defaultSettings,
  initSync,
  inspectTree,
  overlap,
  palette,
  readRequest,
  requestFile,
  Session,
  settingsSchema,
  treeLabels,
  validate,
  version,
} from "@neherlab/treeknit-wasm";
import type { AnalysisRequest, Progress } from "@neherlab/treeknit-wasm";
import { expose, transfer } from "comlink";

import { ProgressThrottle } from "./progressThrottle";
import type { SessionArgs, WorkerApi } from "./protocol";

class AnalysisWorker implements WorkerApi {
  readonly defaultSettings = defaultSettings;
  readonly settingsSchema = settingsSchema;
  readonly inspectTree = inspectTree;
  readonly overlap = overlap;
  readonly validate = validate;
  readonly readRequest = readRequest;
  readonly requestFile = requestFile;
  readonly treeLabels = treeLabels;
  readonly version = version;
  readonly palette = palette;
  #session: Session | undefined;

  init(module: WebAssembly.Module): void {
    initSync({ module });
  }

  run(request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>) {
    const throttle = new ProgressThrottle(
      (progress) => {
        onProgress(progress).catch(reportDeliveryError);
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

  fileText(...args: SessionArgs<"fileText">) {
    return this.#current().fileText(...args);
  }

  zip() {
    const bytes = this.#current().zip();

    return transfer(bytes, [bytes.buffer]);
  }

  commandLine() {
    return this.#current().commandLine();
  }

  pairView(...args: SessionArgs<"pairView">) {
    return this.#current().pairView(...args);
  }

  argView(...args: SessionArgs<"argView">) {
    return this.#current().argView(...args);
  }

  constellation() {
    return this.#current().constellation();
  }

  figure(...args: SessionArgs<"figure">) {
    return this.#current().figure(...args);
  }

  argFigure(...args: SessionArgs<"argFigure">) {
    return this.#current().argFigure(...args);
  }

  #current(): Session {
    if (this.#session === undefined) {
      throw new Error("This worker holds no run.");
    }

    return this.#session;
  }
}

expose(new AnalysisWorker());

function reportDeliveryError(cause: unknown): void {
  console.error("Sending run progress to the page failed.", cause);
}
