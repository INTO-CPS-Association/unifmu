#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "${repo_root}"

for tool in cargo rustc protoc java dotnet; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "Native tests require ${tool}; see development/README.md." >&2
        exit 1
    fi
done
dotnet_version="$(dotnet --version)"
if [[ "${dotnet_version%%.*}" -lt 10 ]]; then
    echo "Native tests require .NET SDK 10 or newer; PATH selects ${dotnet_version}. Put the .NET 10 installation first on PATH." >&2
    exit 1
fi

python="${PYTHON:-}"
if [[ -z "${python}" ]]; then
    for candidate in python3 python3.12 python3.14 python3.13 python3.11; do
        if command -v "${candidate}" >/dev/null 2>&1 &&
            "${candidate}" -c 'import sys; sys.exit(sys.version_info < (3, 11))'; then
            python="${candidate}"
            break
        fi
    done
fi
if [[ -z "${python}" ]] ||
    ! "${python}" -c 'import sys; sys.exit(sys.version_info < (3, 11))'; then
    echo "Native tests require Python 3.11 or newer; set PYTHON to its executable." >&2
    exit 1
fi

venv="${repo_root}/target/native-test-venv"
if [[ ! -x "${venv}/bin/python3" ]]; then
    "${python}" -m venv "${venv}"
fi
"${venv}/bin/python3" -m pip install --disable-pip-version-check \
    -r "${repo_root}/scripts/requirements-test.txt"
export VIRTUAL_ENV="${venv}"
export PATH="${venv}/bin:${PATH}"

./assets/java/gradlew -p assets/java/runtime prepareEmbeddedRuntime --no-daemon

host_target="$(rustc -vV | sed -n 's/^host: //p')"
echo "Testing UniFMU natively on ${host_target}"
# Keep host build metadata separate from Docker's Linux build metadata.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${repo_root}/target/native-tests}"
cargo test --workspace --target "${host_target}" --release "$@"
