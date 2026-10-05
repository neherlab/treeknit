import type { ValidationError } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useSettingsDraftPending } from "../settings/SettingsFormProvider";
import { useWorkspace } from "../workspace/context";
import { useFieldErrors } from "../workspace/useFieldErrors";
import { runBlockedReason, visibleGeneralErrors } from "./runControl";

export interface RunReadinessState {
  blockedReason: string | null;
  generalErrors: readonly ValidationError[];
}

export function useRunReadiness(): RunReadinessState {
  const treeCount = useWorkspace((state) => state.trees.length);
  const hasDraft = useSettingsDraftPending();
  const errors = useFieldErrors();

  return useMemo(() => {
    const all = [...errors.general, ...[...errors.byField.values()].flat()];

    return {
      blockedReason: runBlockedReason({ treeCount, hasDraft, errors: all }),
      generalErrors: visibleGeneralErrors(treeCount, errors.general),
    };
  }, [errors, hasDraft, treeCount]);
}
