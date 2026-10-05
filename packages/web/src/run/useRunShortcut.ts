import { type RefObject, useEffect, useEffectEvent } from "react";

import { useRunAnalysis } from "../workspace/useRunAnalysis";
import { isBehindModal, isRunShortcut } from "./runControl";
import { useRunReadiness } from "./useRunReadiness";

export function useRunShortcut(workspace: RefObject<HTMLElement | null>): void {
  const { run } = useRunAnalysis();
  const { blockedReason } = useRunReadiness();

  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    const root = workspace.current;

    if (!isRunShortcut(event) || root === null || isBehindModal(root)) {
      return;
    }

    event.preventDefault();

    if (blockedReason === null) {
      run();
    }
  });

  useEffect(() => {
    const listener = (event: KeyboardEvent): void => {
      onKeyDown(event);
    };

    document.addEventListener("keydown", listener, { capture: true });

    return () => {
      document.removeEventListener("keydown", listener, { capture: true });
    };
  }, []);
}
