import { projectConfig } from "./dev/lints/oxlint/config.ts";
import * as variants from "./packages/treeknit-wasm/pkg/treeknit_variants.ts";

export default projectConfig({
  root: import.meta.dirname,

  ignorePatterns: [
    "dist",
    ".build",
    ".cache",
    "coverage",
    "fixtures",
    "packages/treeknit-wasm/pkg",
    "packages/treeknit-wasm/www",
  ],

  tailwind: { cwd: "packages/web", entryPoint: "src/index.css" },

  webScopes: ["packages/web/src/**"],

  restrictions: {
    jsonParse: {
      property: {
        object: "JSON",
        property: "parse",
        message:
          "Parsed JSON is untyped. Receive typed values from the WebAssembly module, which parses every TreeKnit file in Rust, or parse in an allowed boundary module.",
      },
    },
  },

  restrictedScopes: [{ files: ["packages/web/src/**"], dir: "web", web: true, allow: [] }],

  restrictionAllowances: [],

  contracts: {
    package: "@neherlab/treeknit-wasm",
    enums: Object.values(variants).flatMap((value): (readonly string[])[] => (Array.isArray(value) ? [value] : [])),
    files: ["packages/web/src/**"],
  },

  overrides: [
    {
      files: ["packages/web/src/main.tsx"],
      rules: {
        "import/no-unassigned-import": "off",
      },
    },
  ],
});
