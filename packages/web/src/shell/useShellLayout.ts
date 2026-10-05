import { useSyncExternalStore } from "react";

import { SHELL_MEDIA_QUERIES, shellLayout, type ShellLayout } from "./layout";

export function useShellLayout(): ShellLayout {
  return useSyncExternalStore(subscribe, currentLayout);
}

function subscribe(onChange: () => void): () => void {
  const queries = SHELL_MEDIA_QUERIES.map((query) => window.matchMedia(query));

  for (const query of queries) {
    query.addEventListener("change", onChange);
  }

  return () => {
    for (const query of queries) {
      query.removeEventListener("change", onChange);
    }
  };
}

function currentLayout(): ShellLayout {
  return shellLayout(window.innerWidth);
}
