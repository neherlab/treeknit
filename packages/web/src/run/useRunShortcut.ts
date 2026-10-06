import type { ValidationError } from "@neherlab/treeknit-wasm";
import { CancelledError, useQueryClient } from "@tanstack/react-query";
import { type RefObject, useEffect, useEffectEvent } from "react";
import { useFormContext } from "react-hook-form";

import { useAnalysisClient } from "../analysis/context";
import { validationQuery } from "../analysis/queries";
import { hasDraft, type SettingsDraft } from "../settings/draft";
import { useWorkspaceStore } from "../workspace/context";
import { selectRequest, selectTextIds } from "../workspace/store";
import { useRunAnalysis } from "../workspace/useRunAnalysis";
import { isBehindModal, isRunShortcut, runBlockedReason } from "./runControl";

export function useRunShortcut(workspace: RefObject<HTMLElement | null>): void {
  const { run } = useRunAnalysis();
  const client = useAnalysisClient();
  const queryClient = useQueryClient();
  const store = useWorkspaceStore();
  const { getValues } = useFormContext<SettingsDraft>();

  const runWhenReady = useEffectEvent(async (): Promise<void> => {
    const state = store.getState();
    const request = selectRequest(state);
    let errors: readonly ValidationError[];

    try {
      errors = await queryClient.query({
        ...validationQuery(client, request, selectTextIds(state)),
        staleTime: "static",
      });
    } catch (error) {
      if (!(error instanceof CancelledError)) {
        console.error("The run shortcut could not check the trees and settings", error);
      }

      return;
    }

    if (selectRequest(store.getState()) !== request) {
      return;
    }

    const reason = runBlockedReason({
      treeCount: request.trees.length,
      hasDraft: hasDraft(getValues()),
      validation: "checked",
      errors,
    });

    if (reason === null) {
      run();
    }
  });

  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    const root = workspace.current;

    if (!isRunShortcut(event) || root === null || isBehindModal(root)) {
      return;
    }

    event.preventDefault();
    commitFocusedField(root);
    void runWhenReady();
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

function commitFocusedField(root: HTMLElement): void {
  const focused = document.activeElement;

  if (focused instanceof HTMLElement && root.contains(focused)) {
    focused.blur();
    focused.focus({ preventScroll: true });
  }
}
