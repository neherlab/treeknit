import type { TransferHandler } from "comlink";
import * as z from "zod";

export const NO_PANIC_TEXT_MESSAGE =
  "The analysis stopped without an error message. It most likely ran out of memory, because the browser limits the memory of a web page. Try smaller trees, or the command-line tool.";

const thrownTrap = z.object({ value: z.instanceof(WebAssembly.RuntimeError) });

export class PendingPanicText {
  #text: string | undefined;

  record(text: string): void {
    this.#text = text;
  }

  take(): string | undefined {
    const text = this.#text;
    this.#text = undefined;

    return text;
  }
}

export function withPanicText(trap: WebAssembly.RuntimeError, pending: string | undefined): WebAssembly.RuntimeError {
  const message = pending ?? `${NO_PANIC_TEXT_MESSAGE} (${trap.message})`;

  return Object.assign(new WebAssembly.RuntimeError(message), { cause: trap });
}

export function panicTextHandler(
  original: TransferHandler<unknown, unknown>,
  pending: PendingPanicText,
): TransferHandler<unknown, unknown> {
  return {
    canHandle: (value): value is unknown => original.canHandle(value),
    serialize: (thrown) => {
      const text = pending.take();
      const trap = thrownTrap.safeParse(thrown);

      return original.serialize(trap.success ? { value: withPanicText(trap.data.value, text) } : thrown);
    },
    deserialize: (serialized) => original.deserialize(serialized),
  };
}
