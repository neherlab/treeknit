# Released web app reports the development version

The web app shows the TreeKnit version that `version()` of `packages/treeknit-wasm` returns. The build script of the bindings derives it with the rule that the command line uses (`packages/version_rule.rs`): `TREEKNIT_VERSION` when set, otherwise the crate version with the suffix `-dev`. The release workflow `.github/workflows/release.yml` sets `TREEKNIT_VERSION` only in the job that builds the command-line binaries. Its `deploy-web` job builds the web app without it, so a released web app reports `<crate version>-dev` while the binaries of the same release report the release version.

## Fix direction

- Set `TREEKNIT_VERSION: ${{ needs.version.outputs.version }}` in the `env` of the `deploy-web` job; `dev/docker/run` already forwards the variable into the build container

## Validation

- After a release or a nightly, the version in the web app equals the output of `treeknit --version` of the same release
