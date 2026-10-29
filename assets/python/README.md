**This FMU was generated using UniFMU.
For general instructions on how to use the tool access the repository https://github.com/INTO-CPS-Association/unifmu**

# Implementing the model

The `resources/model.py` file defines the functional relationship between inputs and outputs of the FMU.

## Declaring inputs and outputs

### Annotation-based FMI2 models

For a local Python FMI2 project, first run `unifmu generate python my_model fmi2`
without `--zipped`. Keep the resulting directory as your editable project.
The generated `resources/model.py` already uses annotations; modify its
declarations and behavior rather than generating a fresh project for every edit.
Use Python 3.10 or newer and install `resources/requirements.txt` in your
activated virtual environment.

For example, declarations inside the generated `Model` can describe a real input
and a calculated output:

```python
from typing import Annotated
from fmi2_model_description import (
    ScalarVariable, RealType, Causality, Variability, Initial,
    fmi_model, build_reference_to_attribute_map,
)


@fmi_model("Your Name")
class Model:
    a: Annotated[float, ScalarVariable(
        causality=Causality.INPUT,
        variability=Variability.CONTINUOUS,
        variable_type=RealType(start=0.0),
    )]
    b: Annotated[float, ScalarVariable(
        causality=Causality.OUTPUT,
        variability=Variability.CONTINUOUS,
        initial=Initial.CALCULATED,
        variable_type=RealType(),
        dependencies=["a"],
    )]
```

This is a declaration example, not a complete replacement for the generated
class. Retain its FMI lifecycle methods, logging callback, and state handling,
adapting them to your variables. Initialize `self.a` and `self.b`, calculate
`self.b` from `self.a` in your model behavior, and retain:

```python
self.reference_to_attribute = build_reference_to_attribute_map(self)
```

Variable names default to the Python attribute names. Numeric value references
are assigned in annotation order, starting at zero; both XML generation and the
runtime mapping use that order. `IntegerType`, `BooleanType`, and `StringType`
are also available. Annotations describe the FMI interface; they do not
initialize Python attributes or implement calculations.

From the generated project's `resources` directory, run:

```bash
python3 main.py --model-description
python3 main.py --pack
```

The first command overwrites `../modelDescription.xml` using `Model(None)`.
The second creates `../my_model.fmu` when the project directory is named
`my_model`. For a custom location, use `python3 main.py --pack --output /path/to/my_model.fmu`.
Relative output paths are resolved from your current working directory.
On Windows, use `python` if that is your interpreter command.

**Regenerate the description before packing whenever you change declarations.**
`--pack` uses an existing XML unchanged and generates one from annotations only
if it is missing. Run `--model-description` and `--pack` separately; passing both
flags only performs description generation. XML regeneration replaces manual XML
edits, so preserve any custom metadata separately if needed.

Packing archives the existing binaries and resources; it does not compile
Python, rebuild native libraries, or install dependencies. Remove unwanted
cache/build files from `resources` before packing. This annotation workflow is
for local FMI2 projects; FMI3 and distributed projects do not use this generator.

### Manually maintained model descriptions

By default, each input, output or parameter declared in the `modelDescription.xml` file is represented as attributes on the instance of the `Model` class.
For instance if a variable `a` is declared in the `modelDescription.xml` file, it an attribute of the same name should be declared in the `Model` class:

```xml
<ScalarVariable name="a" valueReference="0" variability="continuous" causality="input">
    <Real start="0.0" />
</ScalarVariable>
```

```python
def __init__(self) -> None:
    self.a = 0.0

    self.reference_to_attribute = {
        0: "a",
    }
```

The FMI C-API uses numerical indices rather than names to which variables to read or write to.
As such a mapping between the indices declared by the `valueReference` attribute of the xml and the attributes must be defined.
By default the mapping between a value reference and its corresponding Python attribute is defined by adding an entry to the `reference_to_attributes` variable of the `Model` class.

For FMI3 FMUs, we provide extra dictionaries in the template for `clocked_variables`, `parameters`, `tunable_parameters`, and `tunable_structural_parameters`. The logic follows the same principles as for FMI2.

## Defining the behavior

The `Model` class declares several methods that can be used to define the behavior of the FMU.
Methods prefixed with `fmi2` or `fmi3` mirror the methods declared in the C-API defined by the FMI specification.

For instance, in FMI2, to update an output `b` to be twice the value of `a` the `fmi2DoStep` method could be defined as:

```python
def fmi2DoStep(self, current_time, step_size, no_step_prior):
    self.b = self.a * 2
    return Fmi2Status.ok
```

The logic for FMI3 follows the same principles as for FMI2, but it uses the FMI3 functions and datatypes instead.

# Testing and debugging the model

The `model.py` is _plain_ Python code, which means we can test the model using test cases and debugging tools.
A small test program can be written and placed in the `model.py` as seen below:

```python
if __name__ == "__main__":
    m = Model()

    assert m.a == 0.0
    assert m.b == 0.0

    m.a = 1.0
    m.fmiDoStep(0.0, 1.0, False)

    assert m.b == 2.0
```

The program can be executed in your IDE or from the command line by running the `resources/model.py` script.


# Runtime dependencies

The environment that invokes the Python code must provide all the dependencies, otherwise the simulation will fail when instantiating or simulation the model.
For instance, if the `resources/model.py` imports a third-party package such as `numpy`

```python
import numpy as np
```

this must also be available to the Python interpreter specified by the `launch.toml` file, in this case the system's `python3` interpreter:

```toml
linux = ["python3", "backend.py"]
```

One way to address a missing dependency is to install using package manager such as `pip`

```
python3 -m pip install numpy
```

