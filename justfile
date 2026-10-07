# TreeKnit task runner: `just` lists every recipe by group.
#
# Recipes run the tools pinned in .config/mise.toml, on the host after `mise install` or
# in the build container through `dev/docker/run just <recipe>`.
#
# Naming: a leaf recipe runs one tool in one mode. The suffix `-rs`, `-wasm`, or
# `-ts` names the toolchain of a leaf: Rust, Rust for WebAssembly, or TypeScript
# (Bun); tools that serve one toolchain only, such as dylint or knip, keep their
# own name.
# A recipe without a toolchain suffix combines the leaves and calls no tool
# itself. The suffix `-all` adds the slow tools to the fast set of the same
# name: `lint` and `lint-all`, `check` and `check-all`.

set minimum-version := "1.58.0"
set default-list
set dotenv-load
set positional-arguments
set shell := ["mise", "exec", "--", "bash", "-euo", "pipefail", "-c"]
set script-interpreter := ["mise", "exec", "--", "bash", "-euo", "pipefail"]
# User-defined functions, for the build modes
set unstable

export NEXTEST_NO_TESTS := "fail"

# Build output of the checkout. dev/docker/run points TREEKNIT_BUILD_DIR at
# .build/container, so host and container artifacts, which link against
# different system libraries, never mix. Builds and tests share one cargo target
# directory; clippy, dylint, and hawk each have their own, so a lint never waits
# on the build lock.
build_dir := env("TREEKNIT_BUILD_DIR", justfile_directory() / ".build/host")
export CARGO_TARGET_DIR := build_dir / "cargo"
lint_target_dir := build_dir / "lint"
dylint_target_dir := build_dir / "dylint"
hawk_target_dir := build_dir / "hawk"

# The kache compiler cache is optional. KACHE_STORE, in .env or the environment,
# names the store directory; every build and clippy pass then compiles through
# kache. KACHE_PRESERVE_INCREMENTAL keeps incremental compiles out of the cache:
# the workspace crates of dev, test, release, and clippy builds keep their
# incremental state, so an edit rebuilds as fast as without kache, while kache
# serves the dependencies and every build without incremental state (dist,
# profiling, and bench builds). Dylint, hawk, and coverage always compile
# without kache, because a cache hit skips their analysis and instrumentation.
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

# The dylint driver, shared by the check, fix, and baseline recipes. It loads
# the lint libraries of [workspace.metadata.dylint] in Cargo.toml, which read
# their settings from dylint.toml. The pub_unused_in_workspace lint leaves one
# record per compiled crate in the pub-unused directory, and pub-unused-report
# reads them after the check pass. Mordant lists the crates over the committed
# baseline in over-baseline.txt.
pub_unused_env := "CUSTOM_LINTS_PUB_UNUSED_DIR=" + quote(dylint_target_dir / "pub-unused")
dylint_cmd := uncached_env + " CARGO_TARGET_DIR=" + quote(dylint_target_dir) + " " + pub_unused_env + " cargo dylint --quiet --all"
dylint_cargo_args := "--quiet --locked --workspace --all-targets"

# Lint levels of the driver: lints that only one library declares are unknown
# to the others. Of the custom library, the comment lints (no_comments,
# doc_comment_limit) are off, because the Rust code keeps its comments, and the
# builder and error-macro lints (suggest_builder, needless_builder,
# prefer_error_macros) are off, because the project uses neither bon nor error
# helper macros.
dylint_rustflags := "-A unknown_lints -A no_comments -A doc_comment_limit -A suggest_builder -A needless_builder -A prefer_error_macros"
dylint_over_baseline := dylint_target_dir / "mordant/over-baseline.txt"

# Library crates whose public API is an external boundary, skipped by both
# unused-public-code checks (hawk and pub-unused-report): the `#[wasm_bindgen]`
# surface of the WebAssembly bindings is consumed by the web app, and the test
# helpers by the tests of the other crates, which the checks do not count.
public_api_crates := "treeknit_wasm treeknit_testing"

