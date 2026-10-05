import type { SettingFields, TreeText, ValidationError } from "@neherlab/treeknit-wasm";

import { NUMBER_SETTINGS } from "../settings/draft";

export const TREE_LIST_FIELD = "trees";

export interface FieldErrors {
  byField: ReadonlyMap<string, readonly ValidationError[]>;
  general: readonly ValidationError[];
}

export function settingField(key: keyof SettingFields): string {
  return `settings.${key}`;
}

export function treeField(index: number, part: keyof TreeText): string {
  return `trees[${String(index)}].${part}`;
}

export function seqLengthField(index: number): string {
  return `${settingField("seqLengths")}[${String(index)}]`;
}

export function shownFields(treeCount: number, seqLengthsOn: boolean): ReadonlySet<string> {
  const indices = Array.from({ length: treeCount }, (_, index) => index);

  return new Set([
    ...NUMBER_SETTINGS.map((key) => settingField(key)),
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
  return joinedMessage(errors.byField.get(field) ?? []);
}

export function joinedMessage(errors: readonly ValidationError[]): string | undefined {
  return errors.length === 0 ? undefined : errors.map(({ message }) => message).join("; ");
}
