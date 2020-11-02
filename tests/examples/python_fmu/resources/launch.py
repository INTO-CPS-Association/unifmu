import argparse
from argparse import ArgumentError, ArgumentParser
import json
import sys
from fmu import FMU, Fmi2Status
import zmq


import logging


if __name__ == "__main__":

    logging.basicConfig(level=logging.INFO)
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

    print(f"connecting to: {args.handshake_endpoint}")
    handshake_socket.connect(f"{args.handshake_endpoint}")

    command_port = command_socket.bind_to_random_port("tcp://127.0.0.1")

    handshake_info = {
        "serialization_format": "Pickle",
        "command_endpoint": command_socket.getsockopt(zmq.LAST_ENDPOINT).decode(),
    }

    handshake_json = json.dumps(handshake_info)
    handshake_socket.send_string(handshake_json)
    print(handshake_json)

    slave = FMU()

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
        6: slave.set_xxx,
        7: slave.get_xxx,
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
            status = command_to_slave_methods[kind](args)
            command_socket.send_pyobj(status)

        elif kind == 9:

            command_socket.send_pyobj(0)
            sys.exit(0)

