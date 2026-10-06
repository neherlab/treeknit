import { createHash } from "node:crypto";

import { ThemeProvider } from "next-themes";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import type { Plugin } from "vite";

import { THEME_PROVIDER_PROPS } from "../src/shell/theme.ts";

export function contentSecurityPolicy(devServer: boolean, inlineScripts: readonly string[]): string {
  const scriptSources = devServer
    ? ["'self'", "'unsafe-inline'", "'wasm-unsafe-eval'"]
    : ["'self'", ...inlineScripts.map(scriptHash), "'wasm-unsafe-eval'"];

  return [
    "default-src 'self'",
    `script-src ${scriptSources.join(" ")}`,
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data:",
    "font-src 'self'",
    "connect-src 'self' https:",
    "worker-src 'self'",
    "object-src 'none'",
    "base-uri 'none'",
    "form-action 'none'",
  ].join("; ");
}

export function themeScript(): string {
  const markup = renderToStaticMarkup(createElement(ThemeProvider, THEME_PROVIDER_PROPS));
  const code = /^<script>(?<code>.+)<\/script>$/su.exec(markup)?.groups?.["code"];

  if (code === undefined) {
    throw new Error(`next-themes rendered no inline theme script, but: ${markup}`);
  }

  return code;
}

export function contentSecurityPolicyMeta(): Plugin {
  let devServer = false;

  return {
    name: "treeknit-content-security-policy",
    configResolved(config) {
      devServer = config.command === "serve";
    },
    transformIndexHtml() {
      const theme = themeScript();

      return [
        {
          tag: "meta",
          attrs: { "http-equiv": "Content-Security-Policy", content: contentSecurityPolicy(devServer, [theme]) },
          injectTo: "head-prepend",
        },
        { tag: "script", children: theme, injectTo: "head" },
      ];
    },
  };
}

function scriptHash(code: string): string {
  return `'sha256-${createHash("sha256").update(code).digest("base64")}'`;
}
