import { createContext, use } from "react";
import { useStore } from "zustand";

import { IDLE_LAUNCH, type LaunchControl, type LaunchState } from "./LinkLaunch";

export const LaunchContext = createContext<LaunchControl>(IDLE_LAUNCH);

export function useLaunch(): LaunchView {
  const control = use(LaunchContext);

  return { state: useStore(control.state), control };
}

export interface LaunchView {
  state: LaunchState;
  control: LaunchControl;
}
