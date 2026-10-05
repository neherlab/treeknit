import { useCallback, useMemo } from "react";

import { useValidation } from "../analysis/queries";
import { validationState, type ValidationState } from "../run/runControl";
import { useCurrentRequest, useCurrentTextIds } from "./context";
import { type FieldErrors, groupFieldErrors, shownFields } from "./fieldErrors";

export interface ValidationProgress {
  state: ValidationState;
  retry: () => void;
}

export function useValidationProgress(): ValidationProgress {
  const { status, isPlaceholderData, refetch } = useValidation(useCurrentRequest(), useCurrentTextIds());

  const retry = useCallback(() => {
    void refetch();
  }, [refetch]);

  return { state: validationState({ status, isPlaceholderData }), retry };
}

export function useFieldErrors(): FieldErrors {
  const request = useCurrentRequest();
  const { data } = useValidation(request, useCurrentTextIds());
  const treeCount = request.trees.length;
  const seqLengthsOn = request.settings?.seqLengths !== null && request.settings?.seqLengths !== undefined;

  return useMemo(
    () => groupFieldErrors(data ?? [], shownFields(treeCount, seqLengthsOn)),
    [data, treeCount, seqLengthsOn],
  );
}
