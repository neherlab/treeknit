import type { ValidationError } from "@neherlab/treeknit-wasm";

export const SETTING_FIELDS = ["settings.gamma", "settings.nMcmcIt", "settings.rounds", "settings.seed"] as const;

export const TREE_LIST_FIELD = "trees";

export interface FieldErrors {
  byField: ReadonlyMap<string, readonly ValidationError[]>;
  general: readonly ValidationError[];
}

export function treeField(index: number, part: "label" | "newick"): string {
  return `trees[${String(index)}].${part}`;
}

export function seqLengthField(index: number): string {
  return `settings.seqLengths[${String(index)}]`;
}

export function shownFields(treeCount: number, seqLengthsOn: boolean): ReadonlySet<string> {
  const indices = Array.from({ length: treeCount }, (_, index) => index);

  return new Set([
    ...SETTING_FIELDS,
    TREE_LIST_FIELD,
    ...indices.flatMap((index) => [treeField(index, "label"), treeField(index, "newick")]),
    ...(seqLengthsOn ? indices.map((index) => seqLengthField(index)) : []),
  ]);
}

export function groupFieldErrors(errors: readonly ValidationError[], shown: ReadonlySet<string>): FieldErrors {
  const byField = new Map<string, ValidationError[]>();
  const general: ValidationError[] = [];

  for (const error of errors) {
    if (error.field !== null && shown.has(error.field)) {
      byField.set(error.field, [...(byField.get(error.field) ?? []), error]);
    } else {
      general.push(error);
    }
  }

  return { byField, general };
}

export function fieldMessage(errors: FieldErrors, field: string): string | undefined {
  const messages = errors.byField.get(field)?.map(({ message }) => message) ?? [];

  return messages.length === 0 ? undefined : messages.join("; ");
}

export function errorProps(message: string | undefined): { errorMessage: string } | undefined {
  return message === undefined ? undefined : { errorMessage: message };
}
