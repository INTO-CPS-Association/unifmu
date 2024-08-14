<!-- ![Build and update wrappers](https://github.com/INTO-CPS-Association/unifmu/workflows/Build%20and%20update%20wrappers/badge.svg) -->

# UniFMU - Universal Functional Mock-Up Units

- [UniFMU - Universal Functional Mock-Up Units](#unifmu---universal-functional-mock-up-units)
  - [Getting the tool](#getting-the-tool)
  - [Getting help](#getting-help)
  - [How to use the command line interface](#how-to-use-the-command-line-interface)
  - [Language specific documentation](#language-specific-documentation)
  - [Supported Features](#supported-features)
    - [FMI2](#fmi2)
    - [FMI3 (in progress)](#fmi3-in-progress)
  - [Building and deployment](#building-and-deployment)
    - [Building during development](#building-during-development)
    - [Building for deployment](#building-for-deployment)
    - [Troubleshooting](#troubleshooting)
      - [Errors while running the docker build](#errors-while-running-the-docker-build)
  - [Citing the tool](#citing-the-tool)


The [_Functional Mock-Up Interface_](https://fmi-standard.org/) _(FMI)_ defines an exchange format that allows models, referred to as _Functional Mock-Up Unit (FMU)_, to be shared between tools supporting the standard.
In general, an FMU must be implemented in a programming language that can produce binaries that can be called from C, such as C itself or C++.
While this allows efficient execution of a simulation, it is a significant limitation when prototyping models.

UniFMU is a command line tool that facilitates the implementation of FMUs in other popular languages that would otherwise not be able to produce C-compatible binaries.
It does this by providing a precompiled binary that is C-compatible, which then dispatches calls to the implementation of the model in the target language.

| Specification Version |  FMU Interface  |  Languages         |  Binaries                  |
| --------------------- | --------------- | ------------------ | -------------------------  |
| FMI3 (_in progress_)  | (Co-Simulation) | (Python, C#, Java) | (win64, linux64, darwin64) |
| FMI2                  |  Co-Simulation  |  Python, C#, Java  |  win64, linux64, darwin64  |
| FMI1                  |  x              |  x                 |  x                         |

Examples of generated FMUs can be found in the [unifmu_examples](https://github.com/INTO-CPS-Association/unifmu_examples) repo.

## Getting the tool

The tool can be downloaded from [releases](https://github.com/INTO-CPS-Association/unifmu/releases) tab of the repository.
It is a single executable that bundles all assets used during FMU generation as part of the binary.

## Getting help

The current responsible for the tool maintenance is [Claudio Gomes](https://clagms.github.io/contact/). Feel free to reach out for help.

## How to use the command line interface

To display the synopsis use the `--help` flag.

```
unifmu 0.0.8

Implement Functional Mock-up units (FMUs) in various source languages.

* Source:   https://github.com/INTO-CPS-Association/unifmu
* Examples: https://github.com/INTO-CPS-Association/unifmu_examples

USAGE:
    unifmu <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    generate     Create a new FMU using the specified source language
    help         Print this message or the help of the given subcommand(s)
```
The command uses _git-style_ subcommands, an example being `generate`.
Help for the individual commands can be inquired by appending the `--help` after the name of the subcommand.

```
Create a new FMU using the specified source language

USAGE:
    unifmu generate [OPTIONS] <LANGUAGE> <FMU_VERSION> <OUTPATH>

ARGS:
    <LANGUAGE>       Source language of the generated FMU [possible values: python, c-sharp, java]
    <FMU_VERSION>    Version of the FMI specification to target [possible values: fmi2, fmi3 (in progress)]
    <OUTPATH>        Output directory or name of the FMU archive if "--zipped" is passed

OPTIONS:
    -h, --help      Print help information
    -z, --zipped    Compress the generated FMU as a zip-archive and store with '.fmu' extension
```

The generate command can be used to create a new FMU:

```bash
unifmu generate python model
```

The command generates a _placeholder FMU_ implemented in the specific language.
For example the tree below shows the placeholder FMU generated when implementing an FMU in python using UniFMU:

```python
📦model
 ┣ 📂binaries
 ┃ ┣ 📂darwin64
 ┃ ┃ ┗ 📜unifmu.dylib
 ┃ ┣ 📂linux64
 ┃ ┃ ┗ 📜unifmu.so
 ┃ ┗ 📂win64
 ┃ ┃ ┗ 📜unifmu.dll
 ┣ 📂resources
 ┃ ┣ 📂schemas
 ┃ ┃ ┗ 📜fmi2_messages_pb2.py
 ┃ ┣ 📜backend.py
 ┃ ┣ 📜launch.toml
 ┃ ┣ 📜model.py
 ┃ ┗ 📜README.md
 ┗ 📜modelDescription.xml
```

## Language specific documentation

Like the file structure, the workflow for modifying FMUs varies depending on the implementation language.
Depending on the language a `README.md` is placed in the root of the generated FMU, which serves as documentation for the particular language.
For reference the `README.md` copied into Python FMUs looks like [README.md](https://github.com/INTO-CPS-Association/unifmu/blob/f921f645d420c236464330abef79de5ce76bdf6b/assets/python/fmi3/README.md).

## Supported Features

### FMI2

| Name                              | Supported | Notes |
| --------------------------------- | --------- | ----- |
| fmi2GetTypesPlatform              | ✓         |       |
| fmi2GetVersion                    | ✓         |       |
| fmi2SetDebugLogging               | x         |       |
| fmi2Instantiate                   | ✓         |       |
| fmi2FreeInstance                  | ✓         |       |
| fmi2SetupExperiment               | ✓         |       |
| fmi2EnterInitializationMode       | ✓         |       |
| fmi2ExitInitializationMode        | ✓         |       |
| fmi2Terminate                     | ✓         |       |
| fmi2Reset                         | ✓         |       |
| fmi2GetReal                       | ✓         |       |
| fmi2GetInteger                    | ✓         |       |
| fmi2GetBoolean                    | ✓         |       |
| fmi2GetString                     | ✓         |       |
| fmi2SetReal                       | ✓         |       |
| fmi2SetInteger                    | ✓         |       |
| fmi2SetBoolean                    | ✓         |       |
| fmi2SetString                     | ✓         |       |
| fmi2GetFMUstate                   | ✓         |       |
| fmi2SetFMUstate                   | ✓         |       |
| fmi2FreeFMUstate                  | ✓         |       |
| fmi2SerializedFMUstateSize        | ✓        |       |
| fmi2SerializeFMUstate             | ✓        |       |
| fmi2DeSerializeFMUstate           | ✓         |       |
| fmi2GetDirectionalDerivative      | x         |       |
| fmi2EnterEventMode                | x         |       |
| fmi2NewDiscreteStates             | x         |       |
| fmi2EnterContinuousTimeMode       | x         |       |
| fmi2CompletedIntegratorStep       | x         |       |
| fmi2SetTime                       | x         |       |
| fmi2SetContinuousStates           | x         |       |
| fmi2GetDerivatives                | x         |       |
| fmi2GetEventIndicators            | x         |       |
| fmi2GetContinuousStates           | x         |       |
| fmi2GetNominalsOfContinuousStates | x         |       |
| fmi2SetRealInputDerivatives       | x         |       |
| fmi2GetRealOutputDerivatives      | x         |       |
| fmi2DoStep                        | ✓         |       |
| fmi2CancelStep                    | x         |       |
| fmi2GetStatus                     | x         |       |
| fmi2GetRealStatus                 | x         |       |
| fmi2GetIntegerStatus              | x         |       |
| fmi2GetBooleanStatus              | x         |       |
| fmi2GetStringStatus               | x         |       |


### FMI3 (in progress)

| Name                                | Supported | Notes |
| ----------------------------------- | --------- | ----- |
| fmi3GetVersion                      | ✓         |       |
| fmi3SetDebugLogging                 | x         |       |
| fmi3InstantiateModelExchange        | x         |       |
| fmi3InstantiateCoSimulation         | ✓         |       |
| fmi3InstantiateScheduledExecution   | x         |       |
| fmi3FreeInstance                    | ✓         |       |
| fmi3EnterInitializationMode         | ✓         |       |
| fmi3ExitInitializationMode          | ✓         |       |
| fmi3EnterEventMode                  | x         |       |
| fmi3Terminate                       | ✓         |       |
| fmi3Reset                           | ✓         |       |
| fmi3GetFloat32                      | ✓         |       |
| fmi3GetFloat64                      | ✓         |       |
| fmi3GetInt8                         | ✓         |       |
| fmi3GetUInt8                        | ✓         |       |
| fmi3GetInt16                        | ✓         |       |
| fmi3GetUInt16                       | ✓         |       |
| fmi3GetInt32                        | ✓         |       |
| fmi3GetUInt32                       | ✓         |       |
| fmi3GetInt64                        | ✓         |       |
| fmi3GetUInt64                       | ✓         |       |
| fmi3GetBoolean                      | ✓         |       |
| fmi3GetString                       | ✓         |       |
| fmi3GetBinary                       | ✓         |       |
| fmi3GetClock                        | ✓         |       |
| fmi3SetFloat32                      | ✓         |       |
| fmi3SetFloat64                      | ✓         |       |
| fmi3SetInt8                         | ✓         |       |
| fmi3SetUInt8                        | ✓         |       |
| fmi3SetInt16                        | ✓         |       |
| fmi3SetUInt16                       | ✓         |       |
| fmi3SetInt32                        | ✓         |       |
| fmi3SetUInt32                       | ✓         |       |
| fmi3SetInt64                        | ✓         |       |
| fmi3SetUInt64                       | ✓         |       |
| fmi3SetBoolean                      | ✓         |       |
| fmi3SetString                       | ✓         |       |
| fmi3SetBinary                       | ✓         |       |
| fmi3SetClock                        | x         |       |
| fmi3GetNumberOfVariableDependencies | x         |       |
| fmi3GetVariableDependencies         | x         |       |
| fmi3GetFMUState                     | ✓         |       |
| fmi3SetFMUState                     | ✓         |       |
| fmi3FreeFMUState                    | ✓         |       |
| fmi3SerializedFMUStateSize          | ✓         |       |
| fmi3SerializeFMUState               | ✓         |       |
| fmi3DeserializeFMUState             | ✓         |       |
| fmi3GetDirectionalDerivative        | x         |       |
| fmi3GetAdjointDerivative            | x         |       |
| fmi3EnterConfigurationMode          | x         |       |
| fmi3ExitConfigurationMode           | x         |       |
| fmi3GetIntervalDecimal              | x         |       |
| fmi3GetIntervalFraction             | x         |       |
| fmi3GetShiftDecimal                 | x         |       |
| fmi3GetShiftFraction                | x         |       |
| fmi3SetIntervalDecimal              | x         |       |
| fmi3SetIntervalFraction             | x         |       |
| fmi3SetShiftDecimal                 | x         |       |
| fmi3SetShiftFraction                | x         |       |
| fmi3EvaluateDiscreteStates          | x         |       |
| fmi3UpdateDiscreteStates            | x         |       |
| fmi3EnterContinuousTimeMode         | x         |       |
| fmi3CompletedIntegratorStep         | x         |       |
| fmi3SetTime                         | x         |       |
| fmi3SetContinuousStates             | x         |       |
| fmi3GetContinuousStateDerivatives   | x         |       |
| fmi3GetEventIndicators              | x         |       |
| fmi3GetContinuousStates             | x         |       |
| fmi3GetNominalsOfContinuousStates   | x         |       |
| fmi3GetNumberOfEventIndicators      | x         |       |
| fmi3GetNumberOfContinuousStates     | x         |       |
| fmi3EnterStepMode                   | x         |       |
| fmi3GetOutputDerivatives            | x         |       |
| fmi3DoStep                          | x         |       |
| fmi3ActivateModelPartition          | x         |       |

## Building and deployment


### Building during development

Building for local machine (with Windows as the example, and PowerShell commands for reference). This is a good method to locally test if the program is running as it should. A reference script is provided for some of the instructions below: [test_local.sh](test_local.sh)

1. Make sure you have the following installed on your computer:
   - [rust](https://www.rust-lang.org/tools/install)
   - a [C-compiler and linker](https://visualstudio.microsoft.com/vs/features/cplusplus/)
   - [Python](https://www.python.org/) and [Java](https://openjdk.org/) that's compatible with [VDMCheck](https://github.com/INTO-CPS-Association/FMI-VDM-Model) to test the generated FMU.

2. Clone the `unifmu` repository.

3. Make the changes you want.

4. Install the rust toolchain for your operating systems, e.g. `rustup target add x86_64-pc-windows-msvc` (msvc is the microsoft C-compiler).

5. Build the FMU dll using `cargo build --target x86_64-pc-windows-msvc --release`. This should build the project for your operating system, and generate a the `fmiapi.dll` in the folder [target/x86_64-pc-windows-msvc/release](target/x86_64-pc-windows-msvc/release/). The dll contains the FMU headers' implementation.

6. Generate the content for the [./assets/auto_generated/](./assets/auto_generated/) folder, that the CLI is packaged with.
   1. Copy the generated dll into the assets folder (needed by the CLI):
      ```powershell
      Copy-Item -Force ./target/x86_64-pc-windows-msvc/release/fmiapi.dll ./assets/auto_generated/unifmu.dll
      ```
   2. Generate the protobuf schemas for python, csharp, and java backends:
      ```powershell
      protoc -I=schemas --python_out=assets/auto_generated --csharp_out=assets/auto_generated --java_out assets/auto_generated fmi2_messages.proto fmi3_messages.proto
      ```

7. Run the integration tests: `cargo test`

8. Compile the CLI and generate an FMU called `myfmu.fmu` using the newly compiled CLI:
    ```powershell
    cargo run --bin unifmu --release -- generate --zipped python myfmu.fmu fmi2
    ```

9. To test the FMU, we recommend:
   1. Installing [FMPy](https://github.com/CATIA-Systems/FMPy), and use it to simulate the FMU:
      ```powershell
      pip install fmpy[complete]
      fmpy --validate simulate myfmu.fmu --show-plot
      ```
    2. Use [VDMCheck](https://github.com/INTO-CPS-Association/FMI-VDM-Model) tool to validate the FMU:
        ```powershell
        java -jar ./test_dependencies/vdmcheck-1.1.3/vdmcheck2.jar myfmu.fmu
        ```
        If the FMU exported implements FMI 3.0, then use `vdmcheck3.jar`

### Building for deployment

This method should be followed when building the tool to be deployed for different OSs (windows, macos, linux). The cross compilation happens via Docker and the script can be read in [docker-build/build_all.sh](docker-build/build_all.sh)

1. You need to have gone through the steps in the previous instructions for the development build.

2. Have [docker](https://docs.docker.com/engine/install/) installed on your computer.

3. Build the docker image using `docker build -t unifmu-docker docker-build`. `unifmu-docker` is the name of the container, and `docker-build` is the directory where the Dockerfile is (assuming you are running this command from the root of the unifmu repository).

4. Build the unifmu project in docker using the following command:
    ```powershell
    docker run --name builder -it --rm -v <location_of_unifmu_repository_on_local_pc>:/workdir unifmu-docker ./docker-build/build_all.sh
    ```
    where `<location_of_unifmu_repository_on_local_pc>` should be replaced by the path of the unifmu repository location.
    For example:
    ```powershell
    docker run --name builder -it --rm -v .:/workdir unifmu-docker ./docker-build/build_all.sh
    ```
    This should generate three folders in the `target` directory on your local computer, one folder for each OS (windows, macos, linux).

### Troubleshooting

#### Errors while running the docker build

It's useful to open a terminal in the container and debug your way around:
```powershell
docker run --name builder -it --rm -v .:/workdir unifmu-docker bash
```

## Citing the tool

When citing the tool, please cite the following paper:

- Legaard, C. M., Tola, D., Schranz, T., Macedo, H. D., & Larsen, P. G. (2021). A Universal Mechanism for Implementing Functional Mock-up Units. In G. Wagner, F. Werner, T. I. Ören, & F. D. Rango (Eds.), Proceedings of the 11th International Conference on Simulation and Modeling Methodologies, Technologies and Applications, SIMULTECH 2021, Online Streaming, July 7-9, 2021 (pp. 121-129). SCITEPRESS Digital Library. https://doi.org/10.5220/0010577601210129

Bibtex:

```
@inproceedings{Legaard2021,
  title = {A Universal Mechanism for Implementing Functional Mock-up Units},
  booktitle = {11th {{International}} Conference on Simulation and Modeling Methodologies, Technologies and Applications},
  author = {Legaard, Christian M. and Tola, Daniella and Schranz, Thomas and Macedo, Hugo Daniel and Larsen, Peter Gorm},
  year = {2021},
  pages = {121--129},
  address = {{Virtual Event}},
  series = {{{SIMULTECH}} 2021}
}
```
