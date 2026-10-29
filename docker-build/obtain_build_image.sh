#!/usr/bin/env bash
set -euo pipefail

image="lausdahl/unifmu-build:protobuf-36.2"
local_image="unifmu-build:latest"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if docker pull "${image}"; then
    docker tag "${image}" "${local_image}"
else
    echo "Could not pull ${image}; building a local linux/amd64 image instead." >&2
    docker buildx build \
        --platform linux/amd64 \
        --load \
        --progress=plain \
        --build-arg BUILDKIT_INLINE_CACHE=1 \
        -t "${local_image}" \
        -f "${script_dir}/Dockerfile" \
        "${script_dir}"
fi