# Cargo with the seven-day minimum publish age of new dependency releases. The
# setting is unstable in the pinned Rust (stable from 1.100), so the dependency
# recipes enable it for their resolution only; RUSTC_BOOTSTRAP=1 allows unstable
# Cargo features on a stable toolchain.
cargo_min_age := "RUSTC_BOOTSTRAP=1 cargo -Zmin-publish-age --config 'registry.global-min-publish-age=\"7 days\"'"
hawk_toolchain := trim(read("dev/docker/files/hawk-toolchain"))

# Reason of the React 19 pin, printed by lint-ts when the catalog leaves 19.x.
react_pin_reason := "the project keeps React on 19.x"

# Build modes of the build and run recipes: dev, dev-opt, release, and
# profiling are the cargo profiles of the same name; prod, the shipped build, is
# the cargo profile dist. Cargo writes the dev profile to the debug directory.
cargo_profile(mode) := if mode == "prod" { "dist" } else { mode }
cargo_profile_dir(mode) := if mode == "dev" { "debug" } else { cargo_profile(mode) }

# The TypeScript package that wasm-bindgen writes for the WebAssembly module.
# Only its type declarations are committed, so the TypeScript checks read them
# without a Rust build; `just generated-check` keeps them current.
wasm_pkg := "packages/treeknit-wasm/pkg"
wasm_types := wasm_pkg / "treeknit_wasm.d.ts"
wasm_variants := wasm_pkg / "treeknit_variants.ts"

# Groups of the full gate (`just check-group <group>`). A check is a recipe name
# with colon-separated arguments: `build-web:prod` runs `just build-web prod`.
checks_format := "fmt-check-rs fmt-check-ts fmt-check-other lint-shell lint-docker lint-workflows deny shear"
checks_clippy := "lint-rs lint-wasm"
checks_dylint := "dylint"
checks_tests := "test-rs test-wasm"
checks_generated := "generated-check build-web:prod"
checks_typescript := "typecheck lint-ts knip test-ts"
checks_hawk := "hawk"
check_fast := "fmt-check-rs fmt-check-ts fmt-check-other lint-rs lint-wasm lint-ts typecheck"
check_full := checks_format + " " + checks_clippy + " " + checks_dylint + " " + checks_tests + " " + checks_generated + " " + checks_typescript + " " + checks_hawk
lint_fast := "lint-rs lint-wasm lint-ts"
lint_full := "lint-rs lint-wasm lint-ts typecheck dylint hawk knip deny shear lint-shell lint-docker lint-workflows"
dockerfiles := "dev/docker/*.dockerfile"

alias b := build
alias r := run
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
    JS_READY=1 dev/run-checks {{ check_fast }}

# Every check, in parallel; slow
[group("check")]
check-all: _js
    JS_READY=1 dev/run-checks {{ check_full }}

# One group of check-all, serially with streamed output: just check-group <format|clippy|dylint|tests|generated|typescript|hawk>
[group("check")]
check-group group: _js
    recipes="$(just --evaluate "checks_$1")"; JS_READY=1 dev/run-checks --serial ${recipes}

# Apply the fast automatic lint fixes (clippy, oxlint), then format; stage your changes first
[group("check")]
fix: lint-fix fmt

# Apply every automatic lint fix, dylint included, then format; stage your changes first
[group("check")]
fix-all: lint-fix dylint-fix fmt

# Install the pinned tools and the lint toolchains (on the host)
[group("setup")]
setup:
    if [[ -z "${TREEKNIT_CONTAINER:-}" ]]; then mise install; for dir in dev/lints/dylint-*/; do (cd "${dir}" && rustup toolchain install); done; rustup toolchain install {{ hawk_toolchain }} --profile minimal --component rustc-dev,llvm-tools-preview,rust-src; fi

