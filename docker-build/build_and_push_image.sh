#!/usr/bin/env bash
set -euo pipefail

image="lausdahl/unifmu-build:latest"
builder="unifmu-multiarch"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if ! docker buildx inspect "${builder}" >/dev/null 2>&1; then
    docker buildx create --name "${builder}" --driver docker-container
fi

docker buildx build \
    --builder "${builder}" \
    --platform linux/amd64,linux/arm64 \
    --progress=plain \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --cache-to type=inline \
    -t "${image}" \
    --push \
    -f "${script_dir}/Dockerfile" \
    "${script_dir}"
