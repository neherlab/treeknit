# Contributing

Bug reports, questions, and pull requests are welcome in the [issue tracker](https://github.com/neherlab/treeknit/issues) of `neherlab/treeknit`.

## Report a bug

Give:

- the version: `treeknit --version`, or the version on the help page of the web app
- the command or the web app link that shows the problem
- the error or the unexpected output
- the input trees, when you can share them. The web app saves them with the settings in a session file, which `treeknit --session` runs

## Change the code

[`docs/dev/developer_guide.md`](docs/dev/developer_guide.md) describes the development environment, the repository layout, and the `just` recipes. The main rules:

- Run the recipes in the build container: `./dev/docker/run just <recipe>`, or `./dev/docker/run just` for the list
- Before a pull request, run `just check` (formatting, Clippy, oxlint, TypeScript types) and the tests of the changed code:
  - `just test-rs`: Rust
  - `just test-wasm`: the WebAssembly bindings
  - `just test-ts`: the web app
- After a change to the WebAssembly interface, to the command-line options, or to the output files, run `just gen` and commit the files it writes. `just generated-check` fails when they are stale
- Describe user-visible changes under `## Unreleased` in [`CHANGELOG.md`](CHANGELOG.md)

The tests compare the port with reference outputs of TreeKnit.jl in `fixtures/` ([`ref/README.md`](ref/README.md)). The knowledge base in [`kb/`](kb/README.md) records design decisions, known issues, and research reports. Read the issues of the area you change.

[`docs/dev/releases.md`](docs/dev/releases.md) describes the continuous integration and the publication of releases.
