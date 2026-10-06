# Polytomy sort can differ from TreeKnit.jl for more than ten children

The non-strict polytomy sort orders the children of each node of the second tree with a comparison that is not a total order: two children in the MCC of their parent compare by their rank in that MCC, any other pair compares by clade size. The port sorts with an insertion sort, because the standard sorts of Rust may panic on such a comparison (`fn insertion_sort_by()`, [packages/treeknit-core/src/mcc_map.rs#L84-L93](../../packages/treeknit-core/src/mcc_map.rs#L84-L93); `kb/feat/visualization.md`). TreeKnit.jl sorts with Julia's default algorithm, which is an insertion sort only for short vectors, so the two orders can differ for polytomies with more than ten children.

## What Julia does

TreeKnit.jl calls `sortperm(rank, lt = _isless)` in `_sort_children!` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_tools.jl#L249-L262)]. In Julia 1.13.1, which wrote the fixtures:

- `sortperm` sorts an index vector with the default algorithm and the ordering `Perm` [[src](https://github.com/JuliaLang/julia/blob/96ca370cf0e339faf45a7ba863793f6c7c6980fb/base/sort.jl#L1968-L1995)]
- The default algorithm sorts vectors of up to 10 elements by insertion sort (`Small{10}` in `InitialOptimizations` [[src](https://github.com/JuliaLang/julia/blob/96ca370cf0e339faf45a7ba863793f6c7c6980fb/base/sort.jl#L1475-L1480)]). Longer vectors with a `Perm` ordering go to `StableCheckSorted(ScratchQuickSort())` [[src](https://github.com/JuliaLang/julia/blob/96ca370cf0e339faf45a7ba863793f6c7c6980fb/base/sort.jl#L1582-L1595)]
- `StableCheckSorted` returns a vector unchanged when no adjacent pair is out of order, and reverses it when every adjacent pair is strictly decreasing [[src](https://github.com/JuliaLang/julia/blob/96ca370cf0e339faf45a7ba863793f6c7c6980fb/base/sort.jl#L1358-L1381)]
- `ScratchQuickSort` partitions ranges longer than `SMALL_THRESHOLD = 20` [[src](https://github.com/JuliaLang/julia/blob/96ca370cf0e339faf45a7ba863793f6c7c6980fb/base/sort.jl#L1597)] and sorts shorter ranges by insertion sort

With a total order all of these give the same result. With the comparison above they can differ:

- **Up to 10 children**: both run an insertion sort with the same comparisons; the orders agree
- **11 to 21 children**: the orders differ only when every adjacent pair is strictly decreasing, where Julia reverses the vector and the insertion sort of the port can give another order
- **22 or more children**: quicksort partitioning compares other pairs than insertion sort, so the orders can differ

This analysis covers Julia 1.13.1 only. TreeKnit.jl accepts Julia 1.7 and later, and the default algorithms of other versions were not checked.

## Impact

MCCs and tree topologies do not change. The order of children in the output trees and drawings changes, and with it the order of the ARG files and the numbering of ARG nodes (`ARGNode_<i>`), which follows the order of the trees. The TreeKnit paper notes that trees of many closely related viruses often lack resolution and have polytomies ([Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394), Results).

> [!IMPORTANT]
> **Investigation required.** No case with a different order has been observed. Construct a polytomy of more than 21 children that mixes children in and outside the parent's MCC, and compare the leaf order of TreeKnit.jl and of the port.

## Fix direction

- If a difference is found: record it as a deliberate difference, or reproduce the reversal check of `StableCheckSorted` for 11 to 21 children; reproducing the quicksort for 22 or more children would copy Julia's partition scheme
- Add the observed case to the fixtures (`sorted_leaf_order`)

## Validation

- The fixture comparison covers a polytomy of more than 21 children
