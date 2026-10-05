import type { DOMAttributes } from "react";
import { useKeyboard } from "react-aria";

export function useEscapeKey(onEscape: (() => void) | null): DOMAttributes<Element> {
  const { keyboardProps } = useKeyboard({
    onKeyDown: (event) => {
      if (onEscape !== null && event.key === "Escape" && !event.isDefaultPrevented()) {
        onEscape();
      } else {
        event.continuePropagation();
      }
    },
  });

  return keyboardProps;
}
