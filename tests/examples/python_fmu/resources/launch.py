import json
import logging
import sys
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

import zmq

from fmi2 import Fmi2Status
from adder import Adder


def get_slave_instance():
    return Adder()


if __name__ == "__main__":

    logging.basicConfig(level=logging.WARNING)
    logger = logging.getLogger(__file__)

    parser = ArgumentParser()
    parser.add_argument(
        "--handshake-endpoint",
        dest="handshake_endpoint",
        type=str,
        help="socket",
        required=True,
    )
    args = parser.parse_args()

    # initializing message queue
    context = zmq.Context()
    context.setsockopt(zmq.LINGER, -1)
    handshake_socket = context.socket(zmq.PUSH)
    command_socket = context.socket(zmq.REP)

    handshake_socket.connect(f"{args.handshake_endpoint}")

    command_port = command_socket.bind_to_random_port("tcp://127.0.0.1")

    handshake_info = {
        "serialization_format": "Pickle",
        "command_endpoint": command_socket.getsockopt(zmq.LAST_ENDPOINT).decode(),
    }

    handshake_json = json.dumps(handshake_info)
    handshake_socket.send_string(handshake_json)

    # create slave object then use model description to create a mapping between fmi value references and attribute names of FMU
    slave = get_slave_instance()

    reference_to_attr = {}
    with open(Path.cwd().parent / "modelDescription.xml") as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])
                              ] = v.attrib["name"]

    # -------- getter and setter functions ---------

    def get_xxx(references):
        attributes = [reference_to_attr[vref] for vref in references]
        values = [getattr(slave, a) for a in attributes]
        return values

    def set_xxx(references, values):
        attributes = [reference_to_attr[vref] for vref in references]
        for a, v in zip(attributes, values):
            setattr(slave, a, v)
        return Fmi2Status.ok

    def free_instance():
        logger.debug("freeing instance")

    # methods bound to a slave which returns status codes
    command_to_slave_methods = {
        0: slave.set_debug_logging,
        1: slave.setup_experiment,
        2: slave.enter_initialization_mode,
        3: slave.exit_initialization_mode,
        4: slave.terminate,
        5: slave.reset,
        6: set_xxx,
        7: get_xxx,
        8: slave.do_step,
    }

    # commands that which are not bound to a
    command_to_free_function = {9: free_instance}

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
