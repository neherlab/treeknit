import type { DOMAttributes } from "react";
import { useKeyboard } from "react-aria";
import type { KeyboardEvent } from "react-aria-components";

import { useRunAnalysis } from "../workspace/useRunAnalysis";
import { isRunShortcut } from "./runControl";
import { useRunReadiness } from "./useRunReadiness";

export function useRunShortcut(): DOMAttributes<HTMLElement> {
  const { run } = useRunAnalysis();
  const { blockedReason } = useRunReadiness();

  const { keyboardProps } = useKeyboard({
    onKeyDown(event: KeyboardEvent) {
      if (!isRunShortcut(event.key, event.ctrlKey, event.metaKey)) {
        event.continuePropagation();

        return;
      }

      event.preventDefault();

      if (blockedReason === null) {
        run();
      }
    },
  });

  return keyboardProps;
}
