# Command line accepts invalid settings, and zero sequence lengths crash it

The command line and the WebAssembly bindings each turn their settings into `treeknit_core::Options` with their own code: `fn options` in `packages/treeknit-cli/src/main.rs` ([main.rs#L293-L341](../../packages/treeknit-cli/src/main.rs#L293-L341)) and `fn options` in `packages/treeknit-wasm/src/analysis.rs` ([analysis.rs#L194-L222](../../packages/treeknit-wasm/src/analysis.rs#L194-L222)). Only the WebAssembly version checks the value ranges. The command line accepts values that the web app rejects.

## Evidence

- **Negative γ**: `treeknit ha.nwk na.nwk --gamma=-1` exits with 0. Every removal then lowers the score, so every leaf becomes its own MCC
- **Zero sequence lengths**: `--seq-lengths "0 0"` is accepted. `branch_likelihood` divides by $L_1 + L_2 = 0$ and returns `NaN` ([splitgraph.rs#L361-L366](../../packages/treeknit-core/src/splitgraph.rs#L361-L366)). In `choose_conf`, `f64::max` skips `NaN`, so the maximum is $-\infty$, no configuration equals it, and `confs.choose(rng).unwrap()` panics ([pair.rs#L110-L128](../../packages/treeknit-core/src/pair.rs#L110-L128)). Two trees with tied configurations reproduce it with every seed:

```sh
echo '((A:2,B:2):2,C:4);' > l1.nwk
echo '(A:2,(B:1,C:1):1);' > l2.nwk
treeknit l1.nwk l2.nwk --seq-lengths '0 0' --resolve none
# thread 'main' panicked at packages/treeknit-core/src/pair.rs:128:21
```

- **Negative and non-finite values**: the command line parses `--seq-lengths` and `--gamma` as `f64` with no range check, so negative, `NaN`, and infinite values also reach the core

## Fix direction

- Move the conversion from settings to `Options`, with its checks, into one Rust function that both the command line and the WebAssembly bindings call. The project rules require one implementation of each rule for both surfaces
- Reject a negative or non-finite γ and non-positive or non-finite sequence lengths, with the messages that `analysis::options` gives now

## Validation

- Command-line tests for `--gamma=-1`, `--gamma=nan`, and `--seq-lengths "0 0"` expect exit code 1 and the error message
- The tests of `analysis::analyze` keep passing with the shared function
