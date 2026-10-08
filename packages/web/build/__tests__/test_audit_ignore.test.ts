import { describe, expect, test } from "vitest";

import { affectedCopies, packageLocation } from "../audit-ignore";

const STORE = "/repo/node_modules/.bun";

describe("packageLocation", () => {
  test.each([
    { id: "/repo/node_modules/xlsx/xlsx.mjs", name: "xlsx", root: "/repo/node_modules/xlsx" },
    {
      id: `${STORE}/maplibre-gl@4.7.1/node_modules/maplibre-gl/dist/maplibre-gl.js`,
      name: "maplibre-gl",
      root: `${STORE}/maplibre-gl@4.7.1/node_modules/maplibre-gl`,
    },
    {
      id: `${STORE}/@maplibre+maplibre-gl-leaflet@0.1.3/node_modules/@maplibre/maplibre-gl-leaflet/index.js`,
      name: "@maplibre/maplibre-gl-leaflet",
      root: `${STORE}/@maplibre+maplibre-gl-leaflet@0.1.3/node_modules/@maplibre/maplibre-gl-leaflet`,
    },
    { id: "C:\\repo\\node_modules\\uuid\\dist\\v4.js", name: "uuid", root: "C:/repo/node_modules/uuid" },
    { id: "/repo/node_modules/uuid/dist/v4.js?v=1a2b3c", name: "uuid", root: "/repo/node_modules/uuid" },
  ])("finds $name in $id", ({ id, name, root }) => {
    expect(packageLocation(id)).toStrictEqual({ name, root });
  });

  test.each(["/repo/packages/web/src/main.tsx", "/repo/node_modules/@scope", "/repo/node_modules/"])(
    "finds no package in %s",
    (id) => {
      expect(packageLocation(id)).toBeUndefined();
    },
  );
});

describe("affectedCopies", () => {
  const versions = new Map([
    [`${STORE}/uuid@11.1.1/node_modules/uuid`, "11.1.1"],
    [`${STORE}/uuid@7.0.3/node_modules/uuid`, "7.0.3"],
    [`${STORE}/xlsx@0.17.5/node_modules/xlsx`, "0.17.5"],
    [`${STORE}/d3-color@1.4.1/node_modules/d3-color`, "1.4.1"],
  ]);

  const version = (root: string) => versions.get(root) ?? "0.0.0";

  const affected = new Map([
    ["uuid", "<11.1.1"],
    ["xlsx", "<0.20.2"],
  ]);

  test("lists each bundled copy inside an ignored range once, in order", () => {
    const moduleIds = [
      `${STORE}/xlsx@0.17.5/node_modules/xlsx/xlsx.mjs`,
      `${STORE}/uuid@7.0.3/node_modules/uuid/dist/v4.js`,
      `${STORE}/xlsx@0.17.5/node_modules/xlsx/dist/cpexcel.js`,
    ];

    expect(affectedCopies(moduleIds, affected, version)).toStrictEqual(["uuid@7.0.3", "xlsx@0.17.5"]);
  });

  test("accepts a fixed copy of an ignored package and copies of packages it does not list", () => {
    const moduleIds = [
      `${STORE}/uuid@11.1.1/node_modules/uuid/dist/v4.js`,
      `${STORE}/d3-color@1.4.1/node_modules/d3-color/src/color.js`,
      "/repo/packages/web/src/main.tsx",
    ];

    expect(affectedCopies(moduleIds, affected, version)).toStrictEqual([]);
  });
});
