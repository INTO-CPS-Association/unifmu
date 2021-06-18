import json
import logging
import sys
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

try:
    import zmq
except ImportError:
    logger.fatal(
        "unable to import the python library 'zmq' required by the schemaless backend. "
        "please ensure that the library is present in the python environment launching the script. "
        "the missing dependencies can be installed using 'python -m pip install unifmu[python-backend]'"
        )
    sys.exit(-1)

from fmi2 import Fmi2Status, Fmi2FMU
from model import Model

if __name__ == "__main__":

    parser = ArgumentParser()
    parser.add_argument(
        "--handshake-endpoint",
        dest="handshake_endpoint",
        type=str,
        help="socket",
        required=True,
    )
    parser.add_argument(
        "--command-endpoint",
        dest="command_endpoint",
        type=str,
        help="if specified, use this endpoint (ip:port) for command socket instead of randomly allocated.",
        required=False,
    )
    args = parser.parse_args()

    command_endpoint = (
        f"tcp://{args.command_endpoint}"
        if args.command_endpoint
        else "tcp://127.0.0.1:0"
    )

    # initializing message queue
    context = zmq.Context()
    handshake_socket = context.socket(zmq.PUSH)
    command_socket = context.socket(zmq.REP)
    logger.info(f"hanshake endpoint received: {args.handshake_endpoint}")
    handshake_socket.connect(f"{args.handshake_endpoint}")

    command_port = command_socket.bind(command_endpoint)

    handshake_info = {
        "serialization_format": "Pickle",
        "command_endpoint": command_socket.getsockopt(zmq.LAST_ENDPOINT).decode(),
    }

    handshake_json = json.dumps(handshake_info)
    handshake_socket.send_string(handshake_json)

    # create slave object then use model description to create a mapping between fmi value references and attribute names of FMU

    reference_to_attr = {}
    with open(Path.cwd().parent / "modelDescription.xml") as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])] = v.attrib["name"]

    slave: Fmi2FMU = Model(reference_to_attr)

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
