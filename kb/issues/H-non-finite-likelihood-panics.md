# Non-finite branch-length likelihood panics the tie-break

When several configurations tie after annealing, `fn choose_conf` keeps the configurations of maximal branch-length likelihood. A negative, infinite, or NaN branch length makes the likelihood NaN; the NaN configurations are dropped without a message, and when all of them are NaN the run panics.

## Reproduction

```
t1.nwk: ((A:-0.1,B:0.3):0.2,C:0.5);
t2.nwk: (A:0.2,(B:0.4,C:-0.2):0.1);
```

`treeknit t1.nwk t2.nwk --better-MCCs` panics: "called `Option::unwrap()` on a `None` value" at [pair.rs#L165](../../packages/treeknit-core/src/pair.rs#L165).

TreeKnit.jl 0.5.8 stops on the same input with `DomainError` from `log` of a negative number (`src/SplitGraph/likelihood.jl`). Neither implementation gives a usable message.

## Cause

- `fn branch_likelihood` ([splitgraph.rs#L361-L366](../../packages/treeknit-core/src/splitgraph.rs#L361-L366)) takes `ln(ns / n)`; a negative branch length makes the ratio negative and the result NaN. Lengths of opposite sign can also make the shared mean 0 and give `inf - inf`
- `fn choose_conf` ([pair.rs#L139-L166](../../packages/treeknit-core/src/pair.rs#L139-L166)) folds the likelihoods with `f64::max`, which skips NaN, then keeps the configurations equal to the maximum. NaN equals nothing, so NaN configurations are dropped; if every likelihood is NaN, the list is empty and `choose(rng).unwrap()` panics

## Scope

The Newick reader accepts negative, infinite, and NaN lengths. Neighbor-joining trees can contain negative lengths. The real datasets in `data/` have none.

## Fix direction

- Reject non-finite and negative branch lengths when reading trees, with the file and node in the message, or treat them as missing lengths (which contribute 0 to the likelihood, a documented difference from TreeKnit.jl)
- Make `fn choose_conf` total: an empty list after the likelihood filter must not reach `unwrap`

## Validation

- The reproduction above ends with a validation error naming the negative length, or completes
- A unit test of `fn choose_conf` with NaN likelihoods returns one of the input configurations
