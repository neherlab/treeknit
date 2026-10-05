# Developer guide

This guide describes how to set up a development environment, build and test TreeKnit, and maintain the project. It assumes basic familiarity with TreeKnit from a user perspective.

The `justfile` is the entry point for every routine task. `just` lists the recipes by group, and each recipe has a one-line description.

## Setup

TreeKnit builds in two ways: in the build container (recommended), or directly on the host. Both run the same `just` recipes with the same tool versions, which `.config/mise.toml` pins and `.config/mise.lock` locks by URL and checksum.

### Build container (recommended)

Requirements: Docker with buildx, git, and bash (the stock bash 3.2 of macOS works).

```bash
git clone https://github.com/neherlab/treeknit-rs
cd treeknit-rs
./dev/docker/run just check
```

`./dev/docker/run <command>` runs a command in the container, and without a command it opens a shell. The first run builds the image, which takes a while; later runs reuse it until one of its build inputs changes. Notes:

- The checkout is mounted at its host path, and the git metadata is mounted read-only. Commit on the host
- Build output goes to `.build/container/`, the cargo home to `.cache/cargo/`
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

- `KACHE_STORE`: directory of the [kache](https://github.com/kunobi-ninja/kache) compiler cache stores. Every build and clippy pass then compiles through kache, which shares compiled crates across the worktrees of this project. Workspace crates of incremental builds (`dev`, tests, `release`, clippy) keep their incremental state, so an edit rebuilds as fast as without kache; kache serves their dependencies and every build without incremental state (CI, `dist`, `profiling`, `bench`). Coverage compiles without kache
- `KACHE_MAX_SIZE`: size limit of each kache store, 100 GiB when unset

## Everyday commands

| Task                                   | Command                                    |
| -------------------------------------- | ------------------------------------------ |
| List the recipes                       | `just`                                     |
| Fast checks (format, clippy)           | `just check`                               |
| Every check; must pass before merging  | `just check-all`                           |
| Lint fixes and format (stage first)    | `just fix`                                 |
| Build                                  | `just build` (`just b`)                    |
| Run the CLI                            | `just run ha.nwk na.nwk -o tmp/results`    |
| Tests, optionally filtered             | `just test-rs [filter]` (`just t`)         |
| Clippy                                 | `just lint-rs` (`just l`)                  |
| Format                                 | `just fmt`                                 |
| Accuracy against the simulated cases   | `just example accuracy [drop]`             |

In the container, prefix each command with `./dev/docker/run`.

`check` and `check-all` run their checks in parallel through `dev/run-checks`, keep going past failures, and list the failed checks at the end. Each check writes its output to `tmp/checks/<check>.log`. Warnings fail the checks, and a missing tool is a failure, not a skipped check. The full gate consists of groups, one CI job each; `just check-group <group>` runs one group serially, as its CI job does.

## Lints

The workspace `Cargo.toml` enables every Clippy lint group, `restriction` included, and allows the lints that do not fit the project, each under a short heading. New Clippy releases therefore bring their new lints automatically. `clippy.toml` sets the thresholds and bans types and functions that make runs non-reproducible, such as `HashMap` iteration order and unseeded random number generators.

`just review-suppressions` prints every `#[allow]`, allowed lint, ignored test, and test tolerance, for review apart from ordinary code changes.

## Build profiles

- `dev` (`just build`, `just run`, the tests): unoptimized workspace crates, dependencies at `opt-level = 2`, full debug info
- `dev-opt` (`just run-dev-opt`): `dev` with optimized workspace crates, for long runs on real datasets; rebuilds are slower
- `release` (`just build-release`, `just run-release`, `just example`): optimized and fast to rebuild, without LTO, with integer overflow checks
- `dist` (`just build-dist`): the shipped binary, with fat LTO, one codegen unit, and the CPU flags of `dev/lib/dist-flags.sh` (`-C target-cpu=haswell` on x86-64)
- `profiling` (`just build-profiling`): `dist` with full debug info
- `bench` (`just bench`): the `dist` settings

`just build-dist` copies the binary to `.out/treeknit`; use it with `hyperfine`, which the container provides.

## Reports

These recipes produce reports and are never a gate:

- `just coverage`: line coverage of the tests (cargo-llvm-cov), in `tmp/coverage/rs/html/index.html`
- `just mutants`: mutation testing (cargo-mutants) of the code changed since the fork point of the branch, or since `main`; settings in `.cargo/mutants.toml`

## Dependencies

Every dependency release must be at least seven days old before the project adopts it:

- Cargo has the age check as an unstable feature until Rust 1.100. `just deps-update` and `just deps-upgrade` enable it for their resolution, and `just deps-age` fails on a lockfile entry younger than seven days
- Tools in `.config/mise.toml`: `minimum_release_age` makes `just tools-outdated` list only newer releases that are at least seven days old. mise does not filter an exact version, so check the date of a version you type by hand
- The base image in `dev/docker/native.dockerfile` follows the same rule by hand
- mise itself: the image installs the version in `dev/docker/files/mise-version`, verified by its line in `dev/docker/files/checksums`. `min_version` in `.config/mise.toml` is the oldest mise that reads the configuration; raise it when the configuration needs a newer mise
- After changing a tool in `.config/mise.toml`, `just tools-lock <tool>` locks only that tool. Locking calls the GitHub API, which allows 60 anonymous requests per hour. With `MISE_GITHUB_TOKEN` set, mise authenticates instead, on the host and in the container alike (`dev/docker/run` forwards it)

Rust dependencies are pinned exactly in the workspace `Cargo.toml`. `.config/deny.toml` sets the license, source, and duplicate-version policy that `just deny` checks offline; `just audit` checks the dependencies against the security advisory database.

The dependency recipes run in the main checkout only.

## Continuous integration

`.github/workflows/ci.yml` runs on pull requests and on pushes to `main`: the check groups of `just check-all` (`format`, `clippy`, `tests`) in parallel jobs, each in the build container.

The CI jobs pull the container image from Docker Hub by the hash of its build inputs, and build it when the inputs changed. Only pushes to `main` publish images, and only when the Docker Hub secrets are available to the repository.
