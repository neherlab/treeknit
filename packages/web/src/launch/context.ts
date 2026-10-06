import { createContext, use, useCallback, useSyncExternalStore } from "react";

import { IDLE_LAUNCH, type LaunchControl, type LaunchState } from "./LinkLaunch";

export const LaunchContext = createContext<LaunchControl>(IDLE_LAUNCH);

export function useLaunch(): LaunchView {
  const control = use(LaunchContext);
  const subscribe = useCallback((listener: () => void) => control.subscribe(listener), [control]);
  const snapshot = useCallback(() => control.getSnapshot(), [control]);

  return { state: useSyncExternalStore(subscribe, snapshot), control };
}

export interface LaunchView {
  state: LaunchState;
  control: LaunchControl;
}
