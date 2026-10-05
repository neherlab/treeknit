import { Duration } from "luxon";

export function formatElapsed(milliseconds: number): string {
  return Duration.fromMillis(Math.max(0, milliseconds)).toFormat("m:ss");
}
