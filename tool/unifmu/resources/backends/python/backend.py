import logging
import os
import sys
from schemas.unifmu_fmi2_pb2 import (
    Fmi2Command,
    Fmi2Return,
    Fmi2ExtHandshakeReturn,
    Fmi2ExtSerializeSlaveReturn,
    Fmi2StatusReturn,
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

    # result = Fmi2Return()
    # result.Fmi2ExtHandshakeReturn.SetInParent()

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
            state = command.Fmi2ExtDeserializeSlave.state
            result.Fmi2StatusReturn.status = slave.fmi2ExtDeserialize(state)
        elif group == "Fmi2GetReal":
            status, values = slave.fmi2GetReal(command.Fmi2GetReal.references)
            result.Fmi2GetRealReturn.status = status
            result.Fmi2GetRealReturn.values[:] = values
        elif group == "Fmi2GetInteger":
            status, values = slave.fmi2GetInteger(command.Fmi2GetInteger.references)
            result.Fmi2GetIntegerReturn.status = status
            result.Fmi2GetIntegerReturn.values[:] = values
        elif group == "Fmi2GetBoolean":
            status, values = slave.fmi2GetBoolean(command.Fmi2GetBoolean.references)
            result.Fmi2GetBooleanReturn.status = status
            result.Fmi2GetBooleanReturn.values[:] = values
        elif group == "Fmi2GetString":
            status, values = slave.fmi2GetString(command.Fmi2GetString.references)
            result.Fmi2GetStringReturn.status = status
            result.Fmi2GetStringReturn.values[:] = values
        elif group == "Fmi2SetReal":
            status = slave.fmi2SetReal(
                command.Fmi2SetReal.references, command.Fmi2SetReal.values
            )
            result.Fmi2StatusReturn.status = status
        elif group == "Fmi2SetInteger":
            status = slave.fmi2SetInteger(
                command.Fmi2SetInteger.references, command.Fmi2SetInteger.values
            )
            result.Fmi2StatusReturn.status = status
        elif group == "Fmi2SetBoolean":
            status = slave.fmi2SetBoolean(
                command.Fmi2SetBoolean.references, command.Fmi2SetBoolean.values
            )
            result.Fmi2StatusReturn.status = status
        elif group == "Fmi2SetString":
            status = slave.fmi2SetString(
                command.Fmi2SetString.references, command.Fmi2SetString.values
            )
            result.Fmi2StatusReturn.status = status
        elif group == "Fmi2Terminate":
            result.Fmi2StatusReturn.status = slave.fmi2Terminate()
        elif group == "Fmi2Reset":
            result.Fmi2StatusReturn.status = slave.fmi2Reset()
        elif group == "Fmi2FreeInstance":
            result.Fmi2FreeInstanceReturn.SetInParent()
            pass
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)

        if group == "Fmi2FreeInstance":
            logger.info(f"Fmi2FreeInstance received, shutting down")
            sys.exit(0)
