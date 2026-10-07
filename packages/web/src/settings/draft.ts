import type { NumberSetting, SettingFields, Settings, ToggleSetting } from "@neherlab/treeknit-wasm";
import { fromKeys } from "remeda";

import { everyVariantOf } from "../variants";

type SettingKeysOf<Kind> = {
  [Key in keyof SettingFields]: SettingFields[Key] extends Kind ? Key : never;
}[keyof SettingFields];

export type NumberSettingKey = Exclude<SettingKeysOf<NumberSetting>, "seqLengths">;

export type ToggleSettingKey = SettingKeysOf<ToggleSetting>;

export const NUMBER_SETTINGS = everyVariantOf<NumberSettingKey>()(["gamma", "nMcmcIt", "rounds", "seed"]);

export type SettingsDraft = Record<NumberSettingKey, number> & { seqLengths: Record<string, number> };

export const ENTER_A_NUMBER = "Enter a number";

export function draftFromSettings(settings: Settings, treeIds: readonly string[]): SettingsDraft {
  return {
    ...numberDraft((key) => settings[key] ?? Number.NaN),
    seqLengths: Object.fromEntries(
      (settings.seqLengths ?? []).flatMap((length, index) => {
        const id = treeIds[index];

        return id === undefined ? [] : [[id, length]];
      }),
    ),
  };
}

export function nextDraft(
  draft: SettingsDraft,
  settings: Settings,
  treeIds: readonly string[],
  replaced: boolean,
): SettingsDraft {
  return replaced ? draftFromSettings(settings, treeIds) : syncDraft(draft, settings, treeIds);
}

export function syncDraft(draft: SettingsDraft, settings: Settings, treeIds: readonly string[]): SettingsDraft {
  const stored = draftFromSettings(settings, treeIds);

  return {
    ...numberDraft((key) => keepDraft(draft[key], stored[key])),
    seqLengths: Object.fromEntries(
      Object.entries(stored.seqLengths).map(([id, length]) => [id, keepDraft(draft.seqLengths[id], length)]),
    ),
  };
}

function numberDraft(value: (key: NumberSettingKey) => number): Record<NumberSettingKey, number> {
  return fromKeys(NUMBER_SETTINGS, value);
}

function keepDraft(current: number | undefined, value: number): number {
  return current !== undefined && Number.isNaN(current) ? current : value;
}

export function hasDraft(draft: SettingsDraft): boolean {
  return [...NUMBER_SETTINGS.map((key) => draft[key]), ...Object.values(draft.seqLengths)].some((value) =>
    Number.isNaN(value),
  );
}

export function withNumberSetting(settings: Settings, key: NumberSettingKey, value: number): Settings | null {
  return Number.isFinite(value) ? { ...settings, [key]: value } : null;
}

export function withSeqLength(
  settings: Settings,
  treeIds: readonly string[],
  treeId: string,
  value: number,
): Settings | null {
  const index = treeIds.indexOf(treeId);
  const lengths = settings.seqLengths;

  if (!Number.isFinite(value) || index === -1 || lengths === null || lengths === undefined) {
    return null;
  }

  return { ...settings, seqLengths: lengths.with(index, value) };
}

export function draftError(value: number): string | undefined {
  return Number.isNaN(value) ? ENTER_A_NUMBER : undefined;
}
