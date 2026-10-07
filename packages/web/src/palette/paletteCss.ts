import type { Palette, ThemeColors } from "@neherlab/treeknit-wasm";

export const NO_MCC_VARIABLE = "--color-mcc-none";

export function mccVariable(slot: number): string {
  return `--color-mcc-${String(slot)}`;
}

export function mccVariables(colors: ThemeColors<string>): [string, string][] {
  return [
    ...colors.mcc.map((color, slot): [string, string] => [mccVariable(slot), color]),
    [NO_MCC_VARIABLE, colors.noMcc],
  ];
}

export function paletteCss(palette: Palette): string {
  return [rule(":root", palette.light), rule(":root.dark", palette.dark), rule(".light-scope", palette.light)].join(
    "\n",
  );
}

export function applyPalette(target: Document, palette: Palette): () => void {
  const sheet = new CSSStyleSheet();

  sheet.replaceSync(paletteCss(palette));
  target.adoptedStyleSheets = [...target.adoptedStyleSheets, sheet];

  return () => {
    target.adoptedStyleSheets = target.adoptedStyleSheets.filter((adopted) => adopted !== sheet);
  };
}

function rule(selector: string, colors: ThemeColors<string>): string {
  const declarations = mccVariables(colors).map(([name, color]) => `  ${name}: ${color};`);

  return [`${selector} {`, ...declarations, "}"].join("\n");
}
