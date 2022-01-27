<!-- ![Build and update wrappers](https://github.com/INTO-CPS-Association/unifmu/workflows/Build%20and%20update%20wrappers/badge.svg) -->

# UniFMU - Universal Functional Mock-Up Units

The [_Functional Mock-Up Interface_](https://fmi-standard.org/) _(FMI)_ defines an exchange format that allows models, referred to as _Functional Mock-Up Unit (FMU)_, to be shared between tools supporting the standard.
In general, an FMU must be implemented in a programming language that can produce binaries that can be called from C, such as C itself or C++.
While this allows efficient execution of a simulation, it is a significant limitation when prototyping models.

UniFMU is a command line tool that facilitates the implementation of FMUs in other popular languages that would otherwise not be able to produce C-compatible binaries.
It does this by providing a precompiled binary that is C-compatible, which then dispatches calls to the implementation of the model in the target language.

| Specification Version | FMU Types    | Languages  |
| --------------------- | ------------ | ---------- |
| FMI3                  |              |            |
| FMI2                  | cosimulation | Python, C# |
| FMI1                  |              |            |

Examples of generated FMUs can be found in the [unifmu_examples](https://github.com/INTO-CPS-Association/unifmu_examples) repo.

## Getting the tool

The tool can be downloaded from [releases](https://github.com/INTO-CPS-Association/unifmu/releases) tab of the repository.
It is a single executable that bundles all assets used during FMU generation as part of the binary.

## How to use the command line interface?

To display the synopsis use the `--help` flag.

```
UniFMU 0.0.6
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
For reference the `README.md` copied into Python FMUs looks like [README.md](assets/python/README.md).

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
