import { type ComponentType, lazy, type LazyExoticComponent, useCallback } from "react";
import { getErrorMessage } from "react-error-boundary";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { ReadableStore } from "../workspace/store";

type CanvasModule<P> = Promise<{ default: ComponentType<P> }>;

export class DrawingCodeError extends Error {
  constructor(cause: unknown) {
    super(`The drawing code could not be downloaded: ${getErrorMessage(cause) ?? String(cause)}`, {
      cause,
    });
  }
}

export interface LazyCanvas<P extends object> {
  readonly state: ReadableStore<LazyCanvasState<P>>;
  reload(): void;
}

export interface LazyCanvasState<P extends object> {
  component: LazyExoticComponent<ComponentType<P>>;
}

export function loadDrawingCode<P>(load: () => CanvasModule<P>): () => CanvasModule<P> {
  return async () => {
    try {
      return await load();
    } catch (cause) {
      throw new DrawingCodeError(cause);
    }
  };
}

export function lazyCanvas<P extends object>(load: () => CanvasModule<P>): LazyCanvas<P> {
  const loader = loadDrawingCode(load);
  const state = createStore<LazyCanvasState<P>>()(() => ({ component: lazy(loader) }));

  return {
    state,
    reload() {
      state.setState({ component: lazy(loader) });
    },
  };
}

export function useLazyCanvas<P extends object>(
  canvas: LazyCanvas<P>,
): readonly [LazyExoticComponent<ComponentType<P>>, () => void] {
  const component = useStore(canvas.state, (state) => state.component);

  const reload = useCallback(() => {
    canvas.reload();
  }, [canvas]);

  return [component, reload];
}
