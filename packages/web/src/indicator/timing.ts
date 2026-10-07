import type { useSpinDelay } from "spin-delay";

export const INDICATOR_SPIN_DELAY: Parameters<typeof useSpinDelay>[1] = { delay: 300, minDuration: 500, ssr: false };
