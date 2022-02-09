import logging
import os
import sys

from schemas.unifmu_fmi_pb2 import (
    UnifmuHandshakeReturn
)
from schemas.unifmu_fmi_pb2 import (
    Fmi3Command,
    Fmi3StatusReturn,
    Fmi3FreeInstanceReturn,
    UnifmuFmi3SerializeReturn,
    Fmi3GetFloat32Return,
    Fmi3GetFloat64Return,
    Fmi3GetInt8Return,
    Fmi3GetUInt8Return,
    Fmi3GetInt16Return,
    Fmi3GetUInt16Return,
    Fmi3GetInt32Return,
    Fmi3GetUInt32Return,
    Fmi3GetInt64Return,
    Fmi3GetUInt64Return,
    Fmi3GetBooleanReturn, 
    Fmi3GetStringReturn,
    FmiGetBinaryReturn
)
from schemas.unifmu_fmi_pb2 import (
    Fmi2Command,
    Fmi2StatusReturn,
    Fmi2FreeInstanceReturn,
    UnifmuFmi2SerializeReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn
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

def connectFmi3Model(socket):
    command = Fmi3Command
    while True:
        msg = socket.recv()
        command.parseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        if group == "Fmi3DoStep":
            result = Fmi3StatusReturn()
            result.status = model.fmi3DoStep(
                data.current_communication_point,
                data.communication_step_size,
                data.no_set_fmu_state_prior_to_current_point,
                data.event_handling_needed,
                data.terminate_simulation,
                data.early_return,
                data.last_successful_time
            )
        elif group == "Fmi3EnterInitializationMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3EnterInitializationMode(
                data.tolerance,
                data.start_time,
                data.stop_time
            )
        elif group == "Fmi3ExitInitializationMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3ExitInitializationMode()
        elif group == "Fmi3FreeInstance":
            result = Fmi3FreeInstanceReturn()
            logger.info(f"Fmi3FreeInstance received, shutting down")
            sys.exit(0)        
        elif group == "Fmi3Terminate":
            result = Fmi3StatusReturn()
            result.status = model.fmi3Terminate()
        elif group == "Fmi3Reset":
            result = Fmi3StatusReturn()
            result.status = model.fmi3Reset()
        elif group == "UnifmuSerialize":
            result = UnifmuFmi3SerializeReturn()
            (result.status, result.state) = model.unifmuFmi3Serialize()
        elif group == "UnifmuDeserialize":
            result = Fmi3StatusReturn
            result.status = model.unifmuFmi3Deserialize(
                data.state
            )
        # -- Getters --
        elif group == "Fmi3GetFloat32":
            result = Fmi3GetFloat32Return()
            (result.status, result.values) = model.fmi3GetFloat32(
                data.value_references
            )
        elif group == "Fmi3GetFloat64":
            result = Fmi3GetFloat64Return()
            (result.status, result.values) = model.fmi3GetFloat64(
                data.value_references
            )
        elif group == "Fmi3GetInt8":
            result = Fmi3GetInt8Return()
            (result.status, result.values) = model.fmi3GetInt8(
                data.value_references
            )
        elif group == "Fmi3GetUInt8":
            result = Fmi3GetUInt8Return()
            (result.status, result.values) = model.fmi3GetUInt8(
                data.value_references
            )
        elif group == "Fmi3GetInt16":
            result = Fmi3GetInt16Return()
            (result.status, result.values) = model.fmi3GetInt16(
                data.value_references
            )
        elif group == "Fmi3GetUInt16":
            result = Fmi3GetUInt16Return()
            (result.status, result.values) = model.fmi3GetUInt16(
                data.value_references
            )
        elif group == "Fmi3GetInt32":
            result = Fmi3GetInt32Return()
            (result.status, result.values) = model.fmi3GetInt32(
                data.value_references
            )
        elif group == "Fmi3GetUInt32":
            result = Fmi3GetUInt32Return()
            (result.status, result.values) = model.fmi3GetUInt32(
                data.value_references
            )
        elif group == "Fmi3GetInt64":
            result = Fmi3GetInt64Return()
            (result.status, result.values) = model.fmi3GetInt64(
                data.value_references
            )
        elif group == "Fmi3GetUInt64":
            result = Fmi3GetUInt64Return()
            (result.status, result.values) = model.fmi3GetUInt64(
                data.value_references
            )
        elif group == "Fmi3GetBoolean":
            result = Fmi3GetBooleanReturn()
            (result.status, result.values) = model.fmi3GetBoolean(
                data.value_references
            )
        elif group == "Fmi3GetString":
            result = Fmi3GetStringReturn()
            (result.status, result.values) = model.fmi3GetString(
                data.value_references
            )
        elif group == "FmiGetBinary":
            result = FmiGetBinaryReturn()
            (result.status, result.values) = model.fmi3GetBinary(
                data.value_references
            )
        # -- Setters --
        elif group == "Fmi3SetFloat32":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetFloat32(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetFloat64":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetFloat64(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetInt8":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetInt8(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetUInt8":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetUInt8(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetInt16":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetInt16(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetUInt16":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetUInt16(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetInt32":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetInt32(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetUInt32":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetUInt32(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetInt64":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetInt64(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetUInt64":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetUInt64(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetBoolean":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetBoolean(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetString":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetString(
                data.value_references,
                data.values
            )
        elif group == "Fmi3SetBinary":
            result = Fmi2StatusReturn()
            result.status = model.fmi3SetBinary(
                data.value_references,
                data.values
            )
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)  

def connectFmi2Model(socket):
    command = Fmi2Command()

    while True:
        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")

        data = getattr(command, command.WhichOneof("command"))
        if group == "Fmi2DoStep":
            result = Fmi2StatusReturn()
            result.status = model.fmi2DoStep(
                data.current_time, 
                data.step_size, 
                data.no_step_prior
            )
        if group == "Fmi2SetDebugLogging":
            result = Fmi2StatusReturn()
            result.status = model.fmiSetDebugLogging(
                data.categores, data.logging_on
            )
        if group == "Fmi2SetupExperiment":
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
        # -- Getters --
        elif group == "Fmi2GetReal":
            result = Fmi2GetRealReturn()
            result.status, result.values[:] = model.fmi2GetReal(
                command.Fmi2GetReal.references
            )
        elif group == "Fmi2GetInteger":
            result = Fmi2GetIntegerReturn()
            result.status, result.values[:] = model.fmi2GetInteger(
                command.Fmi2GetInteger.references
            )
        elif group == "Fmi2GetBoolean":
            result = Fmi2GetBooleanReturn()
            result.status, result.values[:] = model.fmi2GetBoolean(
                command.Fmi2GetBoolean.references
            )
        elif group == "Fmi2GetString":
            result = Fmi2GetStringReturn()
            result.status, result.values[:] = model.fmi2GetString(
                command.Fmi2GetString.references
            )
        # -- Setters --
        elif group == "Fmi2SetReal":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetReal(
                command.Fmi2SetReal.references, command.Fmi2SetReal.values
            )
        elif group == "Fmi2SetInteger":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetInteger(
                command.Fmi2SetInteger.references, command.Fmi2SetInteger.values
            )
        elif group == "Fmi2SetBoolean":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetBoolean(
                command.Fmi2SetBoolean.references, command.Fmi2SetBoolean.values
            )
        elif group == "Fmi2SetString":
            result = Fmi2StatusReturn()
            result.status = model.fmi2SetString(
                command.Fmi2SetString.references, command.Fmi2SetString.values
            )
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)

if __name__ == "__main__":

    model = Model()

    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)

    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    socket.connect(dispatcher_endpoint)

    state = UnifmuHandshakeReturn().SerializeToString()
    socket.send(state)

    connectFmi2Model(socket)
    connectFmi3Model(socket)
