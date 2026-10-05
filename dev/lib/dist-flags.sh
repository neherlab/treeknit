#!/usr/bin/env bash
#
# The CPU flags of the shipped builds, shared by the prod, profiling, and bench
# builds of the justfile, so a local `just build prod` compiles the code that
# ships.

# Export RUSTFLAGS of the shipped build for a target, by default the host.
# RUSTFLAGS replaces the target rustflags of .cargo/config.toml (mold, frame
# pointers, v0 symbol mangling) instead of adding to them.
export_dist_flags() {
  local target="${1:-$(rustc --print host-tuple)}"

  case "${target}" in
  aarch64-apple-*)
    export RUSTFLAGS="-C target-cpu=apple-m1"
    ;;
  aarch64-*)
    # Features of -C target-feature=+v8.2a, which is unstable, listed explicitly.
    export RUSTFLAGS="-C target-cpu=generic -C target-feature=+crc,+dpb,+lor,+lse,+neon,+pan,+ras,+rdm,+vh"
    ;;
  x86_64-*)
    export RUSTFLAGS="-C target-cpu=haswell"
    ;;
  *)
    printf 'Unsupported target: %s\n' "${target}" >&2
    return 1
    ;;
  esac
}
