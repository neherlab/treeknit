import type { Field, ValidationError } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";

import { NUMBER_SETTINGS } from "../settings/draft";

export interface FieldErrors {
  byField: readonly FieldErrorGroup[];
  general: readonly ValidationError[];
}

export interface FieldErrorGroup {
  field: Field;
  errors: readonly ValidationError[];
}

export function shownFields(treeCount: number, seqLengthsOn: boolean): readonly Field[] {
  const indices = Array.from({ length: treeCount }, (_, index) => index);

  return [
    ...NUMBER_SETTINGS.map((key): Field => ({ kind: "setting", key })),
    { kind: "trees" },
    ...indices.flatMap((index): Field[] => [
      { kind: "treeLabel", index },
      { kind: "treeNewick", index },
    ]),
    ...(seqLengthsOn ? indices.map((index): Field => ({ kind: "seqLength", index })) : []),
  ];
}

export function groupFieldErrors(errors: readonly ValidationError[], shown: readonly Field[]): FieldErrors {
  const byField: { field: Field; errors: ValidationError[] }[] = [];
  const general: ValidationError[] = [];

  for (const error of errors) {
    const { field } = error;

    if (field !== null && shown.some((candidate) => isDeepEqual(candidate, field))) {
      const group = byField.find((existing) => isDeepEqual(existing.field, field));

      if (group === undefined) {
        byField.push({ field, errors: [error] });
      } else {
        group.errors.push(error);
      }
    } else {
      general.push(error);
    }
  }

  return { byField, general };
}

export function fieldErrorsAt(errors: FieldErrors, field: Field): readonly ValidationError[] {
  return errors.byField.find((group) => isDeepEqual(group.field, field))?.errors ?? [];
}

export function fieldMessage(errors: FieldErrors, field: Field): string | undefined {
  return joinedMessage(fieldErrorsAt(errors, field));
}

export function joinedMessage(errors: readonly ValidationError[]): string | undefined {
  return errors.length === 0 ? undefined : errors.map(({ message }) => message).join("; ");
}
