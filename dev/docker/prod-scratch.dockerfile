# syntax=docker/dockerfile:1
# check=experimental=all
#
# Production image neherlab/treeknit without a base: only the static musl build
# of the CLI as /usr/bin/treeknit, which carries its own root certificates for
# https: trees. dev/docker/prod-images stages the binary of each platform under
# .out/docker/<arch>/musl/.
FROM scratch

ARG TARGETARCH
COPY ".out/docker/${TARGETARCH}/musl/treeknit" "/usr/bin/treeknit"

ENV PATH="/usr/bin"
