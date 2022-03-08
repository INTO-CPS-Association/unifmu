protoc -I schemas --python_out=assets\auto_generated --csharp_out=assets\auto_generated --java_out assets\auto_generated fmi2_messages.proto fmi3_messages.proto
cargo build --package fmiapi --target x86_64-pc-windows-gnu --release
Copy-Item ./target/x86_64-pc-windows-gnu/release/fmiapi.dll ./assets/auto_generated/unifmu.dll
# cargo run --bin unifmu --release -- generate python fmi3 myfmu
# cargo run --bin unifmu --release -- validate myfmu
cargo run --bin unifmu --release -- generate python fmi3 myfmu.fmu --zipped
fmpy simulate myfmu.fmu
rm myfmu.fmu
