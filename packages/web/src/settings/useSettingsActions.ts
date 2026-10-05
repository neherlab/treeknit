import type { ResolveMode, Settings } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useWorkspaceStore } from "../workspace/context";
import { type NumberSettingKey, type ToggleSettingKey, withNumberSetting, withSeqLength } from "./draft";

export interface SettingsActions {
  setNumber(key: NumberSettingKey, value: number): void;
  setSeqLength(treeId: string, value: number): void;
  setToggle(key: ToggleSettingKey, value: boolean): void;
  setResolve(mode: ResolveMode): void;
  setSeqLengthsEnabled(enabled: boolean): Promise<void>;
}

export function useSettingsActions(): SettingsActions {
  const store = useWorkspaceStore();

  return useMemo(() => {
    const update = (change: (settings: Settings) => Settings | null): void => {
      const state = store.getState();
      const next = change(state.settings);

      if (next !== null) {
        state.setSettings(next);
      }
    };

    return {
      setNumber(key, value) {
        update((settings) => withNumberSetting(settings, key, value));
      },
      setSeqLength(treeId, value) {
        const treeIds = store.getState().trees.map(({ id }) => id);

        update((settings) => withSeqLength(settings, treeIds, treeId, value));
      },
      setToggle(key, value) {
        update((settings) => ({ ...settings, [key]: value }));
      },
      setResolve(mode) {
        update((settings) => ({ ...settings, resolve: mode }));
      },
      async setSeqLengthsEnabled(enabled) {
        await store.getState().setSeqLengthsEnabled(enabled);
      },
    };
  }, [store]);
}
