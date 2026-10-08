# Security advisories in the JavaScript dependencies

`just audit-ts` (`bun audit`) fails with 35 advisories: 2 critical, 9 high, 20 moderate, 4 low. The Rust dependencies have none (`just audit-rs`).

## Locations

Most advisories come through `auspice` 3.0.0, a dependency of `packages/web`:

- Libraries that a browser may run: `maplibre-gl` 4.7.1 (critical, XSS in its sanitizer), `dompurify` 2.5.9 (16 advisories, mostly XSS), `d3-color` 1.4.1 (high, ReDoS), `xlsx` 0.17.5 (high, prototype pollution and ReDoS), `uuid` 7.0.3 and 3.4.0 (moderate)
- The webpack build tools of Auspice: `ejs` 2.7.4 (critical, template injection), `serialize-javascript` 2.1.2 (high, code execution), `webpack-dev-middleware` 3.7.3 (high, path traversal), `postcss` 7.0.39 (high), `postcss-selector-parser` 6.1.4 (moderate)

One more comes through the build tools of this project: `sprintf-js` 1.0.3 (moderate, denial of service through unbounded precision specifiers), through `argparse` 1.x of `@svgr/core` (`cosmiconfig`, `js-yaml` 3) and of `auspice`. The advisory covers every release up to 1.1.3, the latest one, so no upgrade fixes it, and `argparse` 1.x accepts only `~1.0.2`

> [!IMPORTANT]
> **Investigation required.** Which of the browser libraries of Auspice end up in the bundle of `just build-web prod` depends on the Auspice build plugin of `packages/web`. Check the bundle for `maplibre-gl`, `dompurify`, `d3-color`, `xlsx`, and `uuid` before rating the severity.

## Fix direction

- `sprintf-js`: update it once a fixed release is at least seven days old, through an `overrides` entry in the root `package.json`, because `argparse` 1.x does not accept a 1.1 release
- For the Auspice dependencies, check whether a newer `auspice` release raises them; otherwise pin fixed versions through `overrides` in the root `package.json` where the major version stays compatible

## Validation

- `./dev/docker/run just audit-ts` passes
- `./dev/docker/run just build-web prod` and `./dev/docker/run just test-ts` pass
