import json
import logging
import os
import sys
from argparse import ArgumentParser
from pathlib import Path

import matlab.engine
import zmq
from zmq import Socket

from matlab_interface import MatlabInterface
from unifmu.fmi2 import parse_model_description

# method encoding: strings match matlab files that get run for the corresponding method.
COMMAND_ENCODING = {
    # common
    0: "set_debug_logging",
    1: "setup_experiment",
    3: "enter_initialization_mode",
    4: "exit_initialization_mode",
    5: "terminate",
    6: "reset",
    7: "set_xxx",
    8: "get_xxx",
    9: "serialize",
    10: "deserialize",
    # 11: "get_directional_derivative",
    # model exchange
    # cosim
    # 12: "set_input_derivatives",
    # 13: "get_output_derivatives",
    14: "do_step",
    # 15: "cancel_step",
    # 16: "get_xxx_status",
}


def ensure_matlab_in_dir(cmd_str):
    file_name = f"{cmd_str}.m"
    if not os.path.exists(file_name):
        msg = f"Expected to find file {file_name} in directory {os.getcwd()}."
        logger.error(msg)
        raise ValueError(msg)


def ensure_matlab_files_exist():
    for code in COMMAND_ENCODING:
        if code != 7 and code != 8:
            cmd_str = COMMAND_ENCODING[code]
            ensure_matlab_in_dir(cmd_str)
        elif code == 7:
            ensure_matlab_in_dir("set_real")
            ensure_matlab_in_dir("set_boolean")
            ensure_matlab_in_dir("set_string")
            ensure_matlab_in_dir("set_integer")
        elif code == 8:
            ensure_matlab_in_dir("get_real")
            ensure_matlab_in_dir("get_boolean")
            ensure_matlab_in_dir("get_string")
            ensure_matlab_in_dir("get_integer")


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__file__)

    parser = ArgumentParser()
    parser.add_argument(
        "--handshake-endpoint",
        dest="handshake_endpoint",
        type=str,
        help="socket",
        required=True
    )
    args = parser.parse_args()

    # initializing message queue
    context = zmq.Context()
    handshake_socket: Socket = context.socket(zmq.PUSH)
    command_socket: Socket = context.socket(zmq.REP)
    handshake_socket.connect(f"{args.handshake_endpoint}")
    command_port = command_socket.bind_to_random_port("tcp://127.0.0.1")

    logger.info(f"cwd:{os.getcwd()}")

    handshake_info = {
        "serialization_format": "Pickle",
        "command_endpoint": command_socket.getsockopt(zmq.LAST_ENDPOINT).decode(),
    }

    handshake_json = json.dumps(handshake_info)
    handshake_socket.send_string(handshake_json)

    # Collect variable types, so that the correct matlab method is called for each type.
    # Expects to run from the resources directory.
    with open(Path.cwd().parent / "modelDescription.xml") as f:
        model_description_str = f.read()
        model_desc = parse_model_description(model_description_str)

    attr_to_type = {}
    for v in model_desc.modelVariables:
        attr_to_type[v.value_reference] = v.dataType.lower()

    # Move to the dir where the matlab code is
    os.chdir("matlabcode")

    # Matlab engine initialize
    # eng = matlab.engine.start_matlab('-desktop')
    eng = matlab.engine.start_matlab()
    eng.instantiate()

    # Check that matlab files are implemented in the cwd
    # Assumes we're in "matlabcode" dir.
    ensure_matlab_files_exist()

    # event loop
    while True:

        logger.info(f"slave waiting for command")

        kind, *args = command_socket.recv_pyobj()

        logger.info(f"received command of kind {kind} with args: {args}")

        if kind in COMMAND_ENCODING:
            command_str = COMMAND_ENCODING[kind]
            if kind == 7:
                vrefs = args[0]
                assert len(vrefs) > 0
                command_str = f"set_{attr_to_type[vrefs[0]]}"
            elif kind == 8:
                vrefs = args[0]
                assert len(vrefs) > 0
                command_str = f"get_{attr_to_type[vrefs[0]]}"

            # Invoke matlab file.
            logger.debug(f"invoking matlab function {command_str} with args {args}.")

            result = getattr(MatlabInterface, command_str)(*args, eng=eng)

            logger.info(f"returning value: {result}")
            command_socket.send_pyobj(result)
        elif kind == 2:
            logger.debug("freeing instance")

            eng.quit()
            eng = None

            command_socket.send_pyobj(None)
            sys.exit(0)
        else:
            logger.error(f"Unexpected command code: {kind}")
            raise ValueError(kind)
