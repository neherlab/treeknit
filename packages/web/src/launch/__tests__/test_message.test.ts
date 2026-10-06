import { describe, expect, test } from "vitest";

import {
  type MessageClock,
  type MessagePortal,
  MESSAGE_TIMING,
  type ReceivedMessage,
  READY_MESSAGE,
  receiveSession,
} from "../message";

describe("receiveSession", () => {
  test("announces itself to the sender until the sender replies, and takes the session of the reply", async () => {
    const pair = new WindowPair();
    const clock = new ManualClock();
    const received = receiveSession(pair.portal, "page that opened TreeKnit", MESSAGE_TIMING, clock);

    clock.tick();
    clock.tick();
    pair.send(pair.sender, { type: "treeknit:open", session: "{}", run: true });

    await expect(received).resolves.toStrictEqual({ text: "{}", origin: "https://sender.example", run: true });

    clock.tick();

    expect({ posted: pair.posted, timers: clock.active }).toStrictEqual({
      posted: [READY_MESSAGE, READY_MESSAGE, READY_MESSAGE],
      timers: 0,
    });
  });

  test("ignores replies from other windows and messages of another shape", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit", MESSAGE_TIMING, new ManualClock());

    pair.send({ name: "another window" }, { type: "treeknit:open", session: "wrong" });
    pair.send(pair.sender, { type: "something else", session: "wrong" });
    pair.send(pair.sender, { type: "treeknit:open", session: "right" });

    await expect(received).resolves.toStrictEqual({ text: "right", origin: "https://sender.example", run: false });
  });

  test("gives up when the sender does not reply in time", async () => {
    const pair = new WindowPair();
    const clock = new ManualClock();
    const received = receiveSession(pair.portal, "page that opened TreeKnit", MESSAGE_TIMING, clock);

    clock.expire();

    await expect(received).rejects.toThrow("The page that opened TreeKnit sent no session file within 10 seconds.");
  });

  test("fails at once without a sender window", async () => {
    const portal: MessagePortal = { target: null, listen: () => () => undefined };

    await expect(
      receiveSession(portal, "page that embeds TreeKnit", MESSAGE_TIMING, new ManualClock()),
    ).rejects.toThrow("There is no page that embeds TreeKnit to receive a session file from.");
  });
});

class WindowPair {
  readonly posted: unknown[] = [];

  readonly sender = {
    postMessage: (message: unknown) => {
      this.posted.push(message);
    },
  };

  readonly #listeners = new Set<(event: ReceivedMessage) => void>();

  get portal(): MessagePortal {
    return {
      target: this.sender,
      listen: (listener) => {
        this.#listeners.add(listener);

        return () => {
          this.#listeners.delete(listener);
        };
      },
    };
  }

  send(source: unknown, data: unknown): void {
    for (const listener of this.#listeners) {
      listener({ source, origin: "https://sender.example", data });
    }
  }
}

class ManualClock implements MessageClock {
  readonly #repeating = new Set<() => void>();
  readonly #pending = new Set<() => void>();

  get active(): number {
    return this.#repeating.size + this.#pending.size;
  }

  repeat(_intervalMs: number, callback: () => void): () => void {
    this.#repeating.add(callback);

    return () => {
      this.#repeating.delete(callback);
    };
  }

  once(_delayMs: number, callback: () => void): () => void {
    this.#pending.add(callback);

    return () => {
      this.#pending.delete(callback);
    };
  }

  tick(): void {
    for (const callback of this.#repeating) {
      callback();
    }
  }

  expire(): void {
    for (const callback of this.#pending) {
      callback();
    }
  }
}
