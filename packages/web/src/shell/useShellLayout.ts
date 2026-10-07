import { useMediaQueries } from "../ui/useMediaQueries";
import { SHELL_MEDIA_QUERIES, shellLayout, type ShellLayout } from "./layout";

export function useShellLayout(): ShellLayout {
  return useMediaQueries(SHELL_MEDIA_QUERIES, currentLayout);
}

function currentLayout(): ShellLayout {
  return shellLayout(window.innerWidth);
}
