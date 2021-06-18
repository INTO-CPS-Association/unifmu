**This FMU was generated using UniFMU.
For general instructions on how to use the tool access the repository https://github.com/INTO-CPS-Association/unifmu**

An overview of the role of each file is provided in the tree below:

```python
📦model
 ┣ 📂binaries # binaries for linux, windows and macOS
 ┃ ┣ 📂darwin64
 ┃ ┃ ┗ 📜unifmu.dylib
 ┃ ┣ 📂linux64
 ┃ ┃ ┗ 📜unifmu.so
 ┃ ┗ 📂win64
 ┃ ┃ ┗ 📜unifmu.dll
 ┣ 📂resources
 ┃ ┣ 📂schemas # schemas for grpc backend
 ┃ ┃ ┣ 📜unifmu_fmi2_pb2.py
 ┃ ┃ ┗ 📜unifmu_fmi2_pb2_grpc.py
 ┃ ┣ 📜backend_grpc.py # grpc based backend
 ┃ ┣ 📜backend_schemaless_rpc.py # zmq based backend
 ┃ ┣ 📜fmi2.py # FMI related class definitions
 ┃ ┣ 📜launch.toml* "OS specific commands to bootstrap backend"
 ┃ ┗ 📜model.py* "implementation of FMU"
 ┗ 📜modelDescription.xml* "definition of inputs and outputs"
```

\* denotes files that should be modified by the user.

# Selecting Python interpreter

The `launch.toml` specifies a command that is used to start the selected backend using a specific python interpreter.

```toml
backend = "grpc"

[grpc]
linux = ["python3", "backend_grpc.py"]
macos = ["python3", "backend_grpc.py"]
windows = ["python", "backend_grpc.py"]

[zmq]
linux = ["python3", "backend_schemaless_rpc.py"]
macos = ["python3", "backend_schemaless_rpc.py"]
serialization_format = "Pickle"
windows = ["python", "backend_schemaless_rpc.py"]
```

By default the interpreter used on windows is `python` and `python3` on Linux and macOS.
**The executable must be in the systems path, otherwise the FMU can not be instantiated.**

Note that the command is invoked by the system without the use of a shell.
On some operating systems an name like `python3` may actually be an alias defined by the shell and not an executable that is in the system's path.
There are several solutions to this:

1. Add the python interpreter to the system's path.
2. Replace the alias, `python3`, with the absolute path to the interpreter.
   ```toml
   linux = ["somedir/python3","backend_grpc.py"]
   macos = ["somedir/python3","backend_grpc.py"]
   windows = ["C:\Program Files\Python39\python.exe", "backend_grpc.py"]
   ```
3. Launch the shell, then launch the interpreter using the alias.
   ```toml
   linux = ["sh", "python3", "backend_grpc.py"]
   macos = ["zsh", "python3", "backend_grpc.py"]
   windows = ["powershell", "python3", "backend_grpc.py"]
   ```

# How to modify the model?

The behavior of the model is defined by the `model.py` script, which provides an object oriented API for defining FMUs.
Specifically, to implement an FMU the methods such as the `do_step` or `enter_initialization_model` functions.
The python methods serve as the implementation of the FMU and are invoked whenever a call is made to the corresponding method in FMI's C-API.

```python
def do_step(self, current_time, step_size, no_step_prior):
    ...
    return Fmi2Status.ok

def enter_initialization_mode(self) -> int:
    ...
    return Fmi2Status.ok

```

For a complete list of function see the `Fmi2FMU` class defined in `fmi2.py`

# How to set and get variables?

By default the variables defined in the `modelDescription.xml` are bound to the attributes of the instance of the model in Python.

For instance, consider the a variable `real_a` defined in the `modelDescription.xml` file as shown below:

```xml
<ScalarVariable name="real_a" valueReference="0" variability="continuous" causality="input">
    <Real start="0.0" />
</ScalarVariable>
```

To set the value of the variable from within the model, the variable can be assigned to just like any other python attribute.
For example the following code sets `real_a` to zero:

```python
self.real_a = 0.0
```

It is also possible to define _getters_ and _setters_ for a variable in the model.
Using python's `property` decorator the `real_a` can be turned into a property as follows:

```python
@property
def real_a(self):
    return self._real_a

@real_a.setter
def real_a(self, value):
    self._real_a = value
```

The use of properties shown above, does not provide much advantage compared to representing the variable as a plain python attribute.
However, the use of _getter_ and _setters_ is useful for things such as:

- Validating values when setting variables
- Modelling cases of feedthrough, where a change in input variable causes an update to an output, despite a `fmi2DoStep` is invoked.

An alternative way to customize the mapping between the variables of the `modelDescription.xml` and the attributes of the python object, is to override the `get_xxx` and `set_xxx` defined in the `Fmi2FMU` class.
The default implementation of the two methods is what establishes the direct mapping between the names of variables declared in the `modelDescription.xml` file and the names of the python object.
Below is a _pseudo-code_ representation of the two methods is shown:

```python
def get_xxx(self, references):
    # convert to value references to attribute names, then call `getattr`
    ...
    return Fmi2Status.ok, values

def set_xxx(self, references, values):
    # convert to value references to attribute names, then call `setattr`
    ...
    return Fmi2Status.ok
```

Note that this translation requires that the `modelDescription.xml` is available during runtime for parsing.
Most FMU importing tools unzip the entire FMU archive to the same directory; resulting in the `modelDescription.xml` ending up in the parent folder of the `resources` directory.
However, the FMI specification does not require this to be the case.

If this is a problem for you, it can be addressed in several ways, two of which are:

- The `modelDescription.xml` could be copied to the resources directory.
- The parsing of `modelDescription.xml` could be replaced with manually defined map implemented in `set_xxx` and `get_xxx`

