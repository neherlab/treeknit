export type Clock = () => number;

export function monotonicClock(): number {
  return performance.now();
}
