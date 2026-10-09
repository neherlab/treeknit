# Command-line help lists `--match-topologies` as a former TreeKnit.jl option

`--help-resolve` says that the former options keep "their TreeKnit.jl meaning" and lists `--match-topologies` among them, with the equivalent `--resolve matched` ([main.rs#L34-L46](../../packages/treeknit-cli/src/main.rs#L34-L46)). The option exists as a hidden flag ([main.rs#L642-L643](../../packages/treeknit-cli/src/main.rs#L642-L643)), and `fn former_options` handles it. TreeKnit.jl has no such option on any branch, and `kb/feat/v0/cli.md` does not list it. The README table of former options does not list it either.

A reader of the help text looks for a TreeKnit.jl option that does not exist. The deprecation warning "--match-topologies is deprecated" calls it former too.

## Fix direction

- Remove `--match-topologies` from the command line and from the help text, since `--resolve matched` gives the same result
- Or keep it and describe it as an alias of `--resolve matched`, outside the list of TreeKnit.jl options

## Validation

- `treeknit --help-resolve` lists only options that `kb/feat/v0/cli.md` describes
