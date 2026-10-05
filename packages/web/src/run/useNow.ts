import { useEffect, useState } from "react";

import { monotonicClock } from "./clock";

const TICK_MS = 1000;

export function useNow(ticking: boolean): number {
  const [now, setNow] = useState(monotonicClock);

  useEffect(() => {
    if (!ticking) {
      return undefined;
    }

    const timer = setInterval(() => {
      setNow(monotonicClock());
    }, TICK_MS);

    return () => {
      clearInterval(timer);
    };
  }, [ticking]);

  return now;
}
