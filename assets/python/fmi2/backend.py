import logging
import os
import sys
import zmq

from schemas.unifmu_fmi_pb2 import (
    EmptyReturn,
    FmiCommand,
    FmiGetBinaryReturn,
    Fmi2StatusReturn,
    Fmi2FreeInstanceReturn,
    UnifmuFmi2SerializeReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn,
)
from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)


if __name__ == "__main__":

    model = Model()

    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)

    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    socket.connect(dispatcher_endpoint)

    # send handshake
    state = EmptyReturn().SerializeToString()
    socket.send(state)

    # dispatch commands to model
    command = FmiCommand()
    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        # ================= FMI2 =================

        if group == "Fmi2Instantiate":
            result = EmptyReturn()
        elif group == "Fmi2DoStep":
            result = Fmi2StatusReturn()
            result.status = model.fmi2DoStep(
                data.current_time, data.step_size, data.no_step_prior
            )
        elif group == "Fmi2SetDebugLogging":
            result = Fmi2StatusReturn()
            result.status = model.fmiSetDebugLogging(data.categores, data.logging_on)
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
        elif group == "UnifmuSerialize":
            result = UnifmuFmi2SerializeReturn()
            (result.status, result.state) = model.fmi2ExtSerialize()
        elif group == "Fmi2ExtDeserializeSlave":
            result = Fmi2StatusReturn()
            result.status = model.fmi2ExtDeserialize(data.state)
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