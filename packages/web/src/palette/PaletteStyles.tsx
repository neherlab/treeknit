import { useEffect } from "react";

import { usePalette } from "../analysis/queries";
import { applyPalette } from "./paletteCss";

export function PaletteStyles() {
  const { data: palette } = usePalette();

  useEffect(() => {
    if (palette === undefined) {
      return undefined;
    }

    return applyPalette(document, palette);
  }, [palette]);

  return null;
}
