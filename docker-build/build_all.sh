#!/bin/bash
set -euo pipefail

VER=$(python3 scripts/version.py)
echo "Building UniFMU ${VER}"

host_target=$(rustc -vV | sed -n 's/^host: //p')
case "${host_target}" in
    x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) ;;
    *) echo "Unsupported release build host: ${host_target}" >&2; exit 1 ;;
esac

targets=(
    x86_64-unknown-linux-gnu
    aarch64-unknown-linux-gnu
    x86_64-pc-windows-gnu
    x86_64-apple-darwin
    aarch64-apple-darwin
)

# ------------------------------ fmiapi ------------------------------
for target in "${targets[@]}"; do
    echo "building fmiapi for ${target}"
    cargo build --package fmiapi --target "${target}" --release

    case "${target}" in
        *-unknown-linux-gnu) platform="${target%%-*}-linux"; library=libfmiapi.so; extension=so ;;
        *-pc-windows-gnu) platform="${target%%-*}-windows"; library=fmiapi.dll; extension=dll ;;
        *-apple-darwin) platform="${target%%-*}-darwin"; library=libfmiapi.dylib; extension=dylib ;;
    esac
    mkdir -p "assets/auto_generated/binaries/${platform}"
    cp "target/${target}/release/${library}" \
        "assets/auto_generated/binaries/${platform}/unifmu.${extension}"
done

echo "creating universal macOS fmiapi library"
universal_library=assets/auto_generated/binaries/universal-darwin/unifmu.dylib
mkdir -p "$(dirname "${universal_library}")"
lipo -create \
    target/x86_64-apple-darwin/release/libfmiapi.dylib \
    target/aarch64-apple-darwin/release/libfmiapi.dylib \
    -output "${universal_library}"
lipo "${universal_library}" -verify_arch x86_64 arm64
lipo -info "${universal_library}"

# ------------------------------ schemas ------------------------------
echo "generating protobuf schemas for python, csharp, and java backends"
mkdir -p assets/auto_generated/fmi2
mkdir -p assets/auto_generated/fmi3
protoc -I=schemas --python_out=assets/auto_generated --csharp_out=assets/auto_generated --java_out=assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto
# ------------------------------ cli ------------------------------
echo "testing cli for ${host_target}"
cargo test --target "${host_target}" --release
for target in "${targets[@]}"; do
    echo "building cli for ${target}"
    cargo build --package unifmu --target "${target}" --release
done

# ------------------------------ compress for release ------------------------------

actual_version=$("./target/${host_target}/release/unifmu" --version)
if [[ "${actual_version}" != "unifmu ${VER}" ]]; then
    echo "CLI version mismatch: expected unifmu ${VER}, got ${actual_version}" >&2
    exit 1
fi

for target in "${targets[@]}"; do
    executable=unifmu
    if [[ "${target}" == *-pc-windows-gnu ]]; then
        executable=unifmu.exe
    fi
    echo "zipping cli for ${target}"
    zip -qj "target/unifmu-${target}-${VER}.zip" "target/${target}/release/${executable}"
done
