#!/bin/bash
# ------------------------------ fmiapi ------------------------------
tgt=fmiapi
echo "building fmiapi for linux"
cargo build --package ${tgt} --target x86_64-unknown-linux-gnu --release

echo "building fmiapi for windows"
cargo build --package ${tgt} --target x86_64-pc-windows-gnu --release

echo "building fmiapi for macos"
export PATH=/usr/osxcross/target/bin/:$PATH
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=/usr/osxcross/target/bin/x86_64-apple-darwin20.4-clang
cargo build --package ${tgt} --target x86_64-apple-darwin --release

echo "copying fmiapi into cli assets"
mkdir -p assets/auto_generated/binaries/x86_64-linux
mkdir -p assets/auto_generated/binaries/x86_64-windows
mkdir -p assets/auto_generated/binaries/x86_64-darwin
cp target/x86_64-unknown-linux-gnu/release/lib${tgt}.so assets/auto_generated/binaries/x86_64-linux/unifmu.so
cp target/x86_64-pc-windows-gnu/release/${tgt}.dll assets/auto_generated/binaries/x86_64-windows/unifmu.dll
cp target/x86_64-apple-darwin/release/lib${tgt}.dylib assets/auto_generated/binaries/x86_64-darwin/unifmu.dylib

# ------------------------------ schemas ------------------------------
echo "generating protobuf schemas for python, csharp, and java backends"
mkdir -p assets/auto_generated/fmi2
mkdir -p assets/auto_generated/fmi3
protoc -I=schemas --python_out=assets/auto_generated --csharp_out=assets/auto_generated --java_out=assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto
# ------------------------------ cli ------------------------------
tgt=unifmu
echo "installing dependencies for testing cli in linux"
apt install -qq -y python3-zmq python3-pip # Used by python backend
pip3 install protobuf==5.27.3 --break-system-packages # Used by python backend
pip3 install colorama coloredlogs toml --break-system-packages # Used by the private backend in distributed FMU
pip3 install fmpy --break-system-packages
echo "testing cli for linux"
cargo test --target x86_64-unknown-linux-gnu --release
echo "building cli for linux"
cargo build --package ${tgt} --target x86_64-unknown-linux-gnu --release

echo "building cli for windows"
cargo build --package ${tgt} --target x86_64-pc-windows-gnu --release

echo "building cli for macos"
cargo build --package ${tgt} --target x86_64-apple-darwin --release

# ------------------------------ compress for release ------------------------------

echo "querying for version number of unifmu via 'unifmu --version', the version is defined in cli/cargo.toml"
VER=$(./target/x86_64-unknown-linux-gnu/release/unifmu --version)
arrVER=(${VER//\ / })
VER=${arrVER[1]}
echo "detected version" ${VER}

echo "zipping linux cli"
t=x86_64-unknown-linux-gnu
zip -qj target/unifmu-${t}-${VER}.zip target/${t}/release/unifmu

echo "zipping windows cli"
t=x86_64-pc-windows-gnu
zip -qj target/unifmu-${t}-${VER}.zip target/${t}/release/unifmu.exe

echo "zipping macos cli"
t=x86_64-apple-darwin
zip -qj target/unifmu-${t}-${VER}.zip target/${t}/release/unifmu