# Build the CLI: just build <dev|dev-opt|release|prod|profiling> [cargo args]; prod is the shipped build (dist profile); release and prod copy the binary to .out/
[arg("mode", pattern="dev|dev-opt|release|prod|profiling")]
[group("build")]
build mode *args:
    if [[ {{ quote(mode) }} == prod || {{ quote(mode) }} == profiling ]]; then source dev/lib/dist-flags.sh && export_dist_flags; fi; cargo build --locked --profile={{ quote(cargo_profile(mode)) }} --bin treeknit "${@:2}"
    if [[ {{ quote(mode) }} == release || {{ quote(mode) }} == prod ]]; then mkdir -p .out && cp {{ quote(CARGO_TARGET_DIR / cargo_profile_dir(mode) / "treeknit") }} .out/; fi

# Cross-compile the CLI for the release targets into .out/treeknit-<target> and check its libraries (host only, needs Docker): just build-cross <dev|release|prod> [--target=<triple>] [--run] [--serial]; prod is the shipped build (dist profile)
[arg("mode", pattern="dev|release|prod")]
[group("build")]
build-cross mode *args:
    dev/cross/all --profile={{ quote(cargo_profile(mode)) }} "${@:2}" treeknit

# Build the WebAssembly package into packages/treeknit-wasm/pkg/: just build-wasm <dev|release|prod>; prod, as shipped, uses the dist profile and adds wasm-opt
[arg("mode", pattern="dev|release|prod")]
[group("build")]
build-wasm mode:
    cargo build --locked --profile={{ quote(cargo_profile(mode)) }} --target=wasm32-unknown-unknown -p treeknit-wasm
    wasm-bindgen --target=web --out-dir={{ wasm_pkg }} {{ quote(CARGO_TARGET_DIR / "wasm32-unknown-unknown" / cargo_profile_dir(mode) / "treeknit_wasm.wasm") }}
    if [[ {{ quote(mode) }} == prod ]]; then wasm-opt -O {{ wasm_pkg }}/treeknit_wasm_bg.wasm -o {{ wasm_pkg }}/treeknit_wasm_bg.wasm; fi

# Build the web app into packages/web/dist/: just build-web <dev|prod>; dev: WebAssembly of the release profile, unminified with source maps; prod: as shipped
[arg("mode", pattern="dev|prod")]
[group("app")]
build-web mode: _js (build-wasm (if mode == "dev" { "release" } else { "prod" }))
    bun run --silent build:web --mode {{ if mode == "dev" { "development" } else { "production" } }}

# Run the CLI: just run <dev|dev-opt|release|prod> ha.nwk na.nwk -o tmp/results; prod is the shipped build (dist profile)
[arg("mode", pattern="dev|dev-opt|release|prod")]
[group("run")]
run mode *args:
    if [[ {{ quote(mode) }} == prod ]]; then source dev/lib/dist-flags.sh && export_dist_flags; fi; args=("${@:2}"); [[ "${args[0]:-}" != "--" ]] || args=("${args[@]:1}"); cargo run --locked --profile={{ quote(cargo_profile(mode)) }} --bin treeknit -- ${args[@]+"${args[@]}"}

# Cross-compile the CLI for one release target and run it in its cross image, under QEMU or Wine (host only, needs Docker): just run-cross <dev|release|prod> <target> [CLI args]
[arg("mode", pattern="dev|release|prod")]
[group("run")]
run-cross mode target *args:
    CROSS_COMPILE={{ quote(target) }} dev/docker/run dev/cross/build --profile={{ quote(cargo_profile(mode)) }} treeknit {{ quote(target) }}
    args=("${@:3}"); [[ "${args[0]:-}" != "--" ]] || args=("${args[@]:1}"); CROSS_COMPILE={{ quote(target) }} dev/docker/run dev/cross/run treeknit {{ quote(target) }} ${args[@]+"${args[@]}"}

# Run an example (release profile): just example accuracy
[group("run")]
example name *args:
    cargo run --locked --release --example {{ quote(name) }} -- "${@:2}"

# Prepare a release and push it after confirmation: set the version, rename the Unreleased section of CHANGELOG.md to it, run every check, commit, and tag (host only, main checkout): just release <version>
[group("release")]
release version:
    dev/release "$@"

