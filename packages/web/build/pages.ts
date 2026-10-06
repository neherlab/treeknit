import type { Plugin } from "vite";

import { PAGES } from "../src/pages.ts";

const INDEX_HTML = "index.html";

export function pageCopies(bundle: Readonly<Record<string, BundleFile>>, paths: readonly string[]): PageCopy[] {
  const index = bundle[INDEX_HTML];

  if (index?.type !== "asset" || index.source === undefined) {
    throw new Error(`The build wrote no ${INDEX_HTML} to copy for the other pages.`);
  }

  const { source } = index;

  return paths.flatMap((path) => (path === "/" ? [] : [{ fileName: `${path.slice(1)}.html`, source }]));
}

export function pageFiles(): Plugin {
  return {
    name: "treeknit-page-files",
    apply: "build",
    enforce: "post",
    generateBundle(_options, bundle) {
      for (const copy of pageCopies(bundle, Object.values(PAGES))) {
        this.emitFile({ type: "asset", ...copy });
      }
    },
  };
}

export interface BundleFile {
  type: string;
  source?: string | Uint8Array;
}

export interface PageCopy {
  fileName: string;
  source: string | Uint8Array;
}
