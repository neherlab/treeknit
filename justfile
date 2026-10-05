# TreeKnit task runner: `just` lists every recipe by group.
#
# Recipes run the tools pinned in .config/mise.toml, on the host after `mise install` or
# in the build container through `dev/docker/run just <recipe>`. Before a change
# is merged, `just check-all` must pass.
#
# Naming: a leaf recipe runs one tool in one mode. The suffix `-rs`, `-wasm`, or
# `-ts` names the toolchain of a leaf: Rust, Rust for WebAssembly, or TypeScript
# (Bun); tools that serve one toolchain only, such as knip, keep their own name.
# A recipe without a toolchain suffix combines the leaves and calls no tool
# itself. The suffix `-all` adds the slow tools to the fast set of the same
# name: `lint` and `lint-all`, `check` and `check-all`.

set minimum-version := "1.58.0"
set default-list
set dotenv-load
set positional-arguments
set shell := ["mise", "exec", "--", "bash", "-euo", "pipefail", "-c"]
set script-interpreter := ["mise", "exec", "--", "bash", "-euo", "pipefail"]

export CARGO_TERM_QUIET := "true"
export NEXTEST_NO_TESTS := "fail"

# Build output of the checkout. dev/docker/run points TREEKNIT_BUILD_DIR at
# .build/container, so host and container artifacts, which link against
# different system libraries, never mix. Builds and tests share one cargo target
# directory; clippy has its own, so a lint never waits on the build lock.
build_dir := env("TREEKNIT_BUILD_DIR", justfile_directory() / ".build/host")
export CARGO_TARGET_DIR := build_dir / "cargo"
lint_target_dir := build_dir / "lint"

# The kache compiler cache is optional. KACHE_STORE, in .env or the environment,
# names the store directory; every build and clippy pass then compiles through
# kache. KACHE_PRESERVE_INCREMENTAL keeps incremental compiles out of the cache:
# the workspace crates of dev, test, release, and clippy builds keep their
# incremental state, so an edit rebuilds as fast as without kache, while kache
# serves the dependencies and every build without incremental state (CI, dist,
# profiling, and bench builds). Coverage always compiles without kache, because
# a cache hit skips its instrumentation.
#
# Each store is <KACHE_STORE>/<kache version>/<host|docker>-<pass>: kache does
# not check its store format, so two versions never share a store; host and
# container builds use different toolchains; and builds and clippy run different
# compiler drivers. KACHE_MAX_SIZE in .env caps each store, 100 GiB by default.
kache_store := env("KACHE_STORE", "")
kache_version := if kache_store != "" { `kache --version | cut -d ' ' -f 2` } else { "" }
kache_prefix := kache_store / kache_version / if env("TREEKNIT_CONTAINER", "") != "" { "docker" } else { "host" }
export KACHE_MAX_SIZE := env("KACHE_MAX_SIZE", "100GiB")
export KACHE_PRESERVE_INCREMENTAL := "1"
export RUSTC_WRAPPER := if kache_store != "" { "kache" } else { env("RUSTC_WRAPPER", "") }
export KACHE_CACHE_DIR := if kache_store != "" { kache_prefix + "-build" } else { env("KACHE_CACHE_DIR", "") }
lint_env := "CARGO_TARGET_DIR=" + quote(lint_target_dir) + if kache_store != "" { " KACHE_CACHE_DIR=" + quote(kache_prefix + "-clippy") } else { "" }
uncached_env := "RUSTC_WRAPPER= CARGO_INCREMENTAL=0"

# Cargo with the seven-day minimum publish age of new dependency releases. The
# setting is unstable in the pinned Rust (stable from 1.100), so the dependency
# recipes enable it for their resolution only; RUSTC_BOOTSTRAP=1 allows unstable
# Cargo features on a stable toolchain.
cargo_min_age := "RUSTC_BOOTSTRAP=1 cargo -Zmin-publish-age --config 'registry.global-min-publish-age=\"7 days\"'"

