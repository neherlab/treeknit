import { ThemeProvider } from "next-themes";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, test } from "vitest";

import { CLIENT_THEME_PROVIDER_PROPS } from "../../src/shell/theme.ts";
import { contentSecurityPolicy, themeScript } from "../content-security-policy";

describe("contentSecurityPolicy", () => {
  test("the built app allows scripts and WebAssembly of its own origin only", () => {
    expect(contentSecurityPolicy(false, [])).toBe(
      "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; " +
        "img-src 'self' data:; font-src 'self'; connect-src 'self' https:; worker-src 'self'; object-src 'none'; " +
        "base-uri 'none'; form-action 'none'",
    );
  });

  test("the built app allows each inline script by its SHA-256 hash", () => {
    expect(contentSecurityPolicy(false, ["", "abc"])).toContain(
      "script-src 'self' 'sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=' " +
        "'sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=' 'wasm-unsafe-eval';",
    );
  });

  test("the development server allows inline scripts without hashes, which would disable 'unsafe-inline'", () => {
    expect(contentSecurityPolicy(true, ["abc"])).toContain("script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval';");
  });
});

describe("themeScript", () => {
  test("is the next-themes script for the class attribute, the system default, and the color scheme", () => {
    expect(themeScript()).toMatch(/^\(.+\)\("class","theme","system",null,\["light","dark"\],null,true,true\)$/su);
  });
});

describe("the client theme provider", () => {
  test("renders the theme script as an inert data block, because the page head already runs it", () => {
    expect(renderToStaticMarkup(createElement(ThemeProvider, CLIENT_THEME_PROVIDER_PROPS))).toBe(
      `<script type="text/plain">${themeScript()}</script>`,
    );
  });
});
