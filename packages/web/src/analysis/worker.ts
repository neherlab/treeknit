import * as wasm from "@neherlab/treeknit-wasm";
import { initSync, Session, setPanicSink } from "@neherlab/treeknit-wasm";
import type { AnalysisRequest, Progress } from "@neherlab/treeknit-wasm";
import { expose, proxy, type ProxyMarked, transfer, transferHandlers } from "comlink";
import { omit } from "remeda";

import { panicTextHandler, PendingPanicText } from "./panicText";
import { ProgressThrottle } from "./progressThrottle";
import { type SessionApi, WORKER_ONLY_EXPORTS, type WorkerApi } from "./protocol";

class AnalysisWorker implements WorkerApi {
  readonly stateless = proxy(omit(wasm, WORKER_ONLY_EXPORTS));
  #session: Session | undefined;

  get session(): SessionApi & ProxyMarked {
    return proxy(this.#current());
  }

  init(module: WebAssembly.Module): void {
    initSync({ module });
    setPanicSink((text) => {
      pendingPanicText.record(text);
    });
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

  zip() {
    const bytes = this.#current().zip();

    return transfer(bytes, [bytes.buffer]);
  }

  #current(): Session {
    if (this.#session === undefined) {
      throw new Error("This worker holds no run.");
    }

    return this.#session;
  }
}

const pendingPanicText = new PendingPanicText();

const throwHandler = transferHandlers.get("throw");

if (throwHandler === undefined) {
  throw new Error("comlink has no transfer handler for thrown values.");
}

transferHandlers.set("throw", panicTextHandler(throwHandler, pendingPanicText));

expose(new AnalysisWorker());

function reportDeliveryError(cause: unknown): void {
  console.error("Sending run progress to the page failed.", cause);
}
