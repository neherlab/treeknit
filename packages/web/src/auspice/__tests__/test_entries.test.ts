/// <reference types="vite/client" />
import { describe, expect, test } from "vitest";

import { AUSPICE_ENTRIES } from "../../../build/auspice";

const APP_SOURCES: Record<string, string> = import.meta.glob(["../**/*.{ts,tsx}", "!**/*.d.ts", "!**/__tests__/**"], {
  query: "?raw",
  import: "default",
  eager: true,
});

const TEST_SOURCES: Record<string, string> = import.meta.glob("./*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
});

const DECLARATIONS = Object.keys(import.meta.glob("../types/**/*.d.ts"));

const AUSPICE_IMPORT = /from "(auspice\/src\/[^"]+)"/gu;

const TYPE_ONLY_MODULES = new Set(["auspice/src/state"]);

describe("auspice module list", () => {
  test("the pre-bundled entries are the auspice modules that the app imports", () => {
    expect(importedModules(APP_SOURCES)).toStrictEqual(AUSPICE_ENTRIES.toSorted());
  });

  test("the auspice modules that the app or its tests import have a type declaration, and no other module", () => {
    const declared = DECLARATIONS.map(declaredModule).filter((name) => !TYPE_ONLY_MODULES.has(name));

    const imported = [...new Set([...AUSPICE_ENTRIES, ...importedModules(TEST_SOURCES)])].filter(
      (name) => !TYPE_ONLY_MODULES.has(name),
    );

    expect(declared.toSorted()).toStrictEqual(imported.toSorted());
  });
});

function importedModules(sources: Record<string, string>): string[] {
  const names = new Set<string>();

  for (const source of Object.values(sources)) {
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
