import { useEffect, useState } from "react";

import { useMediaQueries } from "../ui/useMediaQueries";

export const FADE_IN_MS = 300;

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

const REDUCED_MOTION_QUERIES = [REDUCED_MOTION_QUERY];

export function fadeInOpacity(elapsedMs: number, durationMs = FADE_IN_MS): number {
  if (durationMs <= 0) {
    return 1;
  }

  const t = Math.min(Math.max(elapsedMs / durationMs, 0), 1);

  return 1 - (1 - t) ** 2;
}

function reducedMotion(): boolean {
  return matchMedia(REDUCED_MOTION_QUERY).matches;
}

function reducedMotionOnServer(): boolean {
  return true;
}

export interface Fade {
  key: string;
  opacity: number;
}

export function shownOpacity(fade: Fade, resultKey: string, reduced: boolean): number {
  if (reduced) {
    return 1;
  }

  return fade.key === resultKey ? fade.opacity : 0;
}

export function fadeDone(fade: Fade, resultKey: string): boolean {
  return fade.key === resultKey && fade.opacity >= 1;
}

export function reducedFade(fade: Fade, resultKey: string, reduced: boolean): Fade | undefined {
  return reduced && !fadeDone(fade, resultKey) ? { key: resultKey, opacity: 1 } : undefined;
}

export function useFadeIn(resultKey: string): number {
  const reduced = useMediaQueries(REDUCED_MOTION_QUERIES, reducedMotion, reducedMotionOnServer);
  const [fade, setFade] = useState<Fade>({ key: resultKey, opacity: reduced ? 1 : 0 });
  const settled = reducedFade(fade, resultKey, reduced);
  const done = fadeDone(fade, resultKey);

  if (settled !== undefined) {
    setFade(settled);
  }

  useEffect(() => {
    if (reduced || done) {
      return undefined;
    }

    const start = performance.now();

    let frame = requestAnimationFrame(function step(now) {
      const opacity = fadeInOpacity(now - start);

      setFade({ key: resultKey, opacity });

      if (opacity < 1) {
        frame = requestAnimationFrame(step);
      }
    });

    return () => {
      cancelAnimationFrame(frame);
    };
  }, [resultKey, reduced, done]);

  return shownOpacity(fade, resultKey, reduced);
}
