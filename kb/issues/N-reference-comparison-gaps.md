# Features without a comparison with TreeKnit.jl

`packages/treeknit-io/tests/fixtures.rs` compares the port with the fixtures that `ref/` writes from TreeKnit.jl. Some features have no such comparison, and some fixture data is not read. A difference in these features would pass the test suite.

## Locations

- **Pipeline for more than two trees**: the fixtures with three or more trees (`sim_k3_*`, `doc_multitreeknit_3`, `doc_resolve_3`, `doc_resolve_4`, `test_resolve_3_*`, `test_run_treeknit_3`, `test_run_parallel_5`) hold five seeded Julia runs of `run_treeknit!` in the field `multi_runs`. No test reads the field. `annealing_distribution_vs_julia` runs only two-tree cases
- **ARG branch lengths and text**: the ARG check compares the hybrid count and the splits of the segment trees. The branch lengths of `set_branch_lengths`, the extended Newick text, and `nodes.dat` have no comparison
- **Strict polytomy sort**: the fixture field `sorted_leaf_order` covers the non-strict sort only. The strict sort in `fn sort_pair` ([pipeline.rs#L329-L352](../../packages/treeknit-core/src/pipeline.rs#L329-L352)) has no test
- **Cooling schedules**: a unit test checks the geometric schedule. The linear and acos schedules ([anneal.rs#L29-L48](../../packages/treeknit-core/src/anneal.rs#L29-L48)) have no test
- **Auspice JSON**: no test checks the content of `auspice_<label>.json`; the command-line test checks only that the file exists, and that test runs only where the TreeKnit.jl examples exist ([`N-cli-test-skips-without-julia-examples.md`](N-cli-test-skips-without-julia-examples.md))
- **MCC readers**: `mccs::from_json` and `mccs::from_lines` have no caller and no test ([mccs.rs#L42-L67](../../packages/treeknit-io/src/mccs.rs#L42-L67))

## Fix direction

- Compare the `multi_runs` field with Rust runs of the TreeKnit.jl presets (`Options::treeknit_jl`), as distributions, like `annealing_distribution_vs_julia`
- Add the ARG branch lengths and the strict sort order to the fixtures in `ref/fixture_lib.jl`, regenerate the fixtures, and compare them. The `eps()` offset of TreeKnit.jl needs a tolerance or a decision first ([`N-undocumented-differences-from-treeknit-jl.md`](N-undocumented-differences-from-treeknit-jl.md))
- Add unit tests for the linear and acos schedules with values from TreeKnit.jl `get_cooling_schedule`
- Add a round-trip test for `to_json`/`from_json` and `to_lines`/`from_lines`, or remove the readers

## Validation

- Each new comparison fails when one value of the port is changed on purpose
