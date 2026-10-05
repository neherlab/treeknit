import { Duration } from "luxon";

export function formatElapsed(milliseconds: number): string {
  const span = Number.isFinite(milliseconds) && milliseconds > 0 ? milliseconds : 0;

  return Duration.fromMillis(span).toFormat("m:ss");
}
