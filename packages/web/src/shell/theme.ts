export type ThemeName = "light" | "dark";

export interface ThemeToggle {
  label: string;
  next: ThemeName;
}

const TOGGLES: Record<ThemeName, ThemeToggle> = {
  light: { label: "Switch to dark theme", next: "dark" },
  dark: { label: "Switch to light theme", next: "light" },
};

export function themeToggle(resolvedTheme: string | undefined): ThemeToggle {
  return resolvedTheme === "dark" ? TOGGLES.dark : TOGGLES.light;
}
