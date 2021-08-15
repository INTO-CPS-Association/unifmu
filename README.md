<!-- ![Build and update wrappers](https://github.com/INTO-CPS-Association/unifmu/workflows/Build%20and%20update%20wrappers/badge.svg) -->

# UniFMU - Universal Functional Mock-Up Units

The [_Functional Mock-Up Interface_](https://fmi-standard.org/) _(FMI)_ defines an exchange format that allows models, referred to as _Functional Mock-Up Unit (FMU)_, to be shared between tools supporting the standard.

Traditionally, an FMU must be implemented in a programming language that is compatible with C's binary interface such as C itself or C++.

UniFMU is a command line tool that facilitates the implementation of FMUs in languages in several languages, such as:

- Python
- C#

This is made possible by providing a generic binary that dispatches calls to the users implementation using _remote procedure call_ _(RPC)_.

## Installing the tool

The current and revious versions of the tool can be downloaded from the releases tab of the repository.

For convenience the tool can be copied to a directory that is in the systems path such as `/usr/bin/` for most Linux distributions.

## How to use the command line interface?

To display the synopsis use the `--help` flag.

```
UniFMU 0.0.4
Implement 'Functional Mock-up units' (FMUs) in various source languages.

USAGE:
    unifmu.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    generate    Create a new FMU using the specified source language
    help        Prints this message or the help of the given subcommand(s)
    validate

```

The command uses _git-style_ subcommands, an example being `generate`.
Help for the individual commands can be inquired by appending the `--help` after the name of the subcommand.

```
Create a new FMU using the specified source language

USAGE:
    unifmu.exe generate [FLAGS] <language> <outpath>

FLAGS:
    -d, --dockerize    Configure the generated model to deploy and execute code inside a 'Docker' container
    -h, --help         Prints help information
    -V, --version      Prints version information
    -z, --zipped       Compress the generated FMU as a zip-archive and store with '.fmu' extension

ARGS:
    <language>    Source language of the generated FMU [possible values: Python, CSharp, Matlab, Java]
    <outpath>
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
 ┃ ┃ ┗ 📜unifmu_fmi2_pb2.py
 ┃ ┣ 📜backend.py
 ┃ ┣ 📜launch.toml
 ┃ ┣ 📜model.py
 ┃ ┗ 📜README.md
 ┗ 📜modelDescription.xml
```

Like the file structure, the workflow for modifying FMUs varies depending on the implementation language.
Depending on the language a `README.md` is placed in the root of the generated FMU, which serves as documentation for the particular language.
For reference the `README.md` copied into Python FMUs looks like [README.md](tool/unifmu/resources/backends/python/README.md).

## Building and Testing

Build the cross compilation image from the dockerfile stored in `docker-build` folder:

```
docker build -t unifmu-build docker-build
```

**Note: This process may take a long time 10-30 minutes, but must only be done once.**

Start a container with the name `builder` from the cross-compilation image `unifmu-build`:

```bash
docker run --name builder -it -v $(pwd):/workdir unifmu-build  # bash
docker run --name builder -it -v %cd%:/workdir unifmu-build   # windows cmd
```

Sharing the source between the host and container, `unifmu-build`, is done using a volume as indicated by the `$(pwd):/workdir`.

**Note: On windows you may have to enable the use of shared folders through the dockers interface, otherwise the container fails to start.**

To build the code invoke the script `docker-build/build_all.sh` in the `workdir` of the container.
This generates and copies all relevant build artifacts into the `assets/auto_generated` directory:

```
📦auto_generated
 ┣ 📜.gitkeep
 ┣ 📜unifmu.dll
 ┣ 📜unifmu.dylib
 ┣ 📜unifmu.so
 ┣ 📜UnifmuFmi2.cs
 ┗ 📜unifmu_fmi2_pb2.py
```

Following this the cli is compiled for each platform, including the assets that were just compiled.
The final standalone executables can be found in the target folder, under the host tripple:

- linux: unifmu-x86_64-unknown-linux-gnu-0.0.4.zip
- windows: unifmu-x86_64-pc-windows-gnu-0.0.4.zip
- macOS: unifmu-x86_64-apple-darwin-0.0.4.zip

**Note: The executable for any platform embeds implementations of the FMI api for all other platforms. In other words the windows executable can generate FMUs that run on all other platforms.**

## Citing the tool

When citing the tool, please cite the following paper:

- Legaard, Christian M., Daniella Tola, Thomas Schranz, Hugo Daniel Macedo, and Peter Gorm Larsen. “A Universal Mechanism for Implementing Functional Mock-up Units,” to appear. SIMULTECH 2021. Virtual Event, 2021.

Bibtex:

```
@inproceedings{Legaard2021,
  title = {A Universal Mechanism for Implementing Functional Mock-up Units},
  booktitle = {11th {{International}} Conference on Simulation and Modeling Methodologies, Technologies and Applications},
  author = {Legaard, Christian M. and Tola, Daniella and Schranz, Thomas and Macedo, Hugo Daniel and Larsen, Peter Gorm},
  year = {2021},
  pages = {to appear},
  address = {{Virtual Event}},
  series = {{{SIMULTECH}} 2021}
}
```
