import type { ArgOutcome, ArgView } from "@neherlab/treeknit-wasm";

export const ARG_MISSING = "the run returned no ARG";

export function argFailure(outcome: ArgOutcome | null, view: ArgView | null | undefined): string | undefined {
  if (outcome?.status === "failed") {
    return outcome.message.replace(/\.$/u, "");
  }

  return view === null ? ARG_MISSING : undefined;
}
