import { type RefObject, useLayoutEffect, useRef } from "react";

import type { MeasuredSize } from "./viewState";

export interface MeasuredElements {
  areaRef: RefObject<HTMLDivElement | null>;
  canvasRef: RefObject<HTMLDivElement | null>;
}

export function useMeasuredSize(onMeasure: (measured: MeasuredSize) => void): MeasuredElements {
  const areaRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const measure = () => {
      const area = areaRef.current;
      const canvas = canvasRef.current;

      if (area !== null && canvas !== null) {
        onMeasure({ canvas: { width: canvas.clientWidth, height: canvas.clientHeight }, areaWidth: area.clientWidth });
      }
    };

    const observer = new ResizeObserver(measure);

    for (const element of [areaRef.current, canvasRef.current]) {
      if (element !== null) {
        observer.observe(element);
      }
    }

    measure();

    return () => {
      observer.disconnect();
    };
  }, [onMeasure]);

  return { areaRef, canvasRef };
}
