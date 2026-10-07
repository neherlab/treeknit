import { useCallback, useSyncExternalStore } from "react";

export function useMediaQueries<T>(queries: readonly string[], snapshot: () => T, serverSnapshot?: () => T): T {
  const subscribe = useCallback(
    (onChange: () => void) => {
      const lists = queries.map((query) => matchMedia(query));

      for (const list of lists) {
        list.addEventListener("change", onChange);
      }

      return () => {
        for (const list of lists) {
          list.removeEventListener("change", onChange);
        }
      };
    },
    [queries],
  );

  return useSyncExternalStore(subscribe, snapshot, serverSnapshot);
}
