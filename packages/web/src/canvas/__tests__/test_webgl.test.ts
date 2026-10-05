import { describe, expect, test } from "vitest";

import { cachedProbe, type ContextSource, type ProbeContext, supportsWebGl2 } from "../webgl";

function canvasWith(context: ProbeContext | null): () => ContextSource {
  return () => ({ getContext: () => context });
}

describe("supportsWebGl2", () => {
  test("reports support and releases the probe context", () => {
    const lost: string[] = [];

    const context: ProbeContext = {
      getExtension: (name) => ({
        loseContext: () => {
          lost.push(name);
        },
      }),
    };

    expect(supportsWebGl2(canvasWith(context))).toBe(true);
    expect(lost).toStrictEqual(["WEBGL_lose_context"]);
  });

  test("reports no support when the browser returns no context", () => {
    expect(supportsWebGl2(canvasWith(null))).toBe(false);
  });

  test("reports no support when creating the context throws", () => {
    expect(
      supportsWebGl2(() => ({
        getContext: () => {
          throw new Error("WebGL is disabled");
        },
      })),
    ).toBe(false);
  });
});

describe("cachedProbe", () => {
  test("runs the probe once after it reports support", () => {
    const results = [true, false];
    const probe = cachedProbe(() => results.shift() ?? false);

    expect([probe(), probe(), probe()]).toStrictEqual([true, true, true]);
    expect(results).toStrictEqual([false]);
  });

  test("probes again after a probe without support", () => {
    const results = [false, true, false];
    const probe = cachedProbe(() => results.shift() ?? false);

    expect([probe(), probe(), probe()]).toStrictEqual([false, true, true]);
    expect(results).toStrictEqual([false]);
  });
});
