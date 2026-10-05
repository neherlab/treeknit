import type { Progress } from "@neherlab/treeknit-wasm";

export const PROGRESS_INTERVAL_MS = 100;

export class ProgressThrottle {
  readonly #forward: (progress: Progress) => void;
  readonly #now: () => number;
  #lastForwardAt = Number.NEGATIVE_INFINITY;
  #pending: Progress | undefined;

  constructor(forward: (progress: Progress) => void, now: () => number) {
    this.#forward = forward;
    this.#now = now;
  }

  push(progress: Progress): void {
    const now = this.#now();

    if (progress.phase === "done" || now - this.#lastForwardAt >= PROGRESS_INTERVAL_MS) {
      this.#send(progress, now);
    } else {
      this.#pending = progress;
    }
  }

  flush(): void {
    if (this.#pending !== undefined) {
      this.#send(this.#pending, this.#now());
    }
  }

  #send(progress: Progress, now: number): void {
    this.#pending = undefined;
    this.#lastForwardAt = now;
    this.#forward(progress);
  }
}
