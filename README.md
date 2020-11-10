![Build and update wrappers](https://github.com/INTO-CPS-Association/unifmu/workflows/Build%20and%20update%20wrappers/badge.svg)
# Universal Functional Mock-Up Unit (UniFMU)

A fundamental challenge for successfully applying 

## Why use UniFMU?
* Easy to integrate, uses simple message queue, ZMQ


## How does it work?

Recall, a fmu is an zip archive containing a static description of the models interface, `modelDescription.xml`, a set of platform shared object libraries defining the behavior of the model, and finally a set of option resource files that might be used during execution of the model.

To make this more concrete we consider the example of how python may be integrated using UniFMU, as shown in the *python_fmu* example.
Below is the file structure of a the concrete FMU:
```
python_fmu
├── binaries
│   ├── linux64
│   │   └── unifmu.so
│   └── win64
│       └── unifmu.dll
├── modelDescription.xml
└── resources
    ├── adder.py
    ├── fmi2.py
    ├── launch.py
    └── launch.toml
```

UniFMU provides a generic binary that can be dropped into any newly created FMUs by acting as a bridge between the FMI specification and interpreted languages.

![how it works](docs/_static/how_it_works.drawio.svg)

The first thing that happens during simulation is the creation of instances of the particular FMU, each referred to as a slave. 
UniFMU uses a simple configuration file `launch.toml`, located in the resources directory, to specify a command that is used to create new instances of the FMU.

Below is a configuration for starting a python based FMU, see `examples/python_fmu`:

``` toml
[command]
windows = [ "python", "launch.py" ]
linux = [ "python3.8", "launch.py" ]
macos = ["python3.8","launch.py"]

[timeout]
launch = 500
command = 500
```

For this specific launch.toml file the UniFMU starts a new process by invoking the specified command, in this case:
``` bash
python3.8 launch.py --handshake-endpoint "tcp://localhost:5000"
```

The  process reads the launch.py file located in the resource folder.
The newly started script two sockets, a handshake socket used to establish the initial connection with the wrapper, and a command socket used by the wrapper to pass commands and results between the wrapper and the slave.

``` python
# initializing message queue
context = zmq.Context()
handshake_socket = context.socket(zmq.PUSH)
command_socket = context.socket(zmq.REP)
handshake_socket.connect(f"{args.handshake_endpoint}")
command_port = command_socket.bind_to_random_port("tcp://127.0.0.1")
```

Following this the script awaits and executes commands sent to the slave:
``` python
 # event loop
    while True:

        logger.info(f"slave waiting for command")

        kind, *args = command_socket.recv_pyobj()

        logger.info(f"received command of kind {kind} with args: {args}")

        if kind in command_to_slave_methods:
            result = command_to_slave_methods[kind](*args)
            logger.info(f"returning value: {result}")
            command_socket.send_pyobj(result)

        elif kind == 9:

            command_socket.send_pyobj(Fmi2Status.ok)
            sys.exit(0)
```







## How do i use it?

The easiest way to install the tool is using pip:

``` bash
pip install unifmu
```


## Building and Running Tests

Building the project requires the following programs:
* python3
* cargo
* cmake

A utility script, `build.py`, is located in the root of the repository.

To build the and update the wrapper in the examples use:
``` bash
python build.py --update-wrapper
```

To run the C integration tests run:
``` bash
python build.py --test-c
```