**Any Python FMU generated UniFMU requires the `protobuf` package.
The easiest way to install this is using pip:**

```
python3 -m pip install protobuf==7.36.2
```

The correct version can be seen at the top of python files generated by the protocol buffer compiler. Example: fmi2_messages_pb2.py

**REMARK:** We suggest using a virtual environment for the Python runtime dependencies instead of the system-wide installation to avoid any potential conflicts with other libraries. The base dependencies are provided within the file `requirements.txt`.
Before importing the Python FMUs generated with UniFMU create and activate a virtual environment, and then, install the requirements as follows:
**Linux**
``` 
python3 -m venv venv
. venv/bin/activate
pip install -r requirements.txt
```
**Windows**
``` 
python -m venv venv
source venv/Scripts/activate
pip install -r requirements.txt
```

Install any other dependencies required in your model within the virtual environment before importing the FMU.

# Compilation
In case you want to compile your Python FMU to avoid dependency issues and hide your white-box model to some degree, you can make use of the resources within the `compilation_resources` folder.
We provide scripts for shell (Linux/macOS) and batch (Windows) that install `pyinstaller` and compile your Python FMU into an executable. The scripts also take care of reconfiguring the launching process of the FMU.

To execute the scripts, use the following commands from within the `resources` folder:
**Linux**
``` 
chmod +x compilation_resources/unifmu_pyinstaller_build_script.sh
./compilation_resources/unifmu_pyinstaller_build_script.sh
```

**Windows (On Powershell)**
``` 
.\compilation_resources\unifmu_pyinstaller_build_script.bat
```
On Windows, you can also double-click on the `.bat` script to execute it.

After the script has successfully compiled your Python FMU, the new compiled FMU will be available in the parent folder of your FMU as follows:

```
📂parent_folder
 ┣ 📂your_python_fmu_model(uncompressed) --> Execute the script here.
 ┣ 📦your_python_fmu_model.fmu
 ┗ 📦your_python_fmu_model_compiled.fmu --> The resulting compiled FMU.
```

Notice that the compiled FMU won't contain any explicit files from the original white-box FMU.
In case you want to do some changes to your model, you need to do these on the original white-box Python FMU and rerun the compilation script.

**Note for macOS users on ARM64 platforms:**
Currently, the default script fails to successfully compile the Python application on macOS on ARM64 platforms since PyInstaller struggles with setting the correct target architecture with default parameters.
To workaround this problem, use the flag `target_arch=arm64` in the specification generated by PyInstaller (`resources/main.spec`) in the compiled FMU (with suffix `_compiled`).
First, unzip the FMU, and then, reuse the resulting `main.spec` updating the `Analysis` component as follows:

```
...
a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
    target_arch=arm64, # Add this line
)
...
```

Then, copy the updated `main.spec` file into the original FMU within `resources/`. The compilation should work by calling `pyinstaller main.spec` from the terminal. This will build the compiled application within the original FMU.
For a clean compilation, update the file `resources/compilation_resources/unifmu_pyinstaller_build_script.sh` by changing the line
```
"$EXECUTABLE_NAME" main.py
```
to
```
"$EXECUTABLE_NAME" main.spec
```

and execute the script as normal with:
``` 
chmod +x compilation_resources/unifmu_pyinstaller_build_script.sh
./compilation_resources/unifmu_pyinstaller_build_script.sh
```

**Disclaimer:**
- This feature relies on [PyInstaller](https://pyinstaller.org/en/stable/) and its support is out of our scope. We cannot guarantee that this feature will work with complex FMU models, especially with several dependencies; therefore, we recommend testing the FMUs incrementally if this feature is to be used. For problems with PyInstaller, please visit their documentation page.
- Be aware that the compiled FMU is not Intellectual-Property-protected since the generated bytecode can be converted back into equivalent (not necessarily identical) source code.

# File structure

An overview of the role of each file is provided in the tree below:

```python
📦model
 ┣ 📂binaries
 ┃ ┣ 📂darwin64
 ┃ ┃ ┗ 📜unifmu.dylib       # binary for macOS
 ┃ ┣ 📂linux64
 ┃ ┃ ┗ 📜unifmu.so          # binary for Linux
 ┃ ┗ 📂win64
 ┃ ┃ ┗ 📜unifmu.dll         # binary For Windows
 ┣ 📂resources
 ┃ ┣ 📂compilation_resources # scripts and resources for compiling with pyinstaller
 ┃ ┃ ┃ 📜launch_with_pyinstaller.toml
 ┃ ┃ ┃ 📜unifmu_pyinstaller_build_script.bat
 ┃ ┃ ┗ 📜unifmu_pyinstaller_build_script.sh
 ┃ ┣ 📂schemas
 ┃ ┃ ┗ 📜fmix_messages_pbX.py # schema defining structure of messages sent over RPC. x denotes the version: either 2 for FMI2 or 3 for FMI3
 ┃ ┃ ┗ 📜unifmu_handshake_pbX.py # UniFMU handshaking mechanism for backends and dispatcher.
 ┃ ┣ 📜abstract_backend.py  # receives messages and dispatched function calls to "model.py"
 ┃ ┣ 📜backend.py           # receives messages and dispatched function calls to "model.py"
 ┃ ┣ 📜launch.toml          # specifies command used to start FMU
 ┃ ┣ 📜main.py              # entrypoint for the FMU backend
 ┃ ┣ 📜model.py*            # implementation of FMU
 ┃ ┣ 📜README.md            # This README.md file
 ┃ ┗ 📜requirements.txt     # Base dependencies to run UniFMU templates
 ┗ 📜modelDescription.xml*  # definition of inputs and outputs
```

\* denotes files that would typically be modified by the implementor of the FMU
