# Chain and cluster reductions are not evaluated

The chain and cluster reductions of the exact algorithms for agreement forests could shorten the annealing on ladder-like trees and split a tree pair into independent parts ([`kernel-reductions.md`](../proposals/kernel-reductions.md)). Neither TreeKnit.jl nor the port applies them, and it is not known whether TreeKnit's score allows them.

> [!IMPORTANT]
> **Investigation required.** Show whether the TreeKnit score of a pair equals the sum of the scores of the restriction to a common cluster and of the pair in which the cluster is one leaf, including the case where fewer than two leaves of the cluster are kept. Without a proof, compare reduced and unreduced runs on the simulated fixtures by the distribution of MCC counts and of the variation of information to the true MCCs.

## Fix direction

- Depends on the investigation. With additivity: find the minimal common clusters of the reduced trees, anneal each part separately, and combine the removed leaves
- Compare with TreeKnit.jl by distributions, because seeded results change

## Validation

- Accuracy on the simulated fixtures (`just example accuracy`) does not get worse
- Run time on `data/h3n2-2k-4-segments` and on deep benchmark trees ([`N-benchmarks-lack-deep-trees.md`](N-benchmarks-lack-deep-trees.md)) before and after
