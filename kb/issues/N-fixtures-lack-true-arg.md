# Simulated fixtures store no true ARG

The simulated fixtures store the segment trees and the true MCCs of each pair (`true_mccs`, from `ARGTools.MCCs_from_arg`, see [`ref/README.md`](../../ref/README.md#format-notes)), but not the simulated ARG. An inferred ARG of three or more segments ([`N-arg-limited-to-two-trees.md`](N-arg-limited-to-two-trees.md)) can therefore be compared with the truth only through its pairwise MCCs, not event by event. The simulated cases also have at most three segments, while influenza A has eight.

ARGTools can provide the truth. Its simulator takes the number of segments as a parameter, and each simulated reassortment splits the segments of a lineage into two groups [[src](https://github.com/PierreBarrat/ARGTools/blob/824b371cd0a2fd79fe80d4848e3445b3e6718686/src/simulate.jl#L383-L384)]. `write(filename, arg; pruned_singletons, tau)` writes the ARG in extended Newick, with `#H<i>` for reassortment nodes and `[&segments={...}]` for the segments of each branch [[src](https://github.com/PierreBarrat/ARGTools/blob/824b371cd0a2fd79fe80d4848e3445b3e6718686/src/IO.jl#L221)]. Its test covers three segments [[src](https://github.com/PierreBarrat/ARGTools/blob/824b371cd0a2fd79fe80d4848e3445b3e6718686/tests/test_extnwk.jl#L59-L69)].

## Fix direction

- In `ref/simulate.jl`, write the simulated ARG of each case next to its trees, as `fixtures/sim/<case>/arg.nwk`
- Add cases with more segments, for example four and eight, at the reassortment rates of the existing cases
- Regenerate the fixtures with the scripts of `ref/`, never by hand

## Validation

- `ref/README.md` describes the new file, and the output stays byte-identical across reruns
- In the Julia script, the segment trees of the stored ARG (`ARGTools.trees_from_ARG`) equal the written trees of the case
