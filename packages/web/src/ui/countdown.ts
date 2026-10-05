import { useEffect, useState } from "react";

import { type Clock, monotonicClock } from "../run/clock";

const TICK_MS = 200;

export function remainingSeconds(timeoutMs: number, elapsedMs: number): number {
  return Math.max(0, Math.ceil((timeoutMs - elapsedMs) / 1000));
}

export function useCountdown(timeoutMs: number, paused: boolean, clock: Clock = monotonicClock): number {
  const [elapsedMs, setElapsedMs] = useState(0);

  useEffect(() => {
    if (paused) {
      return undefined;
    }

    let last = clock();

    const advance = () => {
      const now = clock();
      const step = now - last;

      last = now;
      setElapsedMs((elapsed) => elapsed + step);
    };

    const timer = setInterval(advance, TICK_MS);

    return () => {
      clearInterval(timer);
      advance();
    };
  }, [paused, clock]);

  return remainingSeconds(timeoutMs, elapsedMs);
}
