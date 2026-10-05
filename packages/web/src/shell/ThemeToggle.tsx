import { useTheme } from "next-themes";
import { useCallback } from "react";
import MoonIcon from "~icons/lucide/moon";
import SunIcon from "~icons/lucide/sun";

import { IconButton } from "../ui/IconButton";
import { themeToggle } from "./theme";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const toggle = themeToggle(resolvedTheme);

  const switchTheme = useCallback(() => {
    setTheme(toggle.next);
  }, [setTheme, toggle.next]);

  return (
    <IconButton
      label={toggle.label}
      icon={toggle.next === "dark" ? MoonIcon : SunIcon}
      tooltipPlacement="bottom"
      onPress={switchTheme}
    />
  );
}
