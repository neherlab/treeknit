# JavaScript advisories: override what runs, ignore what never runs

`packages/web` imports 40 source modules of `auspice` 3.1.0 and the modules they import (`AUSPICE_ENTRIES` in `packages/web/build/auspice.ts`). Installing `auspice` also installs everything else it needs for its own map, spreadsheet export, webpack build, development server, and command lines. `bun audit` reports the advisories of all installed packages, whether the web app runs them or not.

## Decision

- **Override the vulnerable packages that the web app bundles.** The root `package.json` pins `d3-color` 3.1.0 (ReDoS in 1.x), `dompurify` 3.4.16 (XSS and sanitizer bypasses in 2.x), and, for `react-tooltip` only, `uuid` 11.1.1 (buffer bounds check). The bundled consumers use only APIs that these majors keep: the color constructors and interpolators of the d3 1.x modules, `dompurify.addHook` and `dompurify.sanitize` in `auspice/src/util/parseMarkdown.js`, and `v4` of `uuid` in `react-tooltip`
- **Ignore the advisories of packages that nothing runs.** `.config/audit-ignore.json` lists each package with its advisories, the version range they affect, and the reason it never runs. `just audit-ts` passes the advisories to `bun audit --ignore`. The list holds `maplibre-gl` (the map), `xlsx` (the spreadsheet export), `ejs`, `postcss` 7, `postcss-selector-parser`, and `serialize-javascript` (the webpack build), `webpack-dev-middleware` and `uuid` 3 (the development server), and `sprintf-js` (`argparse` 1 of the command lines of Auspice and `js-yaml` 3)
- **The build guards the ignore list.** `auditIgnoredCopiesUnbundled` in `packages/web/build/audit-ignore.ts` fails `vite build` when a bundled module belongs to a listed package whose installed version is inside the affected range. The bundled `uuid` 11.1.1 passes, a bundled `uuid` 7.0.3 would fail
- **No `bun patch`.** Every package that runs has a compatible fixed release. A patch would also leave the advisory in `bun audit`, which compares versions only

## Rationale

- **Overrides fix shipped code.** `d3-color`, `dompurify`, and `uuid` are in the bundle, as the source maps of `just build-web dev` show; the override removes the vulnerable code from what users load
- **Most ignored packages have no compatible fix.** `postcss` 8, `webpack-dev-middleware` 7, and `maplibre-gl` 6 change their APIs, `uuid` 11 drops the `uuid/v4` path that `webpack-log` requires, and npm has no fixed `xlsx` or `sprintf-js` release. An override to an incompatible major would leave code that is still unused and now also broken
- **An ignore list is narrow.** `--ignore` takes advisory IDs, so a new advisory for a listed package fails `audit-ts` again and is reviewed on its own

## Consequences

- An `auspice` upgrade, a new entry in `AUSPICE_ENTRIES`, or a new dependency of the web app can pull a listed package into the bundle; the build then fails and names the copy
- `bun.lock` has `lockfileVersion` 3, which nested overrides need; Bun 1.4.2 of `.config/mise.toml` reads it
- `bun audit` still prints the ignored advisories with `--json`; only the exit code ignores them
- The override versions are exact pins outside the catalog, like `styled-components`; `just deps-upgrade-ts` does not move them
