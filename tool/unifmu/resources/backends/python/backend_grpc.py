import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path
import logging
from concurrent import futures
import sys

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

try:
    import grpc
except ImportError:
    logger.fatal(
        "unable to import the python library 'grpc' required by the grpc backend. "
        "please ensure that the library is present in the python environment launching the script. "
        "the missing dependencies can be installed using 'python -m pip install unifmu[python-backend]'"
        )
    sys.exit(-1)

from schemas.unifmu_fmi2_pb2_grpc import (
    SendCommandServicer,
    add_SendCommandServicer_to_server,
    HandshakerStub,
)
from schemas.unifmu_fmi2_pb2 import (
    StatusReturn,
    GetRealReturn,
    GetIntegerReturn,
    GetBooleanReturn,
    GetStringReturn,
    HandshakeInfo,
    SerializeReturn,
    FmiStatus,
)

from model import Model




class CommandServicer(SendCommandServicer):
    def __init__(self, fmu):
        super().__init__()
        logger.info(f"Created python grpc slave")
        self.fmu = fmu

    ##### REAL #####
    def Fmi2SetReal(self, request, context):
        logger.info(
            f"SetReal called on slave with references {request.references} and values {request.values}"
        )
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetReal(self, request, context):
        logger.info(f"GetReal called on slave with references {request.references}")
        status, values = self.fmu.get_xxx(request.references)
        return GetRealReturn(status=status, values=values)

    ##### INTEGER #####
    def Fmi2SetInteger(self, request, context):
        logger.info(
            f"SetInteger called on slave with references {request.references} and values {request.values}"
        )
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetInteger(self, request, context):
        logger.info(f"GetInteger called on slave with references {request.references}")
        status, values = self.fmu.get_xxx(request.references)
        return GetIntegerReturn(status=status, values=values)

    ##### BOOLEAN #####
    def Fmi2SetBoolean(self, request, context):
        logger.info(
            f"SetBoolean called on slave with references {request.references} and values {request.values}"
        )
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetBoolean(self, request, context):
        logger.info(f"GetBoolean called on slave with references {request.references}")
        status, values = self.fmu.get_xxx(request.references)
        return GetBooleanReturn(status=status, values=values)

    ##### STRING #####
    def Fmi2SetString(self, request, context):
        logger.info(
            f"SetString called on slave with references {request.references} and values {request.values}"
        )
        status = self.fmu.set_xxx(request.references, request.values)
        return StatusReturn(status=status)

    def Fmi2GetString(self, request, context):
        logger.info(f"GetString called on slave with references {request.references}")
        status, values = self.fmu.get_xxx(request.references)
        return GetStringReturn(status=status, values=values)

    #### Do step ####
    def Fmi2DoStep(self, request, context):
        logger.info(
            f"DoStep called on slave with current_time: {request.current_time}, step_size: {request.step_size}, and no_step_prior: {request.no_step_prior}"
        )
        status = self.fmu.do_step(
            request.current_time, request.step_size, request.no_step_prior
        )
        return StatusReturn(status=status)

    ##### Set Debug Logging ####
    def Fmi2SetDebugLogging(self, request, context):
        logger.info(f"SetDebugLogging called on slave")
        status = self.fmu.set_debug_logging(request.categories, request.logging_on)
        return StatusReturn(status=status)

    #### Setup Experiment ####
    def Fmi2SetupExperiment(self, request, context):
        logger.info(
            f"SetupExperiment called on slave with start_time: {request.start_time}, stop_time: {request.stop_time}, tolerance: {request.tolerance}"
        )
        stop_time = request.stop_time
        tolerance = request.tolerance
        if request.has_stop_time == False:
            stop_time = None
        if request.has_tolerance == False:
            tolerance = None
        status = self.fmu.setup_experiment(request.start_time, stop_time, tolerance)
        return StatusReturn(status=status)

    #### Enter initialization mode ####
    def Fmi2EnterInitializationMode(self, request, context):
        logger.info(f"EnterInitializationMode called on slave")
        status = self.fmu.enter_initialization_mode()
        return StatusReturn(status=status)

    #### Exit initialization mode ####
    def Fmi2ExitInitializationMode(self, request, context):
        logger.info(f"ExitInitializationMode called on slave")
        status = self.fmu.exit_initialization_mode()
        return StatusReturn(status=status)

    #### Cancel Step ####
    def Fmi2CancelStep(self, request, context):
        logger.info(f"CancelStep called on slave")
        status = self.fmu.cancel_step()
        return StatusReturn(status=status)

    #### Terminate ####
    def Fmi2Terminate(self, request, context):
        logger.info(f"Terminate called on slave")
        status = self.fmu.terminate()
        return StatusReturn(status=status)

    #### Reset ####
    def Fmi2Reset(self, request, context):
        logger.info(f"Reset called on slave")
        status = self.fmu.reset()
        return StatusReturn(status=status)

    #### Free Instance ####
    def Fmi2FreeInstance(self, request, context):
        logger.info(f"FreeInstance called on slave")
        server.stop(None)
        return StatusReturn(status=FmiStatus.Ok)

    #### Serialize ####
    def Serialize(self, request, context):
        logger.info(f"Serialize called on slave")
        status, serialized_fmu = self.fmu.serialize()
        return SerializeReturn(status=status, state=serialized_fmu)

    #### Deserialize ####
    def Deserialize(self, request, context):
        logger.info(f"Deserialize called on slave")
        status = self.fmu.deserialize(request.state)
        return StatusReturn(status=status)


if __name__ == "__main__":

    parser = ArgumentParser()
    parser.add_argument(
        "--handshake-endpoint",
        dest="handshake_endpoint",
        type=str,
        help="ip_address:port",
        required=True,
    )
    parser.add_argument(
        "--command-endpoint",
        dest="command_endpoint",
        type=str,
        help="if specified, use this endpoint (ip:port) for command socket instead of randomly allocated",
        required=False,
    )

    args = parser.parse_args()

    command_endpoint = args.command_endpoint if args.command_endpoint else "127.0.0.1:0"
    ip, port = command_endpoint.split(":")

    handshake_info = parser.parse_args().handshake_endpoint
    logger.info(f"Connecting to ip and port: {handshake_info}")
    handshaker_channel = grpc.insecure_channel(handshake_info)

    reference_to_attr = {}
    path = Path.cwd().parent / "modelDescription.xml"
    with open(path) as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])] = v.attrib["name"]

    slave = Model(reference_to_attr)

    server = grpc.server(futures.ThreadPoolExecutor())
    add_SendCommandServicer_to_server(CommandServicer(slave), server)
    port = str(server.add_insecure_port(command_endpoint))
    server.start()
    logger.info(f"Started fmu slave on port: {port}")
    logger.info("Waiting!")

    # Tell the unifmu wrapper which ip and port the fmu is connected to
    handshaker_client = HandshakerStub(handshaker_channel)
    handshake_message = HandshakeInfo(ip_address=ip, port=port)
    handshaker_client.PerformHandshake(handshake_message)
    handshaker_channel.close()

    logger.info("Sent port number to wrapper!")
    server.wait_for_termination()

