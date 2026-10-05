import { createContext, useCallback, useContext } from "react";
import { useStore } from "zustand";

import { type WorkspaceRuntime, workspaceSnapshot } from "./runtime";
import { selectRequest, type WorkspaceState, type WorkspaceStore } from "./store";

export const WorkspaceContext = createContext<WorkspaceRuntime | null>(null);

export function useWorkspace<T>(selector: (state: WorkspaceState) => T): T {
  return useStore(useWorkspaceRuntime().store, selector);
}

export function useWorkspaceStore(): WorkspaceStore {
  return useWorkspaceRuntime().store;
}

export function useCurrentRequest() {
  return useWorkspace(selectRequest);
}

export function usePersistenceSwitch(): PersistenceSwitch {
  const { store, persistence } = useWorkspaceRuntime();
  const enabled = useStore(store, (state) => state.persistence);

  const setEnabled = useCallback(
    async (next: boolean) => {
      await (next ? persistence.enable(workspaceSnapshot(store.getState())) : persistence.disable());
    },
    [store, persistence],
  );

  return { enabled, setEnabled };
}

export interface PersistenceSwitch {
  enabled: boolean;
  setEnabled: (enabled: boolean) => Promise<void>;
}

function useWorkspaceRuntime(): WorkspaceRuntime {
  const runtime = useContext(WorkspaceContext);

  if (runtime === null) {
    throw new Error("Workspace hooks need a WorkspaceProvider.");
  }

  return runtime;
}
