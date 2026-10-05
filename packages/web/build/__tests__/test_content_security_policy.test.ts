import { describe, expect, test } from "vitest";

import { contentSecurityPolicy } from "../content-security-policy";

describe("contentSecurityPolicy", () => {
  test("the built app allows scripts and WebAssembly of its own origin only", () => {
    expect(contentSecurityPolicy(false)).toBe(
      "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; " +
        "img-src 'self' data:; font-src 'self'; connect-src 'self'; worker-src 'self'; object-src 'none'; " +
        "base-uri 'none'; form-action 'none'",
    );
  });

  test("the development server also allows the inline script of hot reloading", () => {
    expect(contentSecurityPolicy(true)).toContain("script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval';");
  });
});