# Error handling

FMI uses error codes to indicate the success or failure of an operation, whereas Python typically handles errors using try-catch blocks.
Python FMUs generated by UniFMU use status codes to report the state of each operation performed.
Concretely, the FMI functions defined by the `Model` class, must return a status code indicating if the operation went well.
For example, consider the example of the `do_step` implementation shown below:

```python
# return is error -> error
def do_step(self, current_time, step_size, no_step_prior):

    return Fmi2Status.error

# return is error -> error
def do_step(self, current_time, step_size, no_step_prior):

    return Fmi2Status.error

```

Unrecognized or lack-of returns result in status code error:

```python
def do_step(self, current_time, step_size, no_step_prior):
    ...
    # no return

def do_step(self, current_time, step_size, no_step_prior):
    return ""
```

Additionally, any uncaught exception results in status code error:

```python
# uncaught exception -> error
def do_step(self, current_time, step_size, no_step_prior):

    raise Exception()

    return Fmi2Status.ok
```

In addition to the _ok_ and _error_ several other status codes exists.
The permitted status codes are defined in the `Fmi2Status` class:

```python
class Fmi2Status:
    """Represents the status of the FMU or the results of function calls.

    Values:
        * ok: all well
        * warning: an issue has arisen, but the computation can continue.
        * discard: an operation has resulted in invalid output, which must be discarded
        * error: an error has ocurred for this specific FMU instance.
        * fatal: an fatal error has ocurred which has corrupted ALL FMU instances.
        * pending: indicates that the FMu is doing work asynchronously, which can be retrived later.

    Notes:
        FMI section 2.1.3

    """
    ok = 0
    warning = 1
    discard = 2
    error = 3
    fatal = 4
    pending = 5
```

# Serialization and deserialization

FMUs generated by UniFMU provides a simplified API for serializing and deserializing an FMU's state.
A pair of methods `serialize` and `deserialize` are used to define how to convert the FMU's current state into a array of bytes and how to subsequently turn to restore the state of the FMU using the stream of bytes.

In python this can be achieved using the built in library `pickle`, which allows the vast majority of types in python to be serialized and deserialized with ease.
For instance, an example of how to implement the two methods is shown below:

```python
def serialize(self):
    bytes = pickle.dumps((self.real_a, self.real_b))
    return Fmi2Status.ok, bytes

def deserialize(self, bytes):
    (real_a, real_b) = pickle.loads(bytes)
    self.real_a = real_a
    self.real_b = real_b

    return Fmi2Status.ok
```

Here the example only shows two variables being serialized. However serialization mechanism will work any number of variables.
It is up to the implementer of the FMU to ensure that the variables being serialized are sufficient to restore the FMU.
Similarly, it is also the implementers decision on how to handle shared resources like files opened by an FMU.

# Testing the model

Remember the `model.py` is _plain_ python code, so not why test it in python, where you have good debugging tools?

A small test program can be written and placed in the `model.py` the slave as seen below:

```python
if __name__ == "__main__": # <--- ensures that test-code is not run if module is imported
    import numpy as np
    import matplotlib.pyplot as plt

    # create time stamps
    n_steps = 100
    ts, step_size = np.linspace(0, 10, n_steps, retstep=True)

    # create FMU
    generator = SineGenerator()
    generator.amplitude = 10
    generator.std = 1
    generator.setup_experiment(0)

    # outputs
    ys = np.zeros(n_steps)

    for idx, t in enumerate(ts):
        ys[idx] = generator.y
        generator.do_step(t, step_size, False)

    plt.plot(ts, ys)
    plt.ylabel("y(t)")
    plt.xlabel("t")
    plt.show()
```

To test the FMU simply run the script in the python interpreter:

```bash
python model.py
```

A more complex FMU may warrant multiple test cases, each testing a distinct piece of functionality. For these cases a proper testing framework like [pytest](https://docs.pytest.org/en/stable/) is recommended.

# How to manage runtime dependencies?

Python FMU generated by UniFMU requires the following python packages during runtime:

- grpcio
- protobuf
- zmq

These can be installed in the current environment using `pip`:

```bash
python -m pip install unifmu[python-backend]
```

Additionally, any library imported by the model will also need to be present during runtime.
For instance if the model uses `numpy` and `scipy`, these will also need to be installed in the environment running the FMU.

## Virtual Enviornments

There several options for managing python environments, such as:

- [venv](https://docs.python.org/3/library/venv.html) (part of standard library since 3.3)
- [conda](https://docs.conda.io/en/latest/) (third-party)

Bundling dependencies is not the core goal of UniFMU, rather we aim to provide a mechanism to easily integrate FMUs with other technologies such as virtual environments.

To illustrate this, consider the process of bootstrapping a virtual environment using `venv`.
First, create a new python environment:

```bash
python -m venv fmu_env
```

Next we activate the environment:

```bash
source fmu_env/bin/activate
```

Install UniFMU's python FMU runtime dependencies and dependencies used by the model:

```bash
python -m pip install unifmu[python-backend] numpy scipy
```

We store all requirements in a text file, conventionally named `requirements.txt`:

```bash
python -m pip freeze > requirements.txt
```

The contents of the `requirements.txt` lists all the dependencies installed in the environment.
It should look like something along the lines of:

```txt
grpcio==1.38.0
lxml==4.6.3
numpy==1.20.3
protobuf==3.17.3
pyzmq==22.1.0
scipy==1.6.3
six==1.16.0
toml==0.10.2
unifmu==0.0.2
```

Next, we want to modify the `launch.toml` to do the following:

1. Create a new virtual environment named _fmu_env_ if it does not already exist.
2. Activate the environment.
3. Install all missing dependencies from the `requirements.txt` in the environment.
4. Launch the model from the environment.
