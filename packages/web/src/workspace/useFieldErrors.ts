import { useMemo } from "react";

import { useValidation } from "../analysis/queries";
import { useCurrentRequest } from "./context";
import { type FieldErrors, groupFieldErrors, shownFields } from "./fieldErrors";

export function useFieldErrors(): FieldErrors {
  const request = useCurrentRequest();
  const { data } = useValidation(request);
  const treeCount = request.trees.length;
  const seqLengthsOn = request.settings?.seqLengths !== null && request.settings?.seqLengths !== undefined;

  return useMemo(
    () => groupFieldErrors(data ?? [], shownFields(treeCount, seqLengthsOn)),
    [data, treeCount, seqLengthsOn],
  );
}
