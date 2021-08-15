#!/bin/bash
# ------------------------------ fmi2api ------------------------------
tgt=fmi2api
echo "building fmi2api for linux"
cargo build --package ${tgt} --target x86_64-unknown-linux-gnu --release

echo "building fmi2api for windows"
cargo build --package ${tgt} --target x86_64-pc-windows-gnu --release

echo "building fmi2api for macos"
export PATH=/usr/osxcross/target/bin/:$PATH
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=/usr/osxcross/target/bin/x86_64-apple-darwin20.4-clang
cargo build --package ${tgt} --target x86_64-apple-darwin --release

echo "copying fmi2api into cli assets"
cp ./target/x86_64-unknown-linux-gnu/release/lib${tgt}.so ./assets/auto_generated/unifmu.so
cp ./target/x86_64-pc-windows-gnu/release/${tgt}.dll ./assets/auto_generated/unifmu.dll
cp ./target/x86_64-apple-darwin/release/lib${tgt}.dylib ./assets/auto_generated/unifmu.dylib

# ------------------------------ schemas ------------------------------
echo "generating protobuf schemas for python and csharp backends"
protoc -I=./schemas --python_out=./assets/auto_generated --csharp_out=./assets/auto_generated unifmu_fmi2.proto

# ------------------------------ cli ------------------------------
tgt=unifmu
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