# The TypeScript package that wasm-bindgen writes for the WebAssembly module.
# Only its type declarations are committed, so the TypeScript checks read them
# without a Rust build; `just generated-check` keeps them current.
wasm_pkg := "packages/treeknit-wasm/pkg"
wasm_types := wasm_pkg / "treeknit_wasm.d.ts"

# Groups of the full gate, one CI job each (`just check-group <group>`).
checks_format := "fmt-check-rs fmt-check-ts fmt-check-other lint-shell lint-docker lint-workflows deny shear"
checks_clippy := "lint-rs lint-wasm"
checks_tests := "test-rs test-wasm build-web:prod"
checks_typescript := "typecheck lint-ts knip test-ts generated-check"
check_fast := "fmt-check-rs fmt-check-ts fmt-check-other lint-rs lint-wasm lint-ts typecheck"
check_full := checks_format + " " + checks_clippy + " " + checks_tests + " " + checks_typescript

alias b := build
alias br := build-release
alias bd := build-dist
alias bp := build-profiling
alias r := run
alias rr := run-release
alias t := test-rs
alias tu := test-unit-rs
alias ti := test-integration-rs
alias l := lint-rs
alias lf := lint-fix
alias f := fmt
alias fc := fmt-check

# Fast checks: formatting, clippy, TypeScript types and lints, in parallel
[group("check")]
check: _js
    TREEKNIT_JS_READY=1 dev/run-checks {{ check_fast }}

# Every check, in parallel; must pass before a change is merged
[group("check")]
check-all: _js
    TREEKNIT_JS_READY=1 dev/run-checks {{ check_full }}

# One group of check-all, serially with streamed output, as its CI job runs it: just check-group <format|clippy|tests|typescript>
[group("check")]
check-group group: _js
    recipes="$(just --evaluate "checks_$1")"; TREEKNIT_JS_READY=1 dev/run-checks --serial ${recipes}

# Apply the automatic lint fixes (clippy, oxlint), then format; stage your changes first
[group("check")]
fix: lint-fix fmt

# Install the pinned tools (on the host)
[group("setup")]
setup:
    if [[ -z "${TREEKNIT_CONTAINER:-}" ]]; then mise install; fi

# Build the workspace (dev profile)
[group("build")]
build *args:
    cargo build --locked "$@"

# Build the CLI (release profile: optimized, fast to rebuild) and copy it to .out/
[group("build")]
build-release *args:
    cargo build --locked --release --bin treeknit "$@"
    mkdir -p .out && cp {{ quote(CARGO_TARGET_DIR / "release" / "treeknit") }} .out/

# Build the CLI as shipped (dist profile: fat LTO) and copy it to .out/
[group("build")]
build-dist *args:
    source dev/lib/dist-flags.sh && export_dist_flags && cargo build --locked --profile=dist --bin treeknit "$@"
    mkdir -p .out && cp {{ quote(CARGO_TARGET_DIR / "dist" / "treeknit") }} .out/

# Build the CLI (profiling profile: dist with full debug info)
[group("build")]
build-profiling *args:
    source dev/lib/dist-flags.sh && export_dist_flags && cargo build --locked --profile=profiling --bin treeknit "$@"

# Build the WebAssembly package into packages/treeknit-wasm/pkg/: just build-wasm [dev|release|dist]; dist, as shipped, adds wasm-opt
[group("build")]
build-wasm profile="dist":
    cargo build --locked --profile={{ quote(profile) }} --target=wasm32-unknown-unknown -p treeknit-wasm
    wasm-bindgen --target=web --out-dir={{ wasm_pkg }} {{ quote(CARGO_TARGET_DIR / "wasm32-unknown-unknown" / (if profile == "dev" { "debug" } else { profile }) / "treeknit_wasm.wasm") }}
    if [[ {{ quote(profile) }} == dist ]]; then wasm-opt -O {{ wasm_pkg }}/treeknit_wasm_bg.wasm -o {{ wasm_pkg }}/treeknit_wasm_bg.wasm; fi

