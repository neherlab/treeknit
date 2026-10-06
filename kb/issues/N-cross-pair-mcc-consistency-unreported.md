# Inconsistent MCCs across tree pairs are not reported

With three or more trees, two leaves can share an MCC in the pairs (1, 2) and (1, 3) but not in the pair (2, 3), which no single reassortment history explains. The port infers each pair on its own and does not report such contradictions. The condition, the TreeKnit.jl history, and the options are in [`cross-pair-mcc-consistency.md`](../proposals/cross-pair-mcc-consistency.md).

> [!IMPORTANT]
> **Decision required.** Report the inconsistencies as a diagnostic without changing inference, port the annealing penalty that TreeKnit.jl removed in 2023 because it had little effect on results, or repair the MCCs after inference by splitting them, which adds reassortments. The diagnostic changes no result; the other two do.

## Fix direction

- For a diagnostic: for each triple of trees and each orientation, group the shared leaves by their pair of MCC indices in two pairs and count the groups that span several MCCs of the third pair; log a warning and add the count to the run summary

## Validation

- A unit test with three trees whose pairwise MCCs contradict each other reports the contradiction; a test with consistent MCCs reports none
- Runs on `data/h3n2-2k-4-segments` report a count, and the MCCs stay byte-identical
