# syntax=docker/dockerfile:1
# check=experimental=all
#
# Cross-compilation image for one macOS release target, selected by the build
# arguments that dev/docker/run passes for each target of dev/cross/targets.
# osxcross provides the macOS SDK, the Apple linker (ld64), and the clang
# wrappers of each target; the wrappers run the clang of the base image. The
# base image is that of native.dockerfile, whose gcc links the build scripts and
# procedural macros, which run here.
FROM ubuntu:noble-20260905@sha256:a053cbffda9d424679c103c5b4f452297efc3774a1e491c110289f726fbb5d34
SHELL ["bash", "-euxo", "pipefail", "-c"]

# CROSS_COMPILE is the Rust target
ARG CROSS_COMPILE
# CROSS_COMPILE_UPPER is the Rust target in the form of cargo's variables
ARG CROSS_COMPILE_UPPER
# CROSS_COMPILE_SAFE is the Rust target in the form of the cc crate's variables
ARG CROSS_COMPILE_SAFE
# CROSS_APPLE_TRIPLET is the triplet of the osxcross tools
ARG CROSS_APPLE_TRIPLET

ENV CROSS_COMPILE="${CROSS_COMPILE}"
ENV CROSS_APPLE_TRIPLET="${CROSS_APPLE_TRIPLET}"

RUN set -euxo pipefail >/dev/null \
&& export DEBIAN_FRONTEND=noninteractive \
&& apt-get update -qq \
&& apt-get install --no-install-recommends --yes -qq \
  ca-certificates \
  clang \
  curl \
  file \
  gcc \
  git \
  libc6-dev \
  pixz \
  xz-utils \
>/dev/null \
&& apt-get clean autoclean >/dev/null \
&& apt-get autoremove --yes >/dev/null \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

COPY dev/docker/files/fetch dev/docker/files/checksums /

ENV OSX_CROSS_PATH="/opt/osxcross"
COPY dev/docker/files/install-osxcross /
RUN set -euxo pipefail >/dev/null \
&& /install-osxcross "${OSX_CROSS_PATH}" \
&& rm /install-osxcross

# The oldest macOS that rustc supports on x86_64; on aarch64, rustc raises it to
# its minimum, 11.0. ld64 loads libtapi and libxar from the osxcross library
# directory.
ENV MACOSX_DEPLOYMENT_TARGET="10.12"
ENV LD_LIBRARY_PATH="${OSX_CROSS_PATH}/lib"
ENV PATH="${OSX_CROSS_PATH}/bin:${PATH}"

# The target settings of cargo and of the cc crate (a C dependency would find
# its compiler here). dev/cross/check reads the binary tools of the target.
ENV CROSS_SYSROOT="${OSX_CROSS_PATH}/SDK/MacOSX11.1.sdk"
ENV CROSS_CC="${OSX_CROSS_PATH}/bin/${CROSS_APPLE_TRIPLET}-clang"
ENV CROSS_CXX="${OSX_CROSS_PATH}/bin/${CROSS_APPLE_TRIPLET}-clang++"
ENV CROSS_AR="${OSX_CROSS_PATH}/bin/${CROSS_APPLE_TRIPLET}-ar"
ENV CROSS_OTOOL="${OSX_CROSS_PATH}/bin/${CROSS_APPLE_TRIPLET}-otool"
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
