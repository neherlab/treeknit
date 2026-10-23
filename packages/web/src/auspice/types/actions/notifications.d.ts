import type { AuspiceThunk } from "auspice/src/state";

export declare function errorNotification(params?: {
  message?: string;
  details?: string;
  autoClose?: boolean;
}): AuspiceThunk;
