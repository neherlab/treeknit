import { useCallback, useState } from "react";

export function useElementWidth(): [(element: HTMLElement | null) => (() => void) | undefined, number] {
  const [width, setWidth] = useState(0);

  const ref = useCallback((element: HTMLElement | null) => {
    if (element === null) {
      return undefined;
    }

    const observer = new ResizeObserver(() => {
      setWidth(element.offsetWidth);
    });

    observer.observe(element);

    return () => {
      observer.disconnect();
    };
  }, []);

  return [ref, width];
}
