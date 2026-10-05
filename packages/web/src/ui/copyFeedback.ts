export type CopyState = "idle" | "copied" | "failed";

export const COPY_FEEDBACK_MS = 2000;

export interface CopyFeedback<T> {
  copy(content: T): Promise<void>;
  attach(): () => void;
}

export function copyFeedback<T>(
  write: (content: T) => Promise<void>,
  onStateChange: (state: CopyState) => void,
): CopyFeedback<T> {
  let reset: ReturnType<typeof setTimeout> | undefined;
  let latestRequest = 0;
  let attached = false;

  function cancelReset() {
    clearTimeout(reset);
    reset = undefined;
  }

  function show(request: number, state: CopyState) {
    if (!attached || request !== latestRequest) {
      return;
    }

    cancelReset();
    onStateChange(state);
    reset = setTimeout(() => {
      reset = undefined;
      onStateChange("idle");
    }, COPY_FEEDBACK_MS);
  }

  return {
    async copy(content) {
      latestRequest += 1;
      const request = latestRequest;

      try {
        await write(content);
      } catch {
        show(request, "failed");

        return;
      }

      show(request, "copied");
    },
    attach() {
      attached = true;

      return () => {
        attached = false;
        latestRequest += 1;

        if (reset !== undefined) {
          cancelReset();
          onStateChange("idle");
        }
      };
    },
  };
}
