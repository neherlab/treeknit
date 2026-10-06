# MCC counts are not compared with exact solutions

The number of reassortments that the port reports, the number of MCCs minus one, is at least the hybridization number of the resolved trees and at least their rooted SPR distance ([`agreement-forest-validation.md`](../proposals/agreement-forest-validation.md)). The tests compare the port with TreeKnit.jl and with simulated truth, but never with these exact lower bounds, so an inference that reports too few reassortments on a small case would pass.

## Fix direction

- Add a test helper that finds the smallest acyclic agreement forest of two trees of up to about 8 leaves by enumerating all partitions of the leaves
- Run inference on random small pairs and on the small fixtures, and check that the number of MCCs is at least the size of that forest
- Optionally compare with the rSPR distance of the external program `rspr` in a script under `ref/`, run in a container

> [!IMPORTANT]
> **Decision required.** `rspr` is licensed GPL-3.0-or-later and lists no hybridization-number mode in its documentation. The options are the brute-force helper alone, which covers small trees and needs no external program, or the helper plus `rspr` as an external command for larger binary trees, which gives only the weaker rSPR bound.

## Validation

- On a case where the port finds a smallest forest, the test fails when the inference is changed on purpose to report one MCC fewer
