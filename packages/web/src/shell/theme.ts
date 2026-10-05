import type { ThemeProviderProps } from "next-themes";

export const THEME_PROVIDER_PROPS = {
  attribute: "class",
  defaultTheme: "system",
  enableSystem: true,
  disableTransitionOnChange: true,
} satisfies ThemeProviderProps;

export const CLIENT_THEME_PROVIDER_PROPS = {
  ...THEME_PROVIDER_PROPS,
  scriptProps: { type: "text/plain" },
} satisfies ThemeProviderProps;

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