# Start a nightly in GitHub Actions (host only, needs gh): just trigger-nightly [--force] [--only <cli|web|all>]
[group("release")]
trigger-nightly *args:
    dev/trigger-nightly "$@"

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

# Quiet cargo passes --quiet on to the test binary, which wasm-bindgen-test-runner
# rejects: the tests compile quietly, then run with quiet mode off.
# Tests of the WebAssembly bindings, run in Node
[group("test")]
[script]
test-wasm *args:
    test=(cargo test --locked -p treeknit-wasm --target=wasm32-unknown-unknown --test=wasm --test=panic)
    "${test[@]}" --no-run "$@"
    CARGO_TERM_QUIET=false "${test[@]}" "$@"

# TypeScript tests (vitest)
[group("test")]
test-ts: _js
    bun run --silent test

# Cross-compile the CLI for a Linux x86_64 target and run it on a simulated case in many Linux distribution images (host only, needs Docker): just test-distros <dev|release|prod> <x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl> [--image=<ref>] [-- CLI args]
[arg("mode", pattern="dev|release|prod")]
[arg("target", pattern="x86_64-unknown-linux-(gnu|musl)")]
[group("test")]
test-distros mode target *args:
    CROSS_COMPILE={{ quote(target) }} dev/docker/run dev/cross/build --profile={{ quote(cargo_profile(mode)) }} treeknit {{ quote(target) }}
    dev/cross/test-distros treeknit {{ quote(target) }} "${@:3}"

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

# Copy-paste duplication across Rust and TypeScript (jscpd), never a gate
[group("report")]
duplication *args:
    jscpd --config .config/jscpd.json packages "$@"

# Inventory of lint suppressions and other review-sensitive settings
[group("report")]
review-suppressions:
    dev/review-suppressions

# Fast lints: clippy and oxlint, keep-going
[group("lint")]
lint: _js
    JS_READY=1 dev/run-checks --serial {{ lint_fast }}

# Every lint: the fast lints, TypeScript types, the lint libraries, unused code and dependencies, dependency policy, and the shell, Dockerfile, and workflow lints, keep-going
[group("lint")]
lint-all: _js
    JS_READY=1 dev/run-checks --serial {{ lint_full }}

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

# Lint libraries (dylint) gated against .config/mordant-baseline.toml, then the unused public items they recorded; reports every finding, then fails if there were any
[group("lint")]
[script]
dylint *args:
    status=0
    rm -f {{ quote(dylint_over_baseline) }}
    DYLINT_RUSTFLAGS={{ quote(dylint_rustflags) }} CARGO_BUILD_WARNINGS=deny {{ dylint_cmd }} -- {{ dylint_cargo_args }} --keep-going "$@" || status=1
    if [[ -s {{ quote(dylint_over_baseline) }} ]]; then printf 'mordant: findings over the committed baseline:\n' >&2; cat {{ quote(dylint_over_baseline) }} >&2; status=1; fi
    (cd dev/lints/dylint-custom && {{ uncached_env }} {{ pub_unused_env }} cargo run --quiet --release --locked --target-dir {{ quote(dylint_target_dir / "report") }} --bin pub-unused-report -- {{ quote(justfile_directory() / "Cargo.toml") }} {{ prepend("--exclude-crate ", public_api_crates) }}) || status=1
    exit "${status}"

# Apply the automatic fixes of the lint libraries (dylint); stage your changes first
[group("lint")]
dylint-fix:
    rm -f {{ quote(dylint_over_baseline) }}
    DYLINT_RUSTFLAGS={{ quote(dylint_rustflags) }} {{ dylint_cmd }} --fix -- --allow-staged {{ dylint_cargo_args }}

# Accept the current mordant findings: rewrites .config/mordant-baseline.toml, commit it afterwards
[confirm("Rewrite .config/mordant-baseline.toml with the current findings?")]
[group("lint")]
dylint-baseline:
    DYLINT_RUSTFLAGS={{ quote(dylint_rustflags) }} MORDANT_BASELINE_WRITE=1 {{ dylint_cmd }} -- {{ dylint_cargo_args }} --keep-going

