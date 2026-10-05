# Likelihood comparison with TreeKnit.jl uses a computed tolerance

The energy check in `packages/treeknit-io/tests/fixtures.rs` (`deterministic_functions_match_julia`) accepts a branch-length likelihood when `(got - lk).abs() < 1e-8 * (1.0 + lk.abs())`. The tolerance is computed from the expected value: absolute near 0, relative for large likelihoods. The project testing rules require a literal tolerance (`1e-N`) and list computed tolerances as a hard failure, because a scaled tolerance can hide a real difference in large values.

> [!IMPORTANT]
> **Decision required.** Choose between an absolute literal tolerance (for example `1e-7` or tighter, after checking the range of `lk` in the fixtures) and a documented relative comparison with a literal bound. The evidence to collect is the largest `abs(lk)` in `fixtures/*.json` and the largest observed `abs(got - lk)`.

## Validation

- `just test-rs deterministic_functions_match_julia` passes with the chosen tolerance, and fails when one likelihood term is perturbed by more than the tolerance
