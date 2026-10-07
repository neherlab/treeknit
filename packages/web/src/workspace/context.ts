import { createContext, useCallback, useContext } from "react";
import { useStore } from "zustand";

import type { FocusRequest, FocusStore } from "../drawing/focus";
import type { PersistenceProblem } from "../persistence/persistence";
import { type WorkspaceRuntime, workspaceSnapshot } from "./runtime";
import { selectRequest, selectTextIds, type WorkspaceState, type WorkspaceStore } from "./store";

export const WorkspaceContext = createContext<WorkspaceRuntime | null>(null);

export function useWorkspace<T>(selector: (state: WorkspaceState) => T): T {
  return useStore(useWorkspaceRuntime().store, selector);
}

export function useWorkspaceStore(): WorkspaceStore {
  return useWorkspaceRuntime().store;
}

export function useFocusRequest(): FocusRequest | null {
  return useStore(useWorkspaceRuntime().focus, (state) => state.request);
}

export function useFocusStore(): FocusStore {
  return useWorkspaceRuntime().focus;
}

export function useCurrentRequest() {
  return useWorkspace(selectRequest);
}

export function useCurrentTextIds() {
  return useWorkspace(selectTextIds);
}

export function usePersistenceSwitch(): PersistenceSwitch {
  const { store, persistence } = useWorkspaceRuntime();

  const { enabled, problem } = useStore(persistence.state);

  const setEnabled = useCallback(
    async (next: boolean) => {
      await (next ? persistence.enable(workspaceSnapshot(store.getState())) : persistence.disable());
    },
    [store, persistence],
  );

  const saveNow = useCallback(async () => persistence.saveNow(), [persistence]);

  return { enabled, problem, setEnabled, saveNow };
}

export interface PersistenceSwitch {
  enabled: boolean;
  problem: PersistenceProblem | null;
  setEnabled: (enabled: boolean) => Promise<void>;
  saveNow: () => Promise<void>;
}

function useWorkspaceRuntime(): WorkspaceRuntime {
  const runtime = useContext(WorkspaceContext);

  if (runtime === null) {
    throw new Error("Workspace hooks need a WorkspaceProvider.");
  }

  return runtime;
}
