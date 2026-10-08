# Continuous integration and releases

This page is for maintainers: it describes the workflows that check every change, and how a release is published to every channel. [`developer_guide.md`](developer_guide.md) describes the development environment and the `just` recipes.

Four workflows in `.github/workflows/` check every change and publish the releases. They run the same `just` recipes and scripts as a developer, in the same containers (`dev/docker/run`):

- `ci.yml`: the checks of pull requests and of `main`
- `cli-build.yml`: builds the shipped CLI for every release target of `dev/cross/targets`, one job per target in its cross image, runs `dev/cross/check` on each binary, and uploads it as the artifact `treeknit-<target>`. A failed target fails the run
- `cli-compat.yml`: tests the binaries of `cli-build.yml` where users run them (see [Compatibility tests](#compatibility-tests)) and writes the PyPI wheels
- `release.yml`: the nightly deployment of the web app, and the release of a pushed version tag

`ci.yml` and `release.yml` call `cli-build.yml` and `cli-compat.yml`. The composite action `.github/actions/prepare` frees disk space on the runner, restores the cargo registry and the kache compiler cache, and pulls or builds the image of `dev/docker/run`. Every job that runs a container uses it.

## Checks of pull requests and `main`

`ci.yml` runs on every pull request, on every push to `main`, and by hand ("Run workflow"). The fast recipes run in parallel jobs:

- `just check`: formatting, Clippy (native and WebAssembly), oxlint, TypeScript types
- `just lint-shell`, `just lint-py`, `just lint-docker`, `just lint-workflows`
- `just check-group tests` (`test-rs`, `test-wasm`) and `just test-ts`
- `just generated-check`

The slow checks (dylint, hawk, knip, cargo-deny, cargo-shear, `just build-web prod`) run in no workflow; `just check-all` runs them locally. `dev/release` releases only a commit on which `ci.yml` passed.

The release builds (`cli-build.yml`) and the compatibility tests (`cli-compat.yml`) run on every push to `main`, on manual runs, and on pull requests that change `.github/`, `.cargo/`, `dev/cross/`, `dev/docker/`, `dev/lib/`, `dev/pypi-wheels`, `.config/mise.*`, `rust-toolchain.toml`, `Cargo.toml`, or `Cargo.lock`. These binaries report `<version>-dev`. The Docker images of the compatibility tests build 20 images, half of them under QEMU, so they run only when a push or pull request changes `dev/docker/` or `.github/`, and on manual runs.

## Compatibility tests

`cli-compat.yml` runs these jobs on the binaries of the same run:

- Linux distributions: `dev/cross/test-distros` with the x86_64 glibc and musl binaries (see [Linux distribution tests](developer_guide.md#linux-distribution-tests))
- macOS: on `macos-15-intel` (x86_64), `macos-15`, and `macos-latest` (arm64), `treeknit --version` and the simulated case of `test-distros`
- Windows: the same on `windows-2022`
- PyPI wheels: `dev/pypi-wheels` writes the wheels of the 7 binaries, uploaded as the artifact `pypi-wheels`. pip then picks and installs the wheel of its platform from them on `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15-intel`, `macos-15`, and `windows-2022`, and in the Alpine image of Python, and each installed `treeknit` runs
- Docker images: `dev/docker/prod-images --run` builds every image variant for both platforms and runs `treeknit --version` in each

For a release, every installed or built binary must report the release version.

## Nightly deployment of the web app

Every night at 03:40 UTC, `release.yml` deploys the web app of `main` to GitHub Pages (`just build-web prod`) when the commit of the latest successful deployment differs from `main`; otherwise it does nothing. The build uses relative asset paths (Vite `base: "./"`), so it works under the `/treeknit/` path of Pages and at any other path. The CLI is published by releases only.

## Start a nightly by hand

`just trigger-nightly` (host only, needs the GitHub CLI) starts a nightly on `main`, which deploys when `main` has changed. `--force` deploys an unchanged `main`. The "Run workflow" button of `release.yml` on GitHub takes the same option.

## Publish a release

1. Describe the changes under `## Unreleased` in `CHANGELOG.md`
2. Update the pinned bases of the Docker images: `./dev/docker/run dev/docker/prod-bases-update`, and commit `dev/docker/prod-bases.json` when it changed. Push `main` and wait until `ci.yml` passes on it; `CHANGELOG.md` may stay uncommitted
3. Run `just release <version>` (host only, in the main checkout on `main`). `dev/release` checks that the version is the workspace version of `Cargo.toml` or newer, that `main` is pushed and `ci.yml` passed on it (with the GitHub CLI), that nothing but `CHANGELOG.md` is uncommitted, and that the tag is new. It runs nothing heavy: it sets a new workspace version with `cargo set-version` in the build container (which also updates `Cargo.lock`) only when the version changes, renames `## Unreleased` to `## <version>`, commits `chore: release <version>`, and tags `<version>`, without a `v` prefix
4. Confirm the push: `dev/release` pushes `main` and the tag in one atomic push, and the tag starts `release.yml`. Answering no leaves the commit and the tag local
5. Review the Bioconda pull request when Bioconda maintainers ask for changes

The tag must match the workspace version in `Cargo.toml`. `release.yml` then runs these jobs:

1. `cli-build.yml` with the release version, then `cli-compat.yml` with every test, the Docker images included
2. `publish-release`, only when every build and test passed: the GitHub release of the tag, with the 7 binaries and the notes of `dev/release-notes`. A release of the tag can exist from an earlier attempt, such as a draft with some assets that a cancelled job left behind: then the job fails when an existing asset differs from the binary of this run, uploads the missing assets, and publishes the draft. A re-run never changes a published binary
3. After the GitHub release, in parallel: `pypi` uploads the wheels that the compatibility tests installed, `docker` builds, runs, and pushes the images, then `docker-description` updates the page of the images on Docker Hub, and `bioconda` opens the pull request that updates the Bioconda recipe
4. `verify-pypi` installs the release from PyPI on Linux and macOS, and `verify-docker` runs the image of the release on `linux/amd64` and `linux/arm64`; both retry while the new version is not yet served, and compare `treeknit --version`

A failed job publishes nothing after it; "Re-run failed jobs" runs it again. The `pypi` job compares the files that PyPI already has with the wheels of the run and fails on a difference, because PyPI never accepts a changed file under the same name. The next nightly deploys the web app of `main`, which holds the release commit. Every release stays on the releases page.

`dev/release-notes` writes the notes: the section `## <version>` of `CHANGELOG.md`, then a table of every way to get the release: the web app, every binary with its size, `pip`, `conda`, `docker pull`, the source commit, and the issue tracker. Footnotes cover the glibc and musl builds, unsigned macOS executables, the delay of Bioconda, `chmod +x`, and the CPU requirements.

## Release channels

Every channel ships the binaries of the GitHub release, which `dev/cross/` built and the compatibility tests ran; nothing is compiled a second time (`kb/decisions/channels-from-release-binaries.md`).

- **PyPI** (`pip install treeknit`): `dev/pypi-wheels --version <version> <binaries-dir> <output-dir>` writes one wheel per binary, with the binary as the script `treeknit` and a platform tag that lets pip pick it: `manylinux_2_17` for the glibc builds (glibc 2.17 or newer, the limit of `dev/cross/check`), `musllinux_1_1` for the static musl builds (only musl systems such as Alpine install them), `macosx_10_12_x86_64`, `macosx_11_0_arm64`, and `win_amd64`. There is no source distribution: pip reports "no matching distribution" on other platforms. The wheels are byte-identical for the same binaries: the zip entry dates come from `SOURCE_DATE_EPOCH` (the commit time by default), which `dev/docker/run` forwards. The description of the wheels starts with `docs/channels/summary.md`, the summary that the Docker Hub page shares. The script runs in the container: `./dev/docker/run dev/pypi-wheels --version 1.0.0 .out tmp/pypi-wheels`. The release uploads the wheels with Trusted Publishing, so no PyPI token is stored
- **Docker Hub** (`neherlab/treeknit`): `dev/docker/prod-images [--run] [--push]` (host only, needs Docker with buildx, QEMU for the other platform, `yq`, and `jq`) builds the images for `linux/amd64` and `linux/arm64` from the four Linux binaries in `.out/`: Debian with the glibc build and `bash`, `ca-certificates`, `curl`, `procps`, and `wget`, which workflow managers such as Nextflow need in task containers (`dev/docker/prod-debian.dockerfile`); Alpine and scratch with the static musl build (`prod-alpine`, `prod-scratch`). Each variant builds on every base of `dev/docker/prod-bases.json`: the 6 newest Alpine releases, the 3 newest Debian releases, and scratch. Tags, for version 1.0.0: `1.0.0-debian13`, `debian13`, `1-debian13`, `latest-debian13` for each base; the same with `debian` and `alpine` alone for the newest base of each distribution; and `1.0.0`, `1`, `latest` for the newest Debian. `--run` loads each platform and runs `treeknit --version` in every image; `--push` pushes the images of both platforms, after the runs with `--run`. The page of the repository is `docs/channels/summary.md` followed by `docs/channels/docker.md`, and its short description is the `description` of `[workspace.package]` in `Cargo.toml`
- **Bioconda** (`conda install -c bioconda treeknit`): the recipe `recipes/treeknit/meta.yaml` lives in `bioconda/bioconda-recipes` and installs the release binaries: the glibc builds on `linux-64` and `linux-aarch64`, the native builds on `osx-64` and `osx-arm64`. Bioconda has no Windows. `dev/bioconda-update --version <version> <binaries-dir>` clones that repository, sets the version, resets the build number, and sets the four sha256 values by the selector comments of the source lines; it leaves every other line as the Bioconda maintainers left it. It pushes the branch `bump/treeknit-<version>` to the fork `neherlab/bioconda-recipes` and opens the pull request, replacing an open one of the same branch. `--dry-run` stops after printing the diff, and `--recipe <file>` rewrites a local file instead. The first recipe is submitted by hand; until it is merged, the script prints a warning and opens nothing. The recipe turns off Bioconda's own version updates (`extra: autobump: enable: false`), which would update only some of the platform sha256 values of a recipe with one source per platform, so the release workflow is the only one that updates it. Bioconda maintainers review every update, so a Bioconda release can lag behind

## Docker images

`dev/docker/prod-bases.json` pins each base image of the production images to the digest of its Docker Hub tag, so the images of one release and a re-run use the same base. `./dev/docker/run dev/docker/prod-bases-update` rewrites it from the release lines of endoflife.date and the tags of Docker Hub, with the seven-day rule of the project:

- a release line released less than seven days ago is skipped
- when the tag of a line was pushed less than seven days ago, the line takes the newest dated tag of the same line that is at least seven days old: Debian keeps every build as `<codename>-YYYYMMDD`, such as `trixie-20260918`; Alpine has no dated tags, so an Alpine line keeps the digest that the file already has
- a line without such a digest fails the run, with the date from which a re-run succeeds

Run it before each release, so the images get the security fixes of their bases. The images of Debian 11, which is end-of-life, install their packages from `archive.debian.org` and the last security snapshot.

## Accounts, secrets, and environments

The workflows need these settings of the GitHub repository `neherlab/treeknit` and these external accounts:

- `NEHERLAB_BOT_DOCKERHUB_USERNAME`, `NEHERLAB_BOT_DOCKERHUB_TOKEN` (`neherlab` organization secrets): Docker Hub login of the bot account. Runs other than pull requests push the builder images `neherlab/treeknit_builder` and their layer cache, so later runs pull them instead of building them; the compatibility tests log in to raise the pull limit; the `docker` job of a release pushes `neherlab/treeknit`, which needs write access of the bot account to that repository; the `docker-description` job writes the page of that repository, which needs a token with the scope read/write/delete and the Admin permission of the bot account on the repository
- `NEHERLAB_BOT_GITHUB_TOKEN`, `NEHERLAB_BOT_GITHUB_NAME`, `NEHERLAB_BOT_GITHUB_EMAIL` (organization secrets): the token and commit identity of the bot account, with which the `bioconda` job pushes to the fork `neherlab/bioconda-recipes` and opens the pull request in `bioconda/bioconda-recipes`. The token must be a classic token with the scope `repo` or `public_repo`: a fine-grained token cannot open pull requests in a repository of another owner
- Environment `github-pages`: the deployment of the web app
- Environment `pypi`, limited to tags of the form `<major>.<minor>.<patch>`: the PyPI project `treeknit` trusts the workflow `release.yml` of this repository in this environment (Trusted Publishing), so the upload needs no token
- `GITHUB_TOKEN` (`github.token`): the GitHub release, and reading the deployments of GitHub Pages

## Version of a build

`treeknit --version` reports `TREEKNIT_VERSION` from build time (`packages/treeknit-cli/build.rs`), which must be the workspace version or start with it followed by `-`: `1.0.0-dev` when unset, as in the builds of `ci.yml`, and `1.0.0` for a release.

The workflow jobs pull the container images from Docker Hub by the hash of their build inputs, and build them when the inputs changed. Runs other than pull requests push the images they build to `neherlab/treeknit_builder` (see [Accounts, secrets, and environments](#accounts-secrets-and-environments)).
