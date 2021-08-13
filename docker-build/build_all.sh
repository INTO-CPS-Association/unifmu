cargo build --release

cargo build --target x86_64-pc-windows-gnu --release

export PATH=/usr/osxcross/target/bin/:$PATH
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=/usr/osxcross/target/bin/x86_64-apple-darwin20.4-clang
cargo build --target x86_64-apple-darwin --release
