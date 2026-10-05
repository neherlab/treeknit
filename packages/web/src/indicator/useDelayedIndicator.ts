import { useEffect, useState } from "react";

import { DelayedIndicator } from "./delayedIndicator";

export function useDelayedIndicator(active: boolean): boolean {
  const [visible, setVisible] = useState(false);
  const [indicator] = useState(() => new DelayedIndicator(setVisible));

  useEffect(() => {
    indicator.set(active);
  }, [indicator, active]);

  useEffect(
    () => () => {
      indicator.reset();
    },
    [indicator],
  );

  return visible;
}
