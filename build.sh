#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v docker >/dev/null 2>&1; then
    echo "Docker is required to run the full cross-platform build." >&2
    exit 1
fi

bash "${repo_root}/docker-build/obtain_build_image.sh"

docker run --rm \
    --volume "${repo_root}:/workdir" \
    --workdir /workdir \
    unifmu-build:latest \
    ./docker-build/build_all.sh

bash "${repo_root}/test.sh"