# Unnecessary public surface (cargo-hawk), denying warnings
[group("lint")]
hawk *args:
    {{ uncached_env }} cargo +{{ hawk_toolchain }} hawk check --config .config/hawk.toml --target-dir {{ quote(hawk_target_dir) }} {{ prepend("--exclude-crate=", public_api_crates) }} -W warnings "$@"

# TypeScript lints (oxlint, type-aware) and the React 19 pin, keep-going
[group("lint")]
[script]
lint-ts: _js
    status=0
    bun run --silent lint || status=1
    jq -e '.workspaces.catalog.react | startswith("19.")' package.json >/dev/null || { printf 'the react catalog entry must stay on 19.x: %s\n' {{ quote(react_pin_reason) }} >&2; status=1; }
    exit "${status}"

# TypeScript type checks of the packages and the tool configs, keep-going
[group("lint")]
[script]
typecheck: _js
    status=0
    bun run --silent typecheck:packages || status=1
    bun run --silent typecheck:tools || status=1
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
    hadolint --config .config/hadolint.yaml {{ dockerfiles }}

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
    JS_READY=1 dev/run-checks --serial fmt-check-rs fmt-check-ts fmt-check-other

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

# Regenerate the TypeScript declarations and enum value lists of the WebAssembly package (packages/treeknit-wasm/pkg/treeknit_wasm.d.ts, treeknit_variants.ts) after a change to its interface
[group("generated")]
gen: (build-wasm "dev")
    cargo run --locked --quiet -p treeknit-wasm --example ts_variants > {{ wasm_variants }}

# Fail when the committed TypeScript declarations or enum value lists of the WebAssembly package differ from a fresh build; writes nothing into the checkout
[group("generated")]
[script]
generated-check:
    out="$(mktemp -d)"
    trap 'rm -rf "${out}"' EXIT
    cargo build --locked --target=wasm32-unknown-unknown -p treeknit-wasm
    wasm-bindgen --target=web --out-dir="${out}" {{ quote(CARGO_TARGET_DIR / "wasm32-unknown-unknown" / "debug" / "treeknit_wasm.wasm") }}
    diff -u {{ wasm_types }} "${out}/treeknit_wasm.d.ts" || { printf '%s is stale; run `just gen` and commit it\n' {{ quote(wasm_types) }} >&2; exit 1; }
    cargo run --locked --quiet -p treeknit-wasm --example ts_variants > "${out}/treeknit_variants.ts"
    diff -u {{ wasm_variants }} "${out}/treeknit_variants.ts" || { printf '%s is stale; run `just gen` and commit it\n' {{ quote(wasm_variants) }} >&2; exit 1; }

# Run the web app in the foreground until Ctrl-C, on the port of this checkout and mode: just run-web <dev|prod>; dev: Vite dev server with hot reload (`just build-wasm release` after a Rust change); prod: the shipped build, served
[arg("mode", pattern="dev|prod")]
[group("app")]
run-web mode: _js (build-wasm (if mode == "dev" { "release" } else { "prod" }))
    if [[ {{ quote(mode) }} == prod ]]; then bun run --silent build:web --mode production; fi
    TREEKNIT_WEB_PORT="$(dev/web-port dev)" TREEKNIT_SERVE_PORT="$(dev/web-port prod)" bun run --silent {{ if mode == "dev" { "dev:web" } else { "preview:web" } }}

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
    dev/crate-age Cargo.lock dev/lints/*/Cargo.lock

# Security advisories of both dependency graphs and the crate publish age (network), keep-going
[group("deps")]
audit: _js
    JS_READY=1 dev/run-checks --serial audit-rs audit-ts deps-age

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
    [[ -n "${JS_READY:-}" ]] || bun install --frozen-lockfile --silent

_main-checkout:
    test "$(git rev-parse --path-format=absolute --git-common-dir)" = "$(git rev-parse --path-format=absolute --git-dir)" || { printf 'run this recipe in the main checkout, not in a worktree\n' >&2; exit 1; }
