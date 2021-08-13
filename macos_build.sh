PATH="$(pwd)/osxcross/target/bin:$PATH" \
cargo build --package unifmu --target x86_64-apple-darwin

# export CC=x86_64-apple-darwin20.4-clang
# export CXX=x86_64-apple-darwin20.4-clang++

