import { defineConfig } from "oxfmt";

export default defineConfig({
  ignorePatterns: [
    "**/*.md",
    "**/*.toml",
    "bun.lock",
    "dev/lints/oxlint-anti-slop",
    "fixtures",
    "kb",
    "packages/treeknit-cli/tests/data",
    "packages/treeknit-wasm/pkg",
    "ref",
  ],
  printWidth: 120,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  quoteProps: "as-needed",
  trailingComma: "all",
  sortImports: true,
  sortPackageJson: true,
  sortTailwindcss: {
    functions: ["cn", "clsx", "cva", "tw"],
  },
});
