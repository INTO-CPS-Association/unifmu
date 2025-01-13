protoc -I=./schemas --python_out=./assets/auto_generated --csharp_out=./assets/auto_generated --java_out ./assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto
cargo build --package fmiapi --target x86_64-unknown-linux-gnu --release
cp ./target/x86_64-unknown-linux-gnu/release/libfmiapi.so ./assets/auto_generated/unifmu.so
cargo test
cargo run --bin unifmu --release -- generate python myfmu
cargo run --bin unifmu --release -- generate python myfmu.fmu --zipped
fmpy simulate myfmu.fmu

