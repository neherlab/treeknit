# Release channels ship the release binaries

TreeKnit is published on GitHub Releases, PyPI (`pip install treeknit`), Docker Hub (`neherlab/treeknit`), and Bioconda (`conda install -c bioconda treeknit`). Rust command-line tools usually reach PyPI through maturin, which compiles a wheel per platform, and Bioconda through a recipe that compiles from source.

## Decision

- **One build for every channel.** `dev/cross/` builds the CLI once per release target in its cross image (`.github/workflows/cli-build.yml`), and every channel ships these binaries: the GitHub release attaches them, `dev/pypi-wheels` wraps each in a wheel, `dev/docker/prod-images` copies the Linux builds into the images, and the Bioconda recipe downloads the GitHub release assets and checks their sha256 values, which `dev/bioconda-update` writes
- **Tested before publication.** The release workflow publishes nothing until the compatibility tests of `.github/workflows/cli-compat.yml` pass: the Linux distribution images, the macOS and Windows runners, the installation of every wheel by pip, and the Docker images
- **No PyPI source distribution.** A wheel holds only the prebuilt binary as the script `treeknit`. On a platform without a wheel, pip reports "no matching distribution" instead of compiling
- **glibc first on Linux.** The `manylinux_2_17` wheels hold the glibc builds, which are the recommended Linux builds. The `musllinux_1_1` wheels hold the static musl builds, and pip installs them only on musl systems such as Alpine, where no manylinux wheel installs. The Bioconda recipe uses the glibc builds, because conda's Linux platforms provide glibc 2.17
- **The Bioconda recipe lives in Bioconda.** The recipe exists only in `bioconda/bioconda-recipes`; a second copy in this repository would drift from the one that Bioconda maintainers edit. Bioconda's automatic version updates are turned off in the recipe (`extra: autobump: enable: false`), because they would update only some of the four platform sha256 values

## Rationale

- **Every channel runs the tested binary.** A second compilation by maturin or by a Bioconda build would produce binaries with other compilers, linkers, and flags than the ones the compatibility tests ran, and the CPU flags of `dev/lib/dist-flags.sh` and the static linking of the musl and Windows builds would have to be repeated in each build system
- **Same version and behavior everywhere.** `treeknit --version` and every result are the same whichever channel installed the binary
- **PyPI accepts binary-only wheels.** Other projects publish prebuilt executables the same way, such as `ziglang`, whose wheels hold the Zig compiler. 230 Bioconda recipes repackage prebuilt binaries and skip the lint `should_be_noarch_generic`, as Nextclade does

## Consequences

- A platform without a release target has no wheel, image, or conda package; adding a target to `dev/cross/targets` adds it to `dev/pypi-wheels`, `dev/release-notes`, and, for Linux and macOS, the Bioconda recipe
- The glibc version of the `-gnu` builds is a promise of the wheel tags, the Bioconda recipe, and the release notes; `dev/cross/check` fails a build that needs a glibc newer than 2.17
- Every channel inherits the CPU requirement of the shipped builds ([`M-shipped-binaries-require-recent-cpu.md`](../issues/M-shipped-binaries-require-recent-cpu.md)), which wheel tags and conda platforms cannot express; the release notes, the README, and the PyPI and Docker Hub descriptions (`docs/channels/summary.md`) state it
