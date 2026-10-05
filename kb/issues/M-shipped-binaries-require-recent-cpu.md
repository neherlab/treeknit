# Shipped binaries require a recent CPU

## Summary

The shipped builds (`just build prod`, `just build-cross prod`, and the artifacts of `.github/workflows/cli-build.yml`) compile for a fixed CPU baseline, set in `dev/lib/dist-flags.sh`: Haswell (x86-64-v3 with AVX2 and FMA) on x86_64, and ARMv8.2-A extensions (LSE atomics, RDM, and others) on Linux aarch64. A binary built this way stops with `SIGILL` (illegal instruction) on an older CPU. The CLI help and the README do not state the requirement; only the developer guide does.

## Evidence

- [dev/lib/dist-flags.sh#L22](../../dev/lib/dist-flags.sh#L22) sets `-C target-cpu=haswell` for every `x86_64*` target
- [dev/lib/dist-flags.sh#L19](../../dev/lib/dist-flags.sh#L19) sets `-C target-cpu=generic -C target-feature=+crc,+dpb,+lor,+lse,+neon,+pan,+ras,+rdm,+vh` for the aarch64 Linux targets
- TreeKnit.jl runs on any CPU that runs Julia

## Impact

- The x86_64 binaries fail on CPUs older than Haswell (2013), which remain in HPC clusters and older servers
- The Linux aarch64 binaries fail on ARMv8.0 cores such as the Cortex-A53 and Cortex-A72 (Raspberry Pi 3 and 4, AWS Graviton1)
- The failure is a crash without an error message, so users cannot tell the cause

> [!IMPORTANT]
> **Investigation required.** No benchmark compares the baseline with `x86-64-v2` or generic aarch64 builds. The speedup that justifies the baseline is unmeasured; `just bench` on both builds would measure it.

## Fix direction

- Lower the baseline to `x86-64-v2` and generic aarch64, if the benchmarks show a small cost
- Or ship two builds per architecture (baseline and optimized), and select one at installation or with a small launcher
- Or keep the baseline, state the CPU requirement where users download the binaries, and print a clear error at startup when the CPU lacks a required feature

## Validation

- A shipped x86_64 binary starts under `qemu-x86_64 -cpu Nehalem`, and an aarch64 binary under `qemu-aarch64 -cpu cortex-a72`, or prints the error message when the baseline stays