# Build the web app into packages/web/dist/: just build-web <dev|prod>; dev: WebAssembly of the release profile, unminified with source maps; prod: as shipped, WebAssembly of the dist profile
[arg("mode", pattern="dev|prod")]
[group("app")]
build-web mode: _js (build-wasm (if mode == "dev" { "release" } else { "dist" }))
    bun run --silent build:web --mode {{ if mode == "dev" { "development" } else { "production" } }}

# Run the CLI (dev profile): just run ha.nwk na.nwk -o tmp/results
[group("run")]
run *args:
    args=("$@"); [[ "${args[0]:-}" != "--" ]] || args=("${args[@]:1}"); cargo run --locked --bin treeknit -- ${args[@]+"${args[@]}"}

# Run the CLI (release profile): just run-release ha.nwk na.nwk -o tmp/results
[group("run")]
run-release *args:
    args=("$@"); [[ "${args[0]:-}" != "--" ]] || args=("${args[@]:1}"); cargo run --locked --release --bin treeknit -- ${args[@]+"${args[@]}"}

# Run the CLI (dev-opt profile: dev with optimized workspace crates)
[group("run")]
run-dev-opt *args:
    args=("$@"); [[ "${args[0]:-}" != "--" ]] || args=("${args[@]:1}"); cargo run --locked --profile=dev-opt --bin treeknit -- ${args[@]+"${args[@]}"}

# Run an example (release profile): just example accuracy
[group("run")]
example name *args:
    cargo run --locked --release --example {{ quote(name) }} -- "${@:2}"

# Rust tests (nextest); arguments are nextest filters and options
[group("test")]
test-rs *args:
    cargo nextest run --locked --workspace "$@"

# Rust unit tests only
[group("test")]
test-unit-rs *args:
    cargo nextest run --locked --workspace --lib "$@"

# Rust integration tests only
[group("test")]
test-integration-rs *args:
    cargo nextest run --locked --workspace --test '*' "$@"

# Cargo passes --quiet on to the test binary, which wasm-bindgen-test-runner rejects.
# Tests of the WebAssembly bindings, run in Node
[group("test")]
test-wasm *args:
    CARGO_TERM_QUIET=false cargo test --locked -p treeknit-wasm --target=wasm32-unknown-unknown --test=wasm "$@"

# TypeScript tests (vitest) and the tests of the custom oxlint rules, keep-going
[group("test")]
[script]
test-ts: _js
    status=0
    bun run --silent test || status=1
    node --test "dev/lints/oxlint/__tests__/test_*.ts" "dev/lints/oxlint-anti-slop/**/*.test.ts" || status=1
    exit "${status}"

# List Rust tests without running them
[group("test")]
test-list-rs *args:
    cargo nextest list --locked --workspace "$@"

# Coverage reports of Rust and TypeScript, never a gate
[group("report")]
coverage: coverage-rs coverage-ts

# Rust coverage report (cargo-llvm-cov), never a gate
[group("report")]
coverage-rs:
    {{ uncached_env }} cargo llvm-cov nextest --locked --workspace --html --output-dir tmp/coverage/rs
    printf 'Rust coverage: tmp/coverage/rs/html/index.html\n'

# TypeScript coverage report (vitest), never a gate
[group("report")]
coverage-ts: _js
    bun run --silent coverage
    printf 'TypeScript coverage: tmp/coverage/ts/index.html\n'

# Mutation testing of the code changed since the fork point (or since `main`), never a gate
[group("report")]
mutants *args:
    dev/mutants "$@"

# Inventory of lint suppressions and other review-sensitive settings
[group("report")]
review-suppressions:
    dev/review-suppressions

# Fast lints: clippy and oxlint, keep-going
[group("lint")]
lint: _js
    TREEKNIT_JS_READY=1 dev/run-checks --serial lint-rs lint-wasm lint-ts

# Every lint: clippy, oxlint, TypeScript types, unused code and dependencies, dependency policy, and the shell, Dockerfile, and workflow lints, keep-going
[group("lint")]
lint-all: _js
    TREEKNIT_JS_READY=1 dev/run-checks --serial lint-rs lint-wasm lint-ts typecheck knip deny shear lint-shell lint-docker lint-workflows

