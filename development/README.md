# Developing UniFMU

[User guide](../README.md) | [FMI function support](../docs/fmi-support.md)

This guide covers building and changing UniFMU itself. To implement a model using
an existing release, follow the [user guide](../README.md#create-your-first-fmu)
and the [backend documentation](../README.md#use-other-languages-or-fmi3).

- [Repository layout](#repository-layout)
- [Native development build](#native-development-build)
- [Testing](#testing)
- [Cross-platform release build](#cross-platform-release-build)
- [Maintaining the build image](#maintaining-the-build-image)
- [Versioning and releases](#versioning-and-releases)
- [Troubleshooting](#troubleshooting)

## Repository layout

| Path | Purpose |
| --- | --- |
| `cli/` | FMU generator and integration tests |
| `fmiapi/` | Native FMI2/FMI3 runtime libraries |
| `unifmu_macros/` | Test-generation macros |
| `assets/` | Backend templates and resources embedded into the CLI |
| `assets/auto_generated/` | Generated protobuf bindings and compiled runtime libraries; ignored by Git |
| `schemas/` | Protocol Buffer message definitions |
| `docker-build/` | Cross-compilation image and build scripts |
| `scripts/` | Shared-version reader, release-version selection, and tests |
| `.github/workflows/` | CI build/test and release workflows |

## Native development build

Run commands from the repository root.

### Prerequisites

Install:

- [Rust and Cargo](https://www.rust-lang.org/tools/install), plus a native C compiler/linker.
- [Protocol Buffers compiler 27.3](https://github.com/protocolbuffers/protobuf/releases/tag/v27.3)
  (`protoc`) on your `PATH`.
- Python 3.10 or newer for the Python backend; Python 3.11 or newer for
  `scripts/version.py`, which uses `tomllib`.
- Java 21 JDK for Java backends and
  [VDMCheck](https://github.com/INTO-CPS-Association/FMI-VDM-Model).
- .NET SDK and the .NET 8 runtime for the C# backend. The release image uses
  the .NET 10 SDK alongside the .NET 8 runtime.

Use a Python virtual environment and install the test dependencies:

```bash
python3 -m pip install pyzmq protobuf==5.27.3 fmpy colorama coloredlogs toml
```

Keep the protobuf compiler and backend dependency versions consistent when
upgrading the messaging schema tooling.

### Build the native runtime

Build `fmiapi` before the CLI, which embeds its compiled libraries:

```bash
cargo build --package fmiapi --release
```

Copy the result into `assets/auto_generated/binaries/<arch>-<sys>/` as
`unifmu.<extension>`. The asset directory uses
[FMI3 platform tuples](https://fmi-standard.org/docs/3.0.2/#platform-tuple-definition)
even when the generated FMU targets FMI2.

| Host | Build result | Asset destination |
| --- | --- | --- |
| Linux x86-64 | `target/release/libfmiapi.so` | `assets/auto_generated/binaries/x86_64-linux/unifmu.so` |
| Linux ARM64 | `target/release/libfmiapi.so` | `assets/auto_generated/binaries/aarch64-linux/unifmu.so` |
| macOS Intel | `target/release/libfmiapi.dylib` | `assets/auto_generated/binaries/x86_64-darwin/unifmu.dylib` |
| macOS Apple Silicon | `target/release/libfmiapi.dylib` | `assets/auto_generated/binaries/aarch64-darwin/unifmu.dylib` |
| Windows x86-64 | `target/release/fmiapi.dll` | `assets/auto_generated/binaries/x86_64-windows/unifmu.dll` |

For example, on Apple Silicon:

```bash
mkdir -p assets/auto_generated/binaries/aarch64-darwin
cp target/release/libfmiapi.dylib assets/auto_generated/binaries/aarch64-darwin/unifmu.dylib
```

On Windows (PowerShell):

```powershell
New-Item -ItemType Directory -Force assets/auto_generated/binaries/x86_64-windows
Copy-Item -Force target/release/fmiapi.dll assets/auto_generated/binaries/x86_64-windows/unifmu.dll
```

For an explicit Rust target, install it with `rustup target add <target>` and
pass `--target <target>` to Cargo. The result then lives under
`target/<target>/release/`, not `target/release/`. Cross-compilation also needs a
matching linker/toolchain; installing a Rust target alone is not sufficient.

### Generate messaging bindings and build the CLI

Create `assets/auto_generated/fmi2` and `assets/auto_generated/fmi3` first
(`mkdir -p` on Linux/macOS, `New-Item -ItemType Directory -Force` on PowerShell).
Then generate the Python, C#, and Java bindings:

```bash
protoc -I=schemas --python_out=assets/auto_generated --csharp_out=assets/auto_generated --java_out=assets/auto_generated fmi2_messages.proto fmi3_messages.proto unifmu_handshake.proto
cargo build --package unifmu --release
cargo run --bin unifmu --release -- generate python myfmu fmi2
```

Rebuild and recopy `fmiapi` after changing the native runtime. Rebuild the CLI
after updating embedded assets. A generated FMU only contains libraries available
when its CLI is built; use the release build below for the full platform set.

## Testing

The integration tests generate FMUs and exercise them through FMPy, plus
VDMCheck validation. They require generated assets and the Python, Java, and .NET
runtime dependencies for the tested backends.

```bash
cargo test
```

The FMI2 logger regression tests use a small C callback to check the variadic
FMI2 logging ABI and preserve percent signs as literal message text. Run them
without generating FMUs:

```bash
cargo test --package fmiapi --lib fmi2::fmi2_logger::tests
```

Local backends have stdout and stderr drained continuously, including between
FMI calls. The combined output is relayed to the importer's stdout as it arrives
(subject to the backend's own buffering). Unexpected-exit diagnostics retain the
last 64 KiB of combined output and indicate when earlier output was truncated.
This applies to both FMI2 and FMI3. Test pipe draining and diagnostic capture
without Python or generated FMUs:

```bash
cargo test --package fmiapi --lib common::dispatcher::backend_subprocess::tests
```

For version-selection and CLI-version tests without running simulations:

```bash
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo test --package unifmu --bin unifmu
```

The Python command above requires Python 3.11 or newer.

To try a generated archive manually:

```bash
cargo run --bin unifmu --release -- generate python myfmu.fmu fmi2 --zipped
fmpy simulate myfmu.fmu --show-plot
```

The validation tools are under `test_dependencies/`. Use `vdmcheck2.jar` for
FMI2 and `vdmcheck3.jar` for FMI3 in the corresponding VDMCheck distribution.
See [FMI function support](../docs/fmi-support.md) for the documented interface.

## Cross-platform release build

The [Dockerfile](../docker-build/Dockerfile) pins Ubuntu 24.04 and includes Java
21, the .NET 10 SDK, the .NET 8 runtime, and a Python virtual environment with test
dependencies. Both AMD64 and ARM64 hosts build the same five targets:

| Platform | Rust target |
| --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Windows x86-64 | `x86_64-pc-windows-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

From the repository root, build a host-native image and run the release script:

```bash
docker build -t unifmu-build:local docker-build
docker run --rm -v "$(pwd):/workdir" unifmu-build:local ./docker-build/build_all.sh
```

In PowerShell, use `-v "${PWD}:/workdir"` for the volume mount.
The [release script](../docker-build/build_all.sh):

1. Reads the shared workspace version and compiles all five native libraries.
2. Uses osxcross's Linux-compatible `lipo` to merge the Intel and ARM64 macOS
   libraries and verify both slices.
3. Generates protobuf bindings and runs tests using the host-native Linux target.
4. Builds all five CLIs and checks that the host CLI reports the expected version.
5. Writes five `target/unifmu-<target>-<version>.zip` archives.

Each release CLI embeds all five single-architecture libraries and the universal
macOS library at `assets/auto_generated/binaries/universal-darwin/unifmu.dylib`.
FMI2 generation places the universal library at `binaries/darwin64/unifmu.dylib`
and omits a separate ARM64 macOS folder. This applies to local and distributed
proxy FMUs. FMI3 keeps separate `x86_64-darwin` and `aarch64-darwin` libraries.
Development builds without the universal asset keep architecture-specific macOS
packaging.

macOS cross-compilation uses the macOS 11.3 SDK with a macOS 11 deployment target;
no post-build step on a Mac is needed. Linux builds use Ubuntu 24.04's glibc
baseline. Raising the Linux build-image version can raise runtime requirements:
check compatibility on the oldest supported Linux system before changing it.
FMI2 ARM64 Linux libraries use `aarch64-linux`, which requires importer support.

## Maintaining the build image

To build and push both host architectures to `lausdahl/unifmu-build:latest`:

```bash
docker login
./docker-build/build_and_push_image.sh
```

You need push access to that Docker Hub repository. The script creates or reuses
the `unifmu-multiarch` Buildx builder with the `docker-container` driver, avoiding
the default driver's multi-platform limitation. It does not publish UniFMU releases.

CI calls [obtain_build_image.sh](../docker-build/obtain_build_image.sh):

```bash
./docker-build/obtain_build_image.sh
docker run --rm -v "$(pwd):/workdir" unifmu-build:latest ./docker-build/build_all.sh
```

The obtain script pulls the published image and tags it locally as
`unifmu-build:latest`. If pulling fails, it reports the failure and builds an
AMD64 image locally. It does not push an image from CI.
Republish the image after changing its Dockerfile; pulling `latest` does not
rebuild it from the checked-out source.

## Versioning and releases

The version is set once in the root [Cargo.toml](../Cargo.toml), under
`[workspace.package]`. All Rust crates inherit it. `unifmu --version` (or `-V`)
reports it, and the CLI logs it at startup. The FMI2 and FMI3 runtimes report
their compiled UniFMU version through the importer's logger when logging is
enabled. Their FMI `GetVersion` functions still return the **FMI specification
version**, not the UniFMU version.

To release, run **Actions > Release > Run workflow** on the desired branch:

| Input | Behavior |
| --- | --- |
| Release version | Optional stable version such as `0.14.1` or `v0.14.1`; blank increments the patch from the highest workspace version or stable Git tag |
| Release automatically | Unchecked by default: create a draft for review. Check to publish automatically after a successful build |

For example, the default after `0.14.0` is `0.14.1`. An explicit version must not
precede the workspace version and must exceed existing stable tags. New tags
always use `vMAJOR.MINOR.PATCH`; existing tags and releases are never overwritten.

The [release workflow](../.github/workflows/release.yml) commits the selected
version, tags that commit, builds and tests all five targets, and creates a draft
with generated release notes and the five CLI ZIPs. It then fast-forwards the
selected branch to the version commit, and publishes only if requested.

The workflow needs `contents: write` and permission to update the selected
branch. Branch divergence or branch protection that prevents the update fails
the release; it never force-pushes.

On failure or cancellation, cleanup removes the new release and tag after
checking that the tag still points to this run's commit. Do not manually publish
the draft while its workflow is running. If publication fails after the branch
update, the version commit remains: cleanup does not rewind shared history.

## Troubleshooting

For build-image problems, open an interactive container:

```bash
docker run -it --rm -v "$(pwd):/workdir" unifmu-build:local bash
```

On Windows, ensure shell scripts use LF line endings, not CRLF. Missing backend
dependencies can cause integration tests to fail even when Rust compilation
succeeds; consult the [backend guides](../README.md#use-other-languages-or-fmi3).
