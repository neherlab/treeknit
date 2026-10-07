import { type ComponentType, lazy, type LazyExoticComponent, useCallback, useSyncExternalStore } from "react";
import { getErrorMessage } from "react-error-boundary";

type CanvasModule<P> = Promise<{ default: ComponentType<P> }>;

export class DrawingCodeError extends Error {
  constructor(cause: unknown) {
    super(`The drawing code could not be downloaded: ${getErrorMessage(cause) ?? String(cause)}`, {
      cause,
    });
  }
}

export interface LazyCanvas<P extends object> {
  current(): LazyExoticComponent<ComponentType<P>>;
  subscribe(listener: () => void): () => void;
  reload(): void;
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
  const listeners = new Set<() => void>();
  const component = { current: lazy(loader) };

  return {
    current: () => component.current,
    subscribe(listener) {
      listeners.add(listener);

      return () => {
        listeners.delete(listener);
      };
    },
    reload() {
      component.current = lazy(loader);

      for (const listener of listeners) {
        listener();
      }
    },
  };
}

export function useLazyCanvas<P extends object>(
  canvas: LazyCanvas<P>,
): readonly [LazyExoticComponent<ComponentType<P>>, () => void] {
  const subscribe = useCallback((listener: () => void) => canvas.subscribe(listener), [canvas]);
  const current = useCallback(() => canvas.current(), [canvas]);

  const reload = useCallback(() => {
    canvas.reload();
  }, [canvas]);

  return [useSyncExternalStore(subscribe, current), reload];
}
