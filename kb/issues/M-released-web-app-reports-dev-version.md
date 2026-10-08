# Released web app reports the development version

The web app shows the TreeKnit version that `version()` of `packages/treeknit-wasm` returns. The build script of the bindings derives it with the rule that the command line uses (`packages/version_rule.rs`): `TREEKNIT_VERSION` when set, otherwise the crate version with the suffix `-dev`. The release workflow `.github/workflows/release.yml` sets `TREEKNIT_VERSION` only for the command-line binaries of a version tag. A tag never deploys the web app: the nightly `deploy-web` job deploys `main` without `TREEKNIT_VERSION`, so the web app built from a release commit reports `<crate version>-dev` while the binaries of the same release report the release version.

## Fix direction

- In the `deploy-web` job, set `TREEKNIT_VERSION` to the crate version when the deployed commit is the commit of the tag `v<crate version>`; `dev/docker/run` already forwards the variable into the build container

## Validation

- After the nightly that deploys a release commit, the version in the web app equals the output of `treeknit --version` of that release
