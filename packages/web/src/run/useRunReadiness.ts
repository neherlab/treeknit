import type { ValidationError } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useSettingsDraftPending } from "../settings/SettingsFormProvider";
import { useWorkspace } from "../workspace/context";
import { useFieldErrors, useValidationChecking } from "../workspace/useFieldErrors";
import { runBlockedReason, visibleGeneralErrors } from "./runControl";

export interface RunReadinessState {
  blockedReason: string | null;
  generalErrors: readonly ValidationError[];
}

export function useRunReadiness(): RunReadinessState {
  const treeCount = useWorkspace((state) => state.trees.length);
  const hasDraft = useSettingsDraftPending();
  const errors = useFieldErrors();
  const checking = useValidationChecking();

  return useMemo(() => {
    const all = [...errors.general, ...[...errors.byField.values()].flat()];

    return {
      blockedReason: runBlockedReason({ treeCount, hasDraft, checking, errors: all }),
      generalErrors: visibleGeneralErrors(treeCount, errors.general),
    };
  }, [checking, errors, hasDraft, treeCount]);
}
