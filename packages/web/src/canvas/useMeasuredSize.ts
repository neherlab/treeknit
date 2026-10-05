import { type RefObject, useLayoutEffect, useRef } from "react";
import { flushSync } from "react-dom";

import type { MeasuredSize } from "./viewState";

export interface MeasuredElements {
  areaRef: RefObject<HTMLDivElement | null>;
  canvasRef: RefObject<HTMLDivElement | null>;
}

export interface MeasureOptions {
  beforePaint?: boolean;
}

export interface SizedElement {
  clientWidth: number;
  clientHeight: number;
}

export interface SizeObserver<E> {
  observe(element: E): void;
  disconnect(): void;
}

export function useMeasuredSize(
  onMeasure: (measured: MeasuredSize) => void,
  { beforePaint = false }: MeasureOptions = {},
): MeasuredElements {
  const areaRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(
    () =>
      watchSize(
        areaRef.current,
        canvasRef.current,
        onMeasure,
        (measure) => new ResizeObserver(resizeHandler(measure, beforePaint, flushBeforePaint)),
      ),
    [onMeasure, beforePaint],
  );

  return { areaRef, canvasRef };
}

export function resizeHandler(
  measure: () => void,
  beforePaint: boolean,
  flush: (update: () => void) => void,
): () => void {
  return beforePaint
    ? () => {
        flush(measure);
      }
    : measure;
}

export function watchSize<E extends SizedElement>(
  area: E | null,
  canvas: E | null,
  onMeasure: (measured: MeasuredSize) => void,
  createObserver: (measure: () => void) => SizeObserver<E>,
): () => void {
  const measure = () => {
    if (area !== null && canvas !== null) {
      onMeasure({ canvas: { width: canvas.clientWidth, height: canvas.clientHeight }, areaWidth: area.clientWidth });
    }
  };

  const observer = createObserver(measure);

  for (const element of [area, canvas]) {
    if (element !== null) {
      observer.observe(element);
    }
  }

  measure();

  return () => {
    observer.disconnect();
  };
}

function flushBeforePaint(update: () => void): void {
  flushSync(update);
}
