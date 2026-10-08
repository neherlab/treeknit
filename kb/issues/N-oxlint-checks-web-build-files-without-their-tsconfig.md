# Type-aware oxlint checks the web build files without their tsconfig

`packages/web/tsconfig.node.json` types `packages/web/vite.config.ts` and `packages/web/build/`, with `lib` ES2024, and `just typecheck` checks them with it (`tsc -p tsconfig.node.json`). Type-aware oxlint does not use that file: it finds `packages/web/tsconfig.json`, whose `include` holds only `src`, and checks the build files in a project without the ES2023 library. There, `Array#toSorted` has an error type, so `typescript/no-unsafe-call` and `typescript/no-unsafe-return` report a correct call, while `unicorn/no-array-sort` reports the `Array#sort` that would avoid it.

## Locations

- `packages/web/tsconfig.json`, `packages/web/tsconfig.node.json`
- `packages/web/build/audit-ignore.ts` sorts with `sort` of `remeda`, which does not depend on the library setting

A reference from `packages/web/tsconfig.json` to `tsconfig.node.json` makes oxlint find the build project, but `tsc` then requires `composite` and rejects `noEmit` in the referenced project (TS6306, TS6310).

## Fix direction

- Move `packages/web` to a solution-style `tsconfig.json` with references to an app project and a node project, checked with `tsc -b`, as the Vite templates do; or check whether oxlint accepts the path of the tsconfig for a directory

## Validation

- A call of `Array#toSorted` in `packages/web/build/` passes `./dev/docker/run just lint-ts`
- `./dev/docker/run just typecheck` passes
