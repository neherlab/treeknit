# syntax=docker/dockerfile:1
# check=experimental=all
#
# Cross-compilation image for one Linux or Windows release target, selected by
# the build arguments that dev/docker/run passes for each target of
# dev/cross/targets. A GCC 14 toolchain (crosstool-ng) links the target: the
# glibc toolchains carry an old glibc, so the binaries run on old distributions,
# and the musl and MinGW toolchains link the C runtime statically. The base
# image is that of native.dockerfile, whose gcc links the build scripts and
# procedural macros, which run here.
FROM ubuntu:noble-20260905@sha256:a053cbffda9d424679c103c5b4f452297efc3774a1e491c110289f726fbb5d34
SHELL ["bash", "-euxo", "pipefail", "-c"]

# CROSS_COMPILE is the Rust target
ARG CROSS_COMPILE
# CROSS_COMPILE_UPPER is the Rust target in the form of cargo's variables
ARG CROSS_COMPILE_UPPER
# CROSS_COMPILE_SAFE is the Rust target in the form of the cc crate's variables
ARG CROSS_COMPILE_SAFE
# CROSS_GCC_TRIPLET is the triplet of the GCC toolchain
ARG CROSS_GCC_TRIPLET
# CROSS_RUNNER is the emulator of the target's binaries: QEMU for Linux on other architectures, Wine for Windows, empty when the host runs them
ARG CROSS_RUNNER

ENV CROSS_COMPILE="${CROSS_COMPILE}"
ENV CROSS_GCC_TRIPLET="${CROSS_GCC_TRIPLET}"
ENV CROSS_RUNNER="${CROSS_RUNNER}"

RUN set -euxo pipefail >/dev/null \
&& export DEBIAN_FRONTEND=noninteractive \
&& apt-get update -qq \
&& apt-get install --no-install-recommends --yes -qq \
  ca-certificates \
  curl \
  file \
  gcc \
  git \
  libc6-dev \
  make \
  pixz \
  xz-utils \
>/dev/null \
&& if [[ "${CROSS_RUNNER}" == qemu-* ]]; then apt-get install --no-install-recommends --yes -qq \
  qemu-user \
>/dev/null \
;fi \
&& if [[ "${CROSS_COMPILE}" == *-windows-* ]]; then apt-get install --no-install-recommends --yes -qq \
  wine64 \
>/dev/null \
;fi \
&& apt-get clean autoclean >/dev/null \
&& apt-get autoremove --yes >/dev/null \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

COPY dev/docker/files/fetch dev/docker/files/checksums /

ENV CROSS_GCC_DIR="/opt/cross"
COPY dev/docker/files/install-gcc-cross /
RUN set -euxo pipefail >/dev/null \
&& /install-gcc-cross "${CROSS_GCC_TRIPLET}" "${CROSS_GCC_DIR}" \
&& rm /install-gcc-cross

# The target settings of cargo and of the cc crate (a C dependency would find
# its compiler here). dev/cross/check reads the binary tools of the target.
# Build scripts find the target tools on the PATH, such as the dlltool that
# windows-sys runs to create import libraries. QEMU loads the dynamic linker and
# the C library of a glibc binary from the sysroot of the toolchain that linked
# it.
ENV CROSS_SYSROOT="${CROSS_GCC_DIR}/${CROSS_GCC_TRIPLET}/sysroot"
ENV CROSS_CC="${CROSS_GCC_DIR}/bin/${CROSS_GCC_TRIPLET}-gcc"
ENV CROSS_CXX="${CROSS_GCC_DIR}/bin/${CROSS_GCC_TRIPLET}-g++"
ENV CROSS_AR="${CROSS_GCC_DIR}/bin/${CROSS_GCC_TRIPLET}-gcc-ar"
ENV CROSS_OBJDUMP="${CROSS_GCC_DIR}/bin/${CROSS_GCC_TRIPLET}-objdump"
ENV CROSS_READELF="${CROSS_GCC_DIR}/bin/${CROSS_GCC_TRIPLET}-readelf"
ENV QEMU_LD_PREFIX="${CROSS_SYSROOT}"
ENV PATH="${CROSS_GCC_DIR}/bin:${PATH}"
ENV CC_${CROSS_COMPILE_SAFE}="${CROSS_CC}"
ENV CXX_${CROSS_COMPILE_SAFE}="${CROSS_CXX}"
ENV AR_${CROSS_COMPILE_SAFE}="${CROSS_AR}"
ENV CARGO_TARGET_${CROSS_COMPILE_UPPER}_LINKER="${CROSS_CC}"
ENV CARGO_TARGET_${CROSS_COMPILE_UPPER}_AR="${CROSS_AR}"

# The Rust toolchain of rust-toolchain.toml with the target. dev/docker/run runs
# as the host user with HOME=/tmp/home and the cargo home in the checkout.
ENV RUSTUP_HOME="/usr/local/rustup"
ENV CARGO_HOME="/usr/local/cargo"
ENV PATH="/usr/local/cargo/bin:${PATH}"
COPY dev/docker/files/install-rust /
COPY rust-toolchain.toml /tmp/rust/
RUN set -euxo pipefail >/dev/null \
&& /install-rust "/tmp/rust" "${CROSS_COMPILE}" \
&& rm -rf /install-rust /tmp/rust \
&& mkdir -p "/tmp/home" \
&& chmod 1777 "/tmp/home"

# Rust programs need bcryptprimitives.dll to start, which Wine lacks;
# dev/cross/run copies this stub into the Wine prefix.
ENV WINEARCH="win64"
ENV WINEDEBUG="err+all,err-winediag,err-ole,fixme-all"
COPY dev/docker/files/install-wine-bcryptprimitives /
RUN set -euxo pipefail >/dev/null \
&& if [[ "${CROSS_COMPILE}" == *-windows-* ]]; then /install-wine-bcryptprimitives "/opt/wine"; fi \
&& rm /install-wine-bcryptprimitives

# kache, the optional compiler cache of dev/cross/build, at the version, URL,
# and sha256 of .config/mise.lock, so the cross and development images run the
# same kache.
ENV MISE_DATA_DIR="/opt/mise"
ENV MISE_CACHE_DIR="/tmp/mise/cache"
ENV MISE_STATE_DIR="/tmp/mise/state"
COPY dev/docker/files/install-mise dev/docker/files/mise-version /
COPY .config/mise.toml .config/mise.lock /tmp/mise/project/
RUN set -euxo pipefail >/dev/null \
&& /install-mise "/usr/local/bin" \
&& export MISE_TRUSTED_CONFIG_PATHS="/tmp/mise/project" \
&& mise -C "/tmp/mise/project" install "github:kunobi-ninja/kache" \
&& ln -sf -t "/usr/local/bin" "$(mise -C "/tmp/mise/project" which kache)" \
&& chmod -R a+rX "${MISE_DATA_DIR}" \
&& rm -rf "/tmp/mise" "/install-mise" "/mise-version" "/usr/local/bin/mise" \
&& kache --version
