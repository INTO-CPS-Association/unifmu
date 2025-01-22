import coloredlogs,logging
import colorama
import os
import sys
import zmq
import toml

from schemas.fmi2_messages_pb2 import (
    Fmi2EmptyReturn,
    Fmi2Command,
    Fmi2StatusReturn,
    Fmi2FreeInstanceReturn,
    Fmi2SerializeFmuStateReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn,
)
from schemas.unifmu_handshake_pb2 import (
    HandshakeStatus,
    HandshakeReply,
)
from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)
coloredlogs.install(level='DEBUG')
colorama.init()
__location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))
BOLD = '\033[1m'

if __name__ == "__main__":
    input_ok = False
    if len(sys.argv) == 2:
        try:
            proxy_port = int(sys.argv[1])
            input_ok = True
        except:
            logger.error(f'Only one argument for the port in integer format is accepted.')
            sys.exit(-1)

    while not input_ok:
        port_str = input(f'{colorama.Back.GREEN}Input the port for remote proxy FMU:{colorama.Style.RESET_ALL}\n')
        try:
            proxy_port = int(port_str)
            input_ok = True
        except:
            logger.error(f'Only integers accepted.')

    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)

    with open(os.path.join(__location__,'endpoint.toml'), 'r') as f:
        endpoint_config = toml.load(f)
        proxy_ip_address = endpoint_config["ip"]
    dispatcher_endpoint =  str(proxy_ip_address) + ":" + str(proxy_port)
    logger.info(f"dispatcher endpoint received: {BOLD} {colorama.Back.GREEN} {dispatcher_endpoint} {colorama.Style.RESET_ALL}")

    socket.connect("tcp://" + dispatcher_endpoint)
    logger.info(f"Socket connected successfully.")

    # send handshake
    handshake = HandshakeReply()
    handshake.status = HandshakeStatus.OK
    socket.send(handshake.SerializeToString())

    # dispatch commands to model
    command = Fmi2Command()
    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        # ================= FMI2 =================

        if group == "Fmi2Instantiate":
            model = Model()
            result = Fmi2EmptyReturn()
        elif group == "Fmi2DoStep":
            result = Fmi2StatusReturn()
            result.status = model.fmi2DoStep(
                data.current_time, data.step_size, data.no_set_fmu_state_prior_to_current_point
            )
        elif group == "Fmi2SetDebugLogging":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetDebugLogging(data.categories, data.logging_on)
        elif group == "Fmi2SetupExperiment":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetupExperiment(
                data.start_time, data.stop_time, data.tolerance
            )
        elif group == "Fmi2EnterInitializationMode":
            result = Fmi2StatusReturn()
            result.status = model.fmi2EnterInitializationMode()
        elif group == "Fmi2ExitInitializationMode":
            result = Fmi2StatusReturn()
            result.status = model.fmi2ExitInitializationMode()
        elif group == "Fmi2FreeInstance":
            result = Fmi2FreeInstanceReturn()
            logger.info(f"Fmi2FreeInstance received, shutting down")
            sys.exit(0)
        elif group == "Fmi2Terminate":
            result = Fmi2StatusReturn()
            result.status = model.fmi2Terminate()
        elif group == "Fmi2Reset":
            result = Fmi2StatusReturn()
            result.status = model.fmi2Reset()
        elif group == "Fmi2SerializeFmuState":
            result = Fmi2SerializeFmuStateReturn()
            (result.status, result.state) = model.fmi2SerializeFmuState()
        elif group == "Fmi2DeserializeFmuState":
            result = Fmi2StatusReturn()
            result.status = model.fmi2DeserializeFmuState(data.state)
        elif group == "Fmi2GetReal":
            result = Fmi2GetRealReturn()
            result.status, result.values[:] = model.fmi2GetReal(data.references)
        elif group == "Fmi2GetInteger":
            result = Fmi2GetIntegerReturn()
            result.status, result.values[:] = model.fmi2GetInteger(data.references)
        elif group == "Fmi2GetBoolean":
            result = Fmi2GetBooleanReturn()
            result.status, result.values[:] = model.fmi2GetBoolean(data.references)
        elif group == "Fmi2GetString":
            result = Fmi2GetStringReturn()
            result.status, result.values[:] = model.fmi2GetString(data.references)
        elif group == "Fmi2SetReal":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetReal(data.references, data.values)
        elif group == "Fmi2SetInteger":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetInteger(data.references, data.values)
        elif group == "Fmi2SetBoolean":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetBoolean(data.references, data.values)
        elif group == "Fmi2SetString":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetString(data.references, data.values)
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)