# Apply the automatic lint fixes: clippy, then oxlint; stage your changes first
[group("lint")]
lint-fix: lint-fix-rs lint-fix-ts

# Apply clippy's machine-applicable fixes; stage your changes first
[group("lint")]
lint-fix-rs:
    {{ lint_env }} cargo clippy --locked --workspace --all-targets --fix --allow-staged

# Apply oxlint's automatic fixes
[group("lint")]
lint-fix-ts: _js
    bun run --silent lint:fix

# Clippy over all targets; reports the warnings of every crate, then fails if there were any
[group("lint")]
lint-rs *args:
    {{ lint_env }} CARGO_BUILD_WARNINGS=deny cargo clippy --locked --workspace --all-targets --keep-going "$@"

# Clippy over the WebAssembly bindings for the WebAssembly target; fails on warnings
[group("lint")]
lint-wasm *args:
    {{ lint_env }} CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p treeknit-wasm --all-targets --target=wasm32-unknown-unknown --keep-going "$@"

# TypeScript lints (oxlint, type-aware)
[group("lint")]
lint-ts: _js
    bun run --silent lint

# TypeScript type checks of the packages, the tool configs, and the vendored lint rules, keep-going
[group("lint")]
[script]
typecheck: _js
    status=0
    bun run --silent typecheck:packages || status=1
    bun run --silent typecheck:tools || status=1
    bun run --silent typecheck:vendor || status=1
    exit "${status}"

# Unused TypeScript files, exports, and dependencies (knip), keep-going
[group("lint")]
[script]
knip: _js
    status=0
    bun run --silent knip || status=1
    bun run --silent knip:production || status=1
    exit "${status}"

# Dependency bans, licenses, and sources (cargo-deny, offline)
[group("lint")]
deny:
    cargo deny --locked --all-features --config .config/deny.toml check bans licenses sources

# Unused Rust dependencies (cargo-shear)
[group("lint")]
shear:
    cargo shear

# Shell scripts in dev/ (shellcheck)
[group("lint")]
lint-shell:
    dev/shell-files | xargs -0r shellcheck --source-path=SCRIPTDIR

