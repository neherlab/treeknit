# Pairs with fewer than two shared leaves crash the run or give one MCC

The port accepts trees with different leaf sets (`kb/feat/partial-overlap.md`). `fn infer` in `packages/treeknit-core/src/pipeline.rs` skips a pair that shares fewer than two leaves, with the warning "trees <a> and <b> share fewer than two leaves: skipped". The code around the skip fails in two ways.

## No shared leaf: panic

`infer` calls `restrict_pair` before it checks the shared-leaf count. `restrict_pair` calls `Tree::restricted`, which returns `None` for an empty leaf set, and the `expect("no shared leaves")` panics ([pipeline.rs#L246](../../packages/treeknit-core/src/pipeline.rs#L246)).

```sh
printf '((A:1,B:1):1,(C:1,(D:1,X:1):1):1);\n' > ha.nwk
printf '(P,Q);\n' > pq.nwk
treeknit ha.nwk pq.nwk -o out
# thread 'main' panicked at packages/treeknit-core/src/pipeline.rs:246:40: no shared leaves
```

The command exits with code 101 and writes only `log.txt` and `parameters.json`. In the web app, the panic traps the WebAssembly module, and the analysis fails with a runtime error.

## One shared leaf: all leaves in one MCC

With one shared leaf, `infer` returns that leaf as the only MCC. `attach_pair` then attaches every other leaf of both trees to it, so the pair gets one MCC that holds all leaves, and the imputed trees graft one tree into the other:

```sh
printf '(A,(P,Q));\n' > apq.nwk
treeknit ha.nwk apq.nwk -o out --impute
# MCCs.dat: A,B,C,D,P,Q,X
```

One MCC states that the two trees share all branches, which the input does not support. The ARG construction then fails with "shared node IMPUTED_3 has different parents".

## Fix direction

- Check the shared-leaf count before the restriction, and give a pair with fewer than two shared leaves a defined result
- Decide what that result is. One MCC per leaf states the most reassortments; no result for the pair states that the pair has no information. The JSON output, the imputed trees, and the ARG need a rule for this case

> [!IMPORTANT]
> **Decision required.** The result for a pair with zero or one shared leaf. Options: stop with an error that names the pair; skip the pair and leave it out of `MCCs.json` and of imputation; or give one MCC per leaf. TreeKnit.jl has no counterpart, because it requires equal leaf sets.

## Validation

- A command-line test runs two disjoint trees and two trees with one shared leaf, and checks the exit code and the outputs of the chosen rule
- A test of `analysis::analyze` covers the same inputs, so the web app gets an error message instead of a trap
