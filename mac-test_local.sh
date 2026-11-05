ARCH="x86_64"
OS="apple"
ABI="darwin"
RUST_TARGET="${ARCH}-${OS}-${ABI}"
FMI_PLATFORM_TUPLE="${ARCH}-darwin"
LIB_EXT="dylib"
set -e

# Detect host OS in a shell-portable way.
if [[ "$(uname)" == "Darwin" ]]; then
  echo "Using Docker for non-macOS host."

  docker build . -t unifmu-protoc:latest
  docker run --rm -v "$(pwd)":/app unifmu-protoc:latest protoc -I=./schemas --python_out=./assets/auto_generated --csharp_out=./assets/auto_generated --java_out ./assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto
else
  protoc -I=./schemas --python_out=./assets/auto_generated --csharp_out=./assets/auto_generated --java_out ./assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto

fi



cargo build --package fmiapi --target "${RUST_TARGET}" --release
mkdir -p "./assets/auto_generated/binaries/${FMI_PLATFORM_TUPLE}"
cp "./target/${RUST_TARGET}/release/libfmiapi.${LIB_EXT}" "./assets/auto_generated/binaries/${FMI_PLATFORM_TUPLE}/unifmu.${LIB_EXT}"
cargo test
cargo run --bin unifmu --release -- generate python myfmu
cargo run --bin unifmu --release -- generate python myfmu.fmu --zipped
fmpy simulate myfmu.fmu
