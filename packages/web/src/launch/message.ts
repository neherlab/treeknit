import * as z from "zod";

import type { ReceivedSession } from "./LinkLaunch";

export const READY_MESSAGE = { type: "treeknit:ready", protocol: 1 } as const;

export const MESSAGE_TIMING: MessageTiming = { intervalMs: 250, timeoutMs: 10_000 };

const openMessageSchema = z.object({
  type: z.literal("treeknit:open"),
  session: z.string(),
  run: z.boolean().optional(),
});

export async function receiveSession(port: MessagePortal, description: string): Promise<ReceivedSession> {
  const { target } = port;

  if (target === null) {
    throw new Error(`There is no ${description} to receive a session file from.`);
  }

  const { promise, resolve, reject } = Promise.withResolvers<ReceivedSession>();

  const announce = () => {
    target.postMessage(READY_MESSAGE, "*");
  };

  announce();

  const announcing = setInterval(announce, MESSAGE_TIMING.intervalMs);

  const waiting = setTimeout(() => {
    reject(
      new Error(`The ${description} sent no session file within ${String(MESSAGE_TIMING.timeoutMs / 1000)} seconds.`),
    );
  }, MESSAGE_TIMING.timeoutMs);

  const stopListening = port.listen(({ source, origin, data }) => {
    const message = openMessageSchema.safeParse(data);

    if (source === target && message.success) {
      resolve({ text: message.data.session, origin, run: message.data.run ?? false });
    }
  });

  try {
    return await promise;
  } finally {
    clearInterval(announcing);
    clearTimeout(waiting);
    stopListening();
  }
}

export interface MessagePortal {
  target: MessageTarget | null;
  listen: (listener: (event: ReceivedMessage) => void) => () => void;
}

export interface MessageTarget {
  postMessage(message: typeof READY_MESSAGE, targetOrigin: string): void;
}

export interface ReceivedMessage {
  source: unknown;
  origin: string;
  data: unknown;
}

export interface MessageTiming {
  intervalMs: number;
  timeoutMs: number;
}
