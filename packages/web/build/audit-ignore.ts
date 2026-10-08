import { readFileSync } from "node:fs";

import { sort } from "remeda";
import semver from "semver";
import type { Plugin } from "vite";
import * as z from "zod";

const AUDIT_IGNORE = new URL("../../../.config/audit-ignore.json", import.meta.url);

const NODE_MODULES = "/node_modules/";

const zAuditIgnore = z.object({
  packages: z.record(
    z.string(),
    z.object({
      affected: z.string().refine((range) => semver.validRange(range) !== null, "not a semver range"),
      advisories: z.array(z.string()).min(1),
      reason: z.string().min(1),
    }),
  ),
});

const zPackageManifest = z.object({ version: z.string() });

export function affectedCopies(
  moduleIds: readonly string[],
  affected: ReadonlyMap<string, string>,
  version: (root: string) => string,
): string[] {
  const copies = new Set(
    moduleIds.flatMap((id) => {
      const located = packageLocation(id);
      const range = located === undefined ? undefined : affected.get(located.name);

      if (located === undefined || range === undefined) {
        return [];
      }

      const installed = version(located.root);

      return semver.satisfies(installed, range) ? [`${located.name}@${installed}`] : [];
    }),
  );

  return sort([...copies], (left, right) => left.localeCompare(right));
}

export function packageLocation(moduleId: string): PackageLocation | undefined {
  const path = moduleId.replaceAll("\\", "/");
  const start = path.lastIndexOf(NODE_MODULES);

  if (start === -1) {
    return undefined;
  }

  const [first = "", second] = path.slice(start + NODE_MODULES.length).split("/");
  const name = first.startsWith("@") ? (second === undefined ? undefined : `${first}/${second}`) : first;

  if (name === undefined || name === "") {
    return undefined;
  }

  return { name, root: `${path.slice(0, start)}${NODE_MODULES}${name}` };
}

export function auditIgnoredCopiesUnbundled(): Plugin {
  return {
    name: "treeknit-audit-ignored-copies-unbundled",
    apply: "build",
    generateBundle(_options, bundle) {
      const config = zAuditIgnore.parse(JSON.parse(readFileSync(AUDIT_IGNORE, "utf8")));
      const affected = new Map(Object.entries(config.packages).map(([name, entry]) => [name, entry.affected]));
      const moduleIds = Object.values(bundle).flatMap((file) => (file.type === "chunk" ? file.moduleIds : []));
      const copies = affectedCopies(moduleIds, affected, installedVersion);

      if (copies.length > 0) {
        this.error(
          `The bundle contains ${copies.join(", ")}, inside the affected range of advisories that .config/audit-ignore.json ignores because the web app does not ship them. Fix the advisories and remove the entries.`,
        );
      }
    },
  };
}

export interface PackageLocation {
  name: string;
  root: string;
}

function installedVersion(root: string): string {
  return zPackageManifest.parse(JSON.parse(readFileSync(`${root}/package.json`, "utf8"))).version;
}
