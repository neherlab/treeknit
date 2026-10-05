# Developer guide

This guide describes how to set up a development environment, build and test TreeKnit, and maintain the project. It assumes basic familiarity with TreeKnit from a user perspective.

The `justfile` is the entry point for every routine task. `just` lists the recipes by group, and each recipe has a one-line description.

## Setup

TreeKnit builds in two ways: in the build container (recommended), or directly on the host. Both run the same `just` recipes with the same tool versions, which `.config/mise.toml` pins and `.config/mise.lock` locks by URL and checksum. The tools include Bun for the TypeScript workspace of the web app; the recipes that need its packages install them from `bun.lock` first.

### Build container (recommended)

Requirements: Docker with buildx, git, and bash (the stock bash 3.2 of macOS works).

```bash
git clone https://github.com/neherlab/treeknit-rs
cd treeknit-rs
./dev/docker/run just check
```

`./dev/docker/run <command>` runs a command in the container, and without a command it opens a shell. The first run builds the image, which takes a while; later runs reuse it until one of its build inputs changes. Notes:

- The checkout is mounted at its host path, and the git metadata is mounted read-only. Commit on the host
- Build output goes to `.build/container/`, the cargo home to `.cache/cargo/`, the Bun package cache to `.cache/bun/`
- Commands run as your user, with all Linux capabilities dropped
- On Apple Silicon and other arm64 hosts the image runs under x86_64 emulation, which is slower

### Host

