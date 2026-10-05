/// <reference types="vite/client" />
import { describe, expect, test } from "vitest";

import { AUSPICE_ENTRIES } from "../../../build/auspice";

const SOURCES: Record<string, string> = import.meta.glob(["../**/*.{ts,tsx}", "!**/*.d.ts"], {
  query: "?raw",
  import: "default",
  eager: true,
});

const DECLARATIONS = Object.keys(import.meta.glob("../types/**/*.d.ts"));

const AUSPICE_IMPORT = /from "(auspice\/src\/[^"]+)"/gu;

const TYPE_ONLY_MODULES = new Set(["auspice/src/state"]);

describe("auspice module list", () => {
  test("the pre-bundled entries are the auspice modules that the app imports", () => {
    expect(importedModules()).toStrictEqual(AUSPICE_ENTRIES.toSorted());
  });

  test("each pre-bundled entry has a type declaration, and each declaration an entry", () => {
    const declared = DECLARATIONS.map(declaredModule).filter((name) => !TYPE_ONLY_MODULES.has(name));

    expect(declared.toSorted()).toStrictEqual(AUSPICE_ENTRIES.toSorted());
  });
});

function importedModules(): string[] {
  const names = new Set<string>();

  for (const [path, source] of Object.entries(SOURCES)) {
    if (path.includes("/__tests__/")) {
      continue;
    }

    for (const [, name] of source.matchAll(AUSPICE_IMPORT)) {
      if (name !== undefined && !name.endsWith(".json")) {
        names.add(name);
      }
    }
  }

  return [...names].toSorted();
}

function declaredModule(path: string): string {
  return path
    .replace("../types/", "auspice/src/")
    .replace(/\.d\.ts$/u, "")
    .replace(/\/index$/u, "");
}
