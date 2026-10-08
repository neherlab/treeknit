
## Usage

```sh
docker run --rm --user="$(id -u):$(id -g)" --volume="$PWD:/data" --workdir=/data neherlab/treeknit treeknit ha.nwk na.nwk -o results
```

## Images

Each image exists for `linux/amd64` and `linux/arm64`:

- **Debian** (`latest`, `debian`): the glibc build, with `bash`, `ca-certificates`, `curl`, `procps`, and `wget`, which workflow managers such as Nextflow need in task containers
- **Alpine** (`alpine`): the static musl build
- **Scratch** (`scratch`): only the static musl build, which carries its own root certificates for https: trees

Tags such as `1.0.0`, `1-alpine`, `1.0.0-debian13`, or `alpine3.24` select a version and a base. The notes of each [release](https://github.com/neherlab/treeknit/releases) list its changes.
