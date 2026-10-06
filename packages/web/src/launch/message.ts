import * as z from "zod";

import type { ReceivedSession } from "./LinkLaunch";

export const READY_MESSAGE = { type: "treeknit:ready", protocol: 1 } as const;

export const MESSAGE_TIMING: MessageTiming = { intervalMs: 250, timeoutMs: 10_000 };

export const TIMER_CLOCK: MessageClock = {
  repeat: (intervalMs, callback) => {
    const timer = setInterval(callback, intervalMs);

    return () => {
      clearInterval(timer);
    };
  },
  once: (delayMs, callback) => {
    const timer = setTimeout(callback, delayMs);

    return () => {
      clearTimeout(timer);
    };
  },
};

const openMessageSchema = z.object({
  type: z.literal("treeknit:open"),
  session: z.string(),
  run: z.boolean().optional(),
});

export async function receiveSession(
  port: MessagePortal,
  description: string,
  timing: MessageTiming = MESSAGE_TIMING,
  clock: MessageClock = TIMER_CLOCK,
): Promise<ReceivedSession> {
  const { target } = port;

  if (target === null) {
    throw new Error(`There is no ${description} to receive a session file from.`);
  }

  const { promise, resolve, reject } = Promise.withResolvers<ReceivedSession>();

  const announce = () => {
    target.postMessage(READY_MESSAGE, "*");
  };

  announce();

  const stopAnnouncing = clock.repeat(timing.intervalMs, announce);

  const stopWaiting = clock.once(timing.timeoutMs, () => {
    reject(new Error(`The ${description} sent no session file within ${String(timing.timeoutMs / 1000)} seconds.`));
  });

  const stopListening = port.listen(({ source, origin, data }) => {
    const message = openMessageSchema.safeParse(data);

    if (source === target && message.success) {
      resolve({ text: message.data.session, origin, run: message.data.run ?? false });
    }
  });

  try {
    return await promise;
  } finally {
    stopAnnouncing();
    stopWaiting();
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

export interface MessageClock {
  repeat: (intervalMs: number, callback: () => void) => () => void;
  once: (delayMs: number, callback: () => void) => () => void;
}