Requirements: [mise](https://mise.jdx.dev), [rustup](https://rustup.rs), bash 4 or later (on macOS: `brew install bash`), and a C toolchain for the linker driver. On Debian and Ubuntu:

```bash
sudo apt-get install build-essential
mise install
just check
```

Host builds go to `.build/host/`. On Linux x86_64, `.cargo/config.toml` links with mold, which `mise install` provides.

### Machine settings

Optional settings go into the gitignored `.env` in the checkout; `.env.example` lists them:

- `KACHE_STORE`: directory of the [kache](https://github.com/kunobi-ninja/kache) compiler cache stores. Every build and clippy pass then compiles through kache, which shares compiled crates across the worktrees of this project. Workspace crates of incremental builds (`dev`, tests, `release`, clippy) keep their incremental state, so an edit rebuilds as fast as without kache; kache serves their dependencies and every build without incremental state (CI, `dist`, `profiling`, `bench`, cross builds). Coverage compiles without kache. The directory holds one store per kache version, environment (`host` or `docker`), and pass (`build`, `clippy`, or `cross-<target>`)
- `KACHE_MAX_SIZE`: size limit of each kache store, 100 GiB when unset

## Everyday commands

| Task                                         | Command                                          |
| -------------------------------------------- | ------------------------------------------------ |
| List the recipes                             | `just`                                           |
| Fast checks (format, clippy, oxlint, types)  | `just check`                                     |
| Lint fixes and format (stage first)          | `just fix`                                       |
| Build the CLI                                | `just build <mode>` (`just b`)                   |
| Run the CLI                                  | `just run <mode> ha.nwk na.nwk -o tmp/results`   |
| Rust tests, optionally filtered              | `just test-rs [filter]` (`just t`)               |
| Clippy                                       | `just lint-rs` (`just l`)                        |
| Format                                       | `just fmt`                                       |
| Accuracy against the simulated cases         | `just example accuracy [drop]`                   |
| Build the web app                            | `just build-web <dev\|prod>`                     |
| Run the web app                              | `just run-web <dev\|prod>`                       |
| TypeScript types, lints, tests               | `just typecheck`, `just lint-ts`, `just test-ts` |
| WebAssembly build, tests, Clippy             | `just build-wasm <dev\|release\|prod>`, `just test-wasm`, `just lint-wasm` |
| WebAssembly type declarations                | `just gen`                                       |

In the container, prefix each command with `./dev/docker/run`.

`check` runs its checks in parallel through `dev/run-checks`, keeps going past failures, and lists the failed checks at the end. Each check writes its output to `tmp/checks/<check>.log`. Warnings fail the checks, and a missing tool is a failure, not a skipped check.

## Web app

The web app runs the core in the browser. Two packages make it up, in a Bun workspace whose root is the repository:

- `packages/treeknit-wasm`: the Rust crate with the WebAssembly bindings, and the workspace package `@neherlab/treeknit-wasm`, which wasm-bindgen writes into its `pkg/` directory
- `packages/web`: the React app, built with Vite (`packages/web/README.md`)

The root `package.json` declares the workspace, the scripts that the recipes call, and the version of every JavaScript dependency in its catalog; the package manifests refer to the catalog with `catalog:`. The shared configuration lives at the root as well: `tsconfig.base.json` (strict TypeScript, which the package configurations extend), `oxlint.config.ts`, `oxfmt.config.ts`, `vitest.config.ts`, `bunfig.toml`, and `.config/knip.json`.

Two recipes build and run the web app, each in two modes:

|       | dev                                                                  | prod                                                       |
| ----- | -------------------------------------------------------------------- | ---------------------------------------------------------- |
| build | `just build-web dev`: unminified, with source maps                   | `just build-web prod`: minified, as shipped                |
| run   | `just run-web dev`: Vite dev server with hot reload                  | `just run-web prod`: builds as shipped and serves the build |

The dev mode uses the WebAssembly module of the `release` profile, which is optimized and rebuilds incrementally; after a Rust change, `just build-wasm release` rebuilds it and the dev server reloads it. The prod mode uses the `dist` profile with `wasm-opt`. The build goes to `packages/web/dist/`.

The commands are the same in the main checkout and in a worktree. `run-web` serves on the port of the checkout, which `dev/web-port` derives: 6180 in the main checkout, and a port from the checkout path in a worktree, so the servers of parallel worktrees do not collide. `TREEKNIT_WEB_PORT` overrides it. The recipe prints the URL; open it on `localhost`. In the build container, prefix the commands with `./dev/docker/run`; the container shares the host network, so the browser on the host reaches the server.

The types that cross between Rust and TypeScript are Rust types. tsify writes their TypeScript declarations, which are committed as `packages/treeknit-wasm/pkg/treeknit_wasm.d.ts`, so the TypeScript checks need no Rust build. After a change to the interface, `just gen` rewrites them; `just generated-check`, part of the full gate, fails when they are stale.

## Lints

The workspace `Cargo.toml` enables every Clippy lint group, `restriction` included, and allows the lints that do not fit the project, each under a short heading. New Clippy releases therefore bring their new lints automatically. `clippy.toml` sets the thresholds and bans types and functions that make runs non-reproducible, such as `HashMap` iteration order and unseeded random number generators.

TypeScript is linted by oxlint with type information (`oxlint.config.ts`): the correctness, suspicious, and performance categories, a selection of stricter rules, the sonarjs, Tailwind CSS, and React effect plugins, the vendored anti-slop plugin (`dev/lints/oxlint-anti-slop/UPSTREAM.md`), and the project rules in `dev/lints/oxlint/`, each with its tests. Warnings fail the lint. knip reports unused files, exports, and dependencies.

`just review-suppressions` prints every `#[allow]`, allowed lint, ignored test, and test tolerance, for review apart from ordinary code changes.

## Build modes

The build and run recipes take the mode as their first argument: `just build <mode>`, `just run <mode> [CLI args]`, `just build-wasm <mode>`, `just build-web <dev|prod>`, `just run-web <dev|prod>`, `just build-cross <dev|release|prod>`, `just run-cross <dev|release|prod> <target> [CLI args]`, `just test-distros <dev|release|prod> <target>`. The commands are the same in the main checkout and in a worktree. Each mode is a cargo profile:

- `dev` (the tests): unoptimized workspace crates, dependencies at `opt-level = 2`, full debug info
- `dev-opt`: `dev` with optimized workspace crates, for long runs on real datasets; rebuilds are slower
- `release` (also `just example`): optimized and fast to rebuild, without LTO, with integer overflow checks
- `prod`: the shipped build, the cargo profile `dist`, with fat LTO, one codegen unit, and the CPU flags of `dev/lib/dist-flags.sh` (`-C target-cpu=haswell` on x86-64); the WebAssembly build adds `wasm-opt`
- `profiling` (`just build` only): `prod` with full debug info
- `bench` (`just bench`, no mode): the `prod` settings

`just build release` and `just build prod` copy the binary to `.out/treeknit`; use it with `hyperfine`, which the container provides.

## Cross-compilation

`just build-cross <mode>` (on the host, needs Docker) builds the CLI in the given mode for every release target in `dev/cross/targets`, each in its own cross image, in parallel, into `.out/treeknit-<target>` (`.exe` on Windows), and checks the libraries each binary needs. `just build-cross prod` builds the shipped binaries; `release` builds faster, without LTO. Each target writes its log to `tmp/cross/<target>.log`. Options:

- `--target=<triple>`: build this target only; repeat for several
- `--run`: also run each binary on a simulated case from `fixtures/sim/`, under QEMU or Wine where needed, with output in `tmp/cross/run-<target>/`
- `--serial`: build one target after another, with the output streamed

`just run-cross <mode> <target> [CLI args]` builds the CLI for one target and runs it in the cross image, for example `just run-cross release aarch64-unknown-linux-gnu --help`.

Without `just` on the host, run `./dev/cross/all --profile=dist treeknit`. The scripts take the cargo profile (`dist` for `prod`). The steps of one target run in its cross image, which `CROSS_COMPILE` selects:

```bash
CROSS_COMPILE=aarch64-apple-darwin ./dev/docker/run dev/cross/build --profile=dist treeknit
CROSS_COMPILE=aarch64-apple-darwin ./dev/docker/run dev/cross/check treeknit
```

`dev/cross/build` builds into `.build/cross/<target>/` and copies the binary to `.out/`. `dev/cross/check` prints the libraries the binary needs and the oldest glibc or macOS it supports, and fails when a user system lacks one of the libraries or when an arm64 macOS binary has no code signature. `dev/cross/run` starts the binary: with QEMU for Linux aarch64, with Wine for Windows, and directly for Linux x86_64; macOS binaries cannot run in the images.

| Target                                                    | Toolchain                                  | Binary                                                                                  |
| --------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------- |
| `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`   | GCC 14 (crosstool-ng) with glibc 2.17      | needs glibc 2.17 or newer                                                               |
| `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` | GCC 14 (crosstool-ng) with musl            | static, runs on any Linux                                                               |
| `x86_64-pc-windows-gnu`                                   | GCC 14 (MinGW-w64)                         | links the GCC runtime statically; needs the Universal C Runtime of Windows 10 and newer |
| `x86_64-apple-darwin`, `aarch64-apple-darwin`             | osxcross with the macOS 11.1 SDK and clang | macOS 10.12 or newer on x86_64, 11.0 or newer on aarch64                                |

The `prod` binaries carry the CPU flags of `dev/lib/dist-flags.sh`: they need an x86_64 CPU with AVX2 (Haswell or newer), and on Linux aarch64 the ARMv8.2-A extensions.

The macOS binaries carry the ad-hoc signature of the linker and no Developer ID signature or notarization: macOS (Gatekeeper) blocks a downloaded binary until the user allows it, or until `xattr -d com.apple.quarantine <file>` removes the quarantine attribute.

The cross images are built from `dev/docker/cross-linux.dockerfile` and `dev/docker/cross-darwin.dockerfile`, with the build arguments that `dev/docker/run` sets for each target. They share the base image of the development image, and the toolchains come from the release archives that the scripts in `dev/docker/files/` download and verify.

### Linux distribution tests

`just test-distros <mode> <target>` (on the host, needs Docker) builds the CLI for a Linux x86_64 target and runs it in many Linux distribution images at once, on a simulated case from `fixtures/sim/` with a fixed seed. Every image must finish and write the same `MCCs.json`: the glibc binary takes libm from the image, so a libm that rounds differently would change the result. The target selects the images:

- `x86_64-unknown-linux-gnu`: the images of `dev/cross/distros-glibc`, which have glibc 2.17 or newer: Debian 8 and newer, Ubuntu 14.04 and newer, Amazon Linux, CentOS 7 and 8, Fedora, Oracle Linux, Red Hat UBI, openSUSE, and Arch Linux
- `x86_64-unknown-linux-musl`: the images of `dev/cross/distros-other`, which the glibc binary cannot run on: images without a C library (an empty image, distroless), with musl (Alpine, Void, Chimera, Gentoo, OpenWrt), with BusyBox on glibc, musl, or uClibc, and with a glibc older than 2.17 (Debian 4 to 7, CentOS 5 and 6, Ubuntu 12.04)

Options: `--image=<ref>` runs in one image instead of the list (repeat for several), and arguments after `--` replace the simulated case, for example `just test-distros release x86_64-unknown-linux-musl --image=centos:5 -- --help`; then only the exit status counts. Each container runs as the current user, without network access or capabilities, and sees the checkout read-only; the output of each image goes to `tmp/cross/distros/<target>/<image>/`, its log next to it. To test a binary that is already in `.out/`, run `./dev/cross/test-distros treeknit <target>`.

A run pulls every image of its list, most of them from Docker Hub, which limits the pulls of an IP address without login, and of an account; log in with `docker login` before repeated runs.

## Reports

These recipes produce reports and are never a gate:

- `just coverage`: line coverage of the Rust tests (cargo-llvm-cov), in `tmp/coverage/rs/html/index.html`, and of the TypeScript tests (vitest), in `tmp/coverage/ts/index.html`
- `just mutants`: mutation testing (cargo-mutants) of the code changed since the fork point of the branch, or since `main`; settings in `.cargo/mutants.toml`

## Dependencies

Every dependency release must be at least seven days old before the project adopts it:

- Cargo has the age check as an unstable feature until Rust 1.100. `just deps-update` and `just deps-upgrade` enable it for their resolution, and `just deps-age` fails on a lockfile entry younger than seven days
- Tools in `.config/mise.toml`: `minimum_release_age` makes `just tools-outdated` list only newer releases that are at least seven days old. mise does not filter an exact version, so check the date of a version you type by hand
- npm packages: `minimumReleaseAge` in `bunfig.toml` makes Bun refuse a release younger than seven days, and `exact` keeps the versions exact. `just deps-upgrade-ts <package>...` upgrades named packages in the catalog
- The Dockerfiles in `dev/docker/` share one base image, which follows the same rule by hand
- The toolchain archives of the cross images (`dev/docker/files/install-*`): each script names one release, and `dev/docker/files/fetch` verifies the download by its line in `dev/docker/files/checksums`. Check the release date before changing a release
- mise itself: the image installs the version in `dev/docker/files/mise-version`, verified by its line in `dev/docker/files/checksums`. `min_version` in `.config/mise.toml` is the oldest mise that reads the configuration; raise it when the configuration needs a newer mise
- After changing a tool in `.config/mise.toml`, `just tools-lock <tool>` locks only that tool. Locking calls the GitHub API, which allows 60 anonymous requests per hour. With `MISE_GITHUB_TOKEN` set, mise authenticates instead, on the host and in the container alike (`dev/docker/run` forwards it)

Rust dependencies are pinned exactly in the workspace `Cargo.toml`, JavaScript dependencies in the catalog of the root `package.json`. Upgrade the `wasm-bindgen` crate and the `wasm-bindgen` tool in `.config/mise.toml` together: the tool supports only the crate version it was released with. `.config/deny.toml` sets the license, source, and duplicate-version policy that `just deny` checks offline; `just audit` checks both dependency graphs against the security advisory databases.

The dependency recipes run in the main checkout only.

## Continuous integration

`.github/workflows/ci.yml` runs on pull requests and on pushes to `main`: the check groups `format`, `clippy`, `tests`, and `typescript` in parallel jobs, each in the build container.

`.github/workflows/cli-build.yml` runs on the same events and builds the shipped CLI for every release target, one job per target in its cross image: it builds, checks the libraries the binary needs, runs it on a simulated case (except on macOS), and uploads it as the artifact `treeknit-<target>`. On pushes to `main` and manual runs, two more jobs download the Linux x86_64 binaries and run `dev/cross/test-distros` on them, logged in to Docker Hub when the secrets are available; pull requests skip them to stay within the Docker Hub pull limits.

The CI jobs pull the container image from Docker Hub by the hash of its build inputs, and build it when the inputs changed. Only pushes to `main` publish images, and only when the Docker Hub secrets are available to the repository.
