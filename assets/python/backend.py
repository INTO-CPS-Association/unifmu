import logging
import os
import sys
from schemas.unifmu_fmi2_pb2 import (
    Fmi2Command,
    Fmi2ExtHandshakeReturn,
    Fmi2ExtSerializeSlaveReturn,
    Fmi2StatusReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn,
    Fmi2FreeInstanceReturn,
)

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

from model import Model

if __name__ == "__main__":

    slave = Model()

    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")
    socket.connect(dispatcher_endpoint)

    state = Fmi2ExtHandshakeReturn().SerializeToString()
    socket.send(state)

    command = Fmi2Command()

    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")

        data = getattr(command, command.WhichOneof("command"))
        if group == "Fmi2SetupExperiment":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2SetupExperiment(
                data.start_time, data.stop_time, data.tolerance
            )

        elif group == "Fmi2SetDebugLogging":
            result = Fmi2StatusReturn()
            result.status = slave.fmiSetDebugLogging(
                data.categores, data.logging_on
            )

        elif group == "Fmi2DoStep":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2DoStep(
                data.current_time, data.step_size, data.no_step_prior
            )
        elif group == "Fmi2EnterInitializationMode":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2EnterInitializationMode()
        elif group == "Fmi2ExitInitializationMode":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2ExitInitializationMode()
        elif group == "Fmi2ExtSerializeSlave":
            result = Fmi2ExtSerializeSlaveReturn()
            (result.status, result.state) = slave.fmi2ExtSerialize()
        elif group == "Fmi2ExtDeserializeSlave":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2ExtDeserialize(data.state)
        elif group == "Fmi2GetReal":
            result = Fmi2GetRealReturn()
            result.status, result.values[:] = slave.fmi2GetReal(
                command.Fmi2GetReal.references
            )
        elif group == "Fmi2GetInteger":
            result = Fmi2GetIntegerReturn()
            result.status, result.values[:] = slave.fmi2GetInteger(
                command.Fmi2GetInteger.references
            )
        elif group == "Fmi2GetBoolean":
            result = Fmi2GetBooleanReturn()
            result.status, result.values[:] = slave.fmi2GetBoolean(
                command.Fmi2GetBoolean.references
            )
        elif group == "Fmi2GetString":
            result = Fmi2GetStringReturn()
            result.status, result.values[:] = slave.fmi2GetString(
                command.Fmi2GetString.references
            )
        elif group == "Fmi2SetReal":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2SetReal(
                command.Fmi2SetReal.references, command.Fmi2SetReal.values
            )

        elif group == "Fmi2SetInteger":
            result = Fmi2StatusReturn()

            result.status = slave.fmi2SetInteger(
                command.Fmi2SetInteger.references, command.Fmi2SetInteger.values
            )

        elif group == "Fmi2SetBoolean":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2SetBoolean(
                command.Fmi2SetBoolean.references, command.Fmi2SetBoolean.values
            )
        elif group == "Fmi2SetString":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2SetString(
                command.Fmi2SetString.references, command.Fmi2SetString.values
            )
        elif group == "Fmi2Terminate":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2Terminate()
        elif group == "Fmi2Reset":
            result = Fmi2StatusReturn()
            result.status = slave.fmi2Reset()
        elif group == "Fmi2FreeInstance":
            result = Fmi2FreeInstanceReturn()
            logger.info(f"Fmi2FreeInstance received, shutting down")
            sys.exit(0)
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)
