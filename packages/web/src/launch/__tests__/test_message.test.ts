import { describe, expect, test } from "vitest";

import { type MessagePortal, type ReceivedMessage, READY_MESSAGE, receiveSession } from "../message";

const TIMING = { intervalMs: 5, timeoutMs: 200 };

describe("receiveSession", () => {
  test("announces itself to the sender until the sender replies, and takes the session of the reply", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit", TIMING);

    await pair.waitForReady(2);
    pair.send(pair.sender, { type: "treeknit:open", session: "{}", run: true });

    await expect(received).resolves.toStrictEqual({ text: "{}", origin: "https://sender.example", run: true });

    const announced = pair.posted.length;

    await new Promise((resolve) => setTimeout(resolve, 20));

    expect({ ready: pair.posted.every((message) => message === READY_MESSAGE), stopped: pair.posted.length === announced }).toStrictEqual({
      ready: true,
      stopped: true,
    });
  });

  test("ignores replies from other windows and messages of another shape", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit", TIMING);

    pair.send({ name: "another window" }, { type: "treeknit:open", session: "wrong" });
    pair.send(pair.sender, { type: "something else", session: "wrong" });
    pair.send(pair.sender, { type: "treeknit:open", session: "right" });

    await expect(received).resolves.toStrictEqual({ text: "right", origin: "https://sender.example", run: false });
  });

  test("gives up when the sender does not reply in time", async () => {
    const pair = new WindowPair();

    await expect(receiveSession(pair.portal, "page that opened TreeKnit", TIMING)).rejects.toThrow(
      "The page that opened TreeKnit sent no session file within 0.2 seconds.",
    );
  });

  test("fails at once without a sender window", async () => {
    const portal: MessagePortal = { target: null, listen: () => () => undefined };

    await expect(receiveSession(portal, "page that embeds TreeKnit", TIMING)).rejects.toThrow(
      "There is no page that embeds TreeKnit to receive a session file from.",
    );
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

  async waitForReady(count: number): Promise<void> {
    while (this.posted.length < count) {
      await new Promise((resolve) => setTimeout(resolve, 1));
    }
  }
}
