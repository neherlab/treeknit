# syntax=docker/dockerfile:1
# check=experimental=all
#
# Development image: the Rust toolchain, Bun, and the lint and test tools.
# dev/docker/run builds it and runs every command as the host user, with
# HOME=/tmp/home and the cargo home in the checkout. Ubuntu 24.04 keeps the
# glibc of the binaries built here no newer than on current hosts, so the
# binaries run on the host.
FROM ubuntu:noble-20260905@sha256:a053cbffda9d424679c103c5b4f452297efc3774a1e491c110289f726fbb5d34
SHELL ["bash", "-euxo", "pipefail", "-c"]

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
  libssl-dev \
  make \
  pkg-config \
  unzip \
  xz-utils \
  zstd \
>/dev/null \
&& apt-get clean autoclean >/dev/null \
&& apt-get autoremove --yes >/dev/null \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

COPY dev/docker/files/fetch dev/docker/files/checksums /

# The Rust toolchain of rust-toolchain.toml, then the pinned nightly of the lint
# libraries.
ENV RUSTUP_HOME="/usr/local/rustup"
ENV CARGO_HOME="/usr/local/cargo"
ENV PATH="/usr/local/cargo/bin:${PATH}"
COPY dev/docker/files/install-rust /
COPY rust-toolchain.toml /tmp/rust/
COPY dev/lints/dylint-custom/rust-toolchain.toml /tmp/lints/dylint-custom/
COPY dev/lints/dylint-mordant/rust-toolchain.toml /tmp/lints/dylint-mordant/
COPY dev/lints/dylint-trailofbits/rust-toolchain.toml /tmp/lints/dylint-trailofbits/
# hadolint ignore=DL3003
RUN set -euxo pipefail >/dev/null \
&& /install-rust "/tmp/rust" \
&& for dir in /tmp/lints/*; do (cd "${dir}" && rustup toolchain install); done \
&& chmod -R a+w "${RUSTUP_HOME}" "${CARGO_HOME}" \
&& rm -rf /install-rust /tmp/rust /tmp/lints

# mise installs every tool of .config/mise.toml at the URL and sha256 of .config/mise.lock and
# links their executables into /usr/local/bin. The cargo registry that source
# builds fetch is a BuildKit cache, so it stays out of the image.
ENV MISE_DATA_DIR="/opt/mise"
ENV MISE_CACHE_DIR="/tmp/mise/cache"
ENV MISE_STATE_DIR="/tmp/mise/state"
COPY dev/docker/files/install-mise dev/docker/files/mise-version /
COPY .config/mise.toml .config/mise.lock /tmp/mise/project/
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
  set -euxo pipefail >/dev/null \
&& /install-mise "/usr/local/bin" \
&& export MISE_TRUSTED_CONFIG_PATHS="/tmp/mise/project" \
&& mise -C "/tmp/mise/project" install \
&& for dir in $(mise -C "/tmp/mise/project" bin-paths); do \
  find -L "${dir}" -mindepth 1 -maxdepth 1 -type f -executable -exec ln -sf -t "/usr/local/bin" {} + ; \
done \
&& chmod -R a+rX "${MISE_DATA_DIR}" \
&& rm -rf "/tmp/mise" "/install-mise" "/mise-version" \
&& just --version \
&& cargo nextest --version \
&& cargo dylint --version \
&& bun --version

# dev/docker/run runs as the host user with HOME=/tmp/home.
RUN set -euxo pipefail >/dev/null \
&& mkdir -p "/tmp/home" \
&& chmod 1777 "/tmp/home"
