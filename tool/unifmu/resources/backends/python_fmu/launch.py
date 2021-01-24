import json
import logging
import sys
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

import zmq

from fmi2 import Fmi2Status, Fmi2FMU
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
    slave: Fmi2FMU = get_slave_instance()

    reference_to_attr = {}
    with open(Path.cwd().parent / "modelDescription.xml") as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])] = v.attrib["name"]

    # -------- getter and setter functions ---------

    def get_xxx(references):

        try:
            attributes = [reference_to_attr[vref] for vref in references]
            values = [getattr(slave, a) for a in attributes]
            logging.debug(f"read vref: {references} with value: {values}")
            return Fmi2Status.ok, values
        except Exception:
            return Fmi2Status.error, None

    def set_xxx(references, values):
        try:
            logging.debug(f"setting {references} to {values}")
            attributes = [reference_to_attr[vref] for vref in references]
            for a, v in zip(attributes, values):
                setattr(slave, a, v)
            return Fmi2Status.ok

        except Exception:
            return Fmi2Status.error

    def free_instance():
        logger.debug("freeing instance")

    # methods bound to a slave which returns status codes
    command_to_slave_methods = {
        # common
        0: slave.set_debug_logging,
        1: slave.setup_experiment,
        3: slave.enter_initialization_mode,
        4: slave.exit_initialization_mode,
        5: slave.terminate,
        6: slave.reset,
        7: set_xxx,
        8: get_xxx,
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

    # commands that which are not bound to a
    command_to_free_function = {2: free_instance}

    assert (
        len(
            set(command_to_slave_methods.keys()).intersection(
                set(command_to_free_function.keys())
            )
        )
        == 0
    ), "command kind should be either free or bound to slave, not both"

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

            command_socket.send_pyobj(None)
            sys.exit(0)
