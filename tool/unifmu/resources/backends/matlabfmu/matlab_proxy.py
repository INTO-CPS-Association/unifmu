import json
import logging
import os
import sys
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

import zmq

from matlabfmu import MatlabFMU
from fmi2 import Fmi2FMU
from unifmu.fmi2 import parse_model_description

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
    handshake_socket = context.socket(zmq.PUSH)
    command_socket = context.socket(zmq.REP)
    handshake_socket.connect(f"{args.handshake_endpoint}")
    command_port = command_socket.bind_to_random_port("tcp://127.0.0.1")

    logger.info(f"cwd:{os.getcwd()}")

    handshake_info = {
        "serialization_format": "Pickle",
        "command_endpoint": command_socket.getsockopt(zmq.LAST_ENDPOINT).decode(),
    }

    handshake_json = json.dumps(handshake_info)
    handshake_socket.send_string(handshake_json)

    # create slave object then use model description to create a mapping between fmi value references and attribute names of FMU

    reference_to_attr = {}
    attr_to_start = {}

    with open(Path.cwd().parent / "modelDescription.xml") as f:
        model_description_str = f.read()
        model_desc = parse_model_description(model_description_str)

        for v in model_desc.modelVariables:
            reference_to_attr[v.value_reference] = v.name

    os.chdir("matlabcode")

    slave: Fmi2FMU = MatlabFMU(reference_to_attr)

    # methods bound to a slave which returns status codes
    command_to_slave_methods = {
        # common
        0: slave.set_debug_logging,
        1: slave.setup_experiment,
        3: slave.enter_initialization_mode,
        4: slave.exit_initialization_mode,
        5: slave.terminate,
        6: slave.reset,
        7: slave.set_xxx,
        8: slave.get_xxx,
        9: slave.serialize,
        10: slave.deserialize,
        11: slave.get_directional_derivative,
        # model exchange
        # cosim
        12: slave.set_input_derivatives,
        13: slave.get_output_derivatives,
        14: slave.do_step,
        15: slave.cancel_step,
        16: slave.get_xxx_status,
    }

    # event loop
    while True:

        logger.info(f"slave waiting for command")

        kind, *args = command_socket.recv_pyobj()

        logger.info(f"received command of kind {kind} with args: {args}")

        if kind in command_to_slave_methods:
            result = command_to_slave_methods[kind](*args)
            logger.info(f"returning value: {result}")
            command_socket.send_pyobj(result)

        elif kind == 2:
            logger.debug("freeing instance")
            command_socket.send_pyobj(None)
            sys.exit(0)