# Dockerfiles (hadolint)
[group("lint")]
lint-docker:
    hadolint --config .config/hadolint.yaml dev/docker/*.dockerfile

# GitHub Actions workflows (actionlint)
[group("lint")]
lint-workflows:
    actionlint

# Format Rust, TypeScript, shell, TOML, and the justfile
[group("format")]
fmt: fmt-rs fmt-ts fmt-other

# Check formatting of Rust, TypeScript, shell, TOML, and the justfile, keep-going
[group("format")]
fmt-check: _js
    TREEKNIT_JS_READY=1 dev/run-checks --serial fmt-check-rs fmt-check-ts fmt-check-other

# Format Rust (rustfmt)
[group("format")]
fmt-rs:
    cargo fmt --all

# Check formatting of Rust (rustfmt)
[group("format")]
fmt-check-rs:
    cargo fmt --all --check

# Format TypeScript, JavaScript, JSON, YAML, CSS, and HTML (oxfmt)
[group("format")]
fmt-ts: _js
    bun run --silent format

# Check formatting of TypeScript, JavaScript, JSON, YAML, CSS, and HTML (oxfmt)
[group("format")]
fmt-check-ts: _js
    bun run --silent format:check

# Format shell scripts (shfmt), TOML (taplo), and the justfile
[group("format")]
fmt-other:
    dev/shell-files | xargs -0r shfmt --write
    dev/toml-files | RUST_LOG=warn xargs -0r taplo fmt
    just --fmt

# Check formatting of shell scripts (shfmt), TOML (taplo), and the justfile, keep-going
[group("format")]
[script]
fmt-check-other:
    status=0
    dev/shell-files | xargs -0r shfmt --diff || status=1
    dev/toml-files | RUST_LOG=warn xargs -0r taplo fmt --check --diff || status=1
    just --fmt --check || status=1
    exit "${status}"

# Regenerate the TypeScript declarations of the WebAssembly package (packages/treeknit-wasm/pkg/treeknit_wasm.d.ts) after a change to its interface
[group("generated")]
gen: (build-wasm "dev")

# Fail when the committed TypeScript declarations of the WebAssembly package differ from a fresh build; writes nothing into the checkout
[group("generated")]
[script]
generated-check:
    out="$(mktemp -d)"
    trap 'rm -rf "${out}"' EXIT
    cargo build --locked --target=wasm32-unknown-unknown -p treeknit-wasm
    wasm-bindgen --target=web --out-dir="${out}" {{ quote(CARGO_TARGET_DIR / "wasm32-unknown-unknown" / "debug" / "treeknit_wasm.wasm") }}
    diff -u {{ wasm_types }} "${out}/treeknit_wasm.d.ts" || { printf '%s is stale; run `just gen` and commit it\n' {{ quote(wasm_types) }} >&2; exit 1; }

# Run the web app in the foreground until Ctrl-C, on the port of this checkout: just run-web <dev|prod>; dev: Vite dev server with hot reload (`just build-wasm release` after a Rust change); prod: the shipped build, served
[arg("mode", pattern="dev|prod")]
[group("app")]
run-web mode: _js (build-wasm (if mode == "dev" { "release" } else { "dist" }))
    if [[ {{ quote(mode) }} == prod ]]; then bun run --silent build:web --mode production; fi
    TREEKNIT_WEB_PORT="$(dev/web-port)" bun run --silent {{ if mode == "dev" { "dev:web" } else { "preview:web" } }}

# Run all benchmarks (bench profile: the shipped dist settings)
[group("bench")]
bench *args:
    source dev/lib/dist-flags.sh && export_dist_flags && cargo bench --locked --workspace --benches "$@"

# Explain why a crate is in the dependency tree: just why <crate> [cargo tree options]
[group("deps")]
why crate *args:
    cargo tree --locked --invert "$@"

# Update Cargo.lock within the manifest ranges to releases at least 7 days old (main checkout only)
[confirm("Update Cargo.lock?")]
[group("deps")]
deps-update *args: _main-checkout
    {{ cargo_min_age }} update "$@"
    just deps-age

# Upgrade the Rust dependency pins to the newest releases at least 7 days old (main checkout only)
[confirm("Upgrade the Rust dependency pins and rewrite Cargo.lock?")]
[group("deps")]
deps-upgrade *args: _main-checkout
    cargo upgrade --pinned --incompatible --recursive "$@"
    {{ cargo_min_age }} update
    just deps-age

# Upgrade the named JavaScript packages in the catalog of package.json to their newest releases at least 7 days old (main checkout only)
[group("deps")]
deps-upgrade-ts +packages: _main-checkout
    bun update --latest "$@"

# Fail when a crate in Cargo.lock was published less than 7 days ago (network)
[group("deps")]
deps-age:
    dev/crate-age Cargo.lock

# Security advisories of both dependency graphs and the crate publish age (network), keep-going
[group("deps")]
audit: _js
    TREEKNIT_JS_READY=1 dev/run-checks --serial audit-rs audit-ts deps-age

# Security advisories of the Rust dependencies (cargo-deny, network)
[group("deps")]
audit-rs:
    cargo deny --locked --all-features --config .config/deny.toml check advisories

# Security advisories of the JavaScript dependencies (bun audit, network)
[group("deps")]
audit-ts: _js
    bun audit

# Newer releases of the tools pinned in .config/mise.toml
[group("deps")]
tools-outdated:
    mise outdated --bump

# Lock .config/mise.lock for every supported host platform after a change to .config/mise.toml; name the changed tools to lock only those
[group("deps")]
tools-lock *tools:
    mise lock --platform linux-x64,linux-arm64,macos-x64,macos-arm64 "$@"

# Install the JavaScript dependencies from bun.lock
_js:
    [[ -n "${TREEKNIT_JS_READY:-}" ]] || bun install --frozen-lockfile --silent

_main-checkout:
    test "$(git rev-parse --path-format=absolute --git-common-dir)" = "$(git rev-parse --path-format=absolute --git-dir)" || { printf 'run this recipe in the main checkout, not in a worktree\n' >&2; exit 1; }
