import { DateTime } from "luxon";
import { useEffect, useState } from "react";

const TICK_MS = 1000;

export function useNow(ticking: boolean): number {
  const [now, setNow] = useState(() => DateTime.now().toMillis());

  useEffect(() => {
    if (!ticking) {
      return undefined;
    }

    const timer = setInterval(() => {
      setNow(DateTime.now().toMillis());
    }, TICK_MS);

    return () => {
      clearInterval(timer);
    };
  }, [ticking]);

  return now;
}
