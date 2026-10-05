export type CopyState = "idle" | "copied" | "failed";

export const COPY_FEEDBACK_MS = 2000;

export interface CopyFeedback {
  copy(text: string): Promise<void>;
  dispose(): void;
}

export function copyFeedback(
  write: (text: string) => Promise<void>,
  onStateChange: (state: CopyState) => void,
): CopyFeedback {
  let reset: ReturnType<typeof setTimeout> | undefined;

  function cancelReset() {
    clearTimeout(reset);
    reset = undefined;
  }

  function show(state: CopyState) {
    cancelReset();
    onStateChange(state);
    reset = setTimeout(() => {
      reset = undefined;
      onStateChange("idle");
    }, COPY_FEEDBACK_MS);
  }

  return {
    async copy(text) {
      try {
        await write(text);
      } catch {
        show("failed");

        return;
      }

      show("copied");
    },
    dispose: cancelReset,
  };
}
