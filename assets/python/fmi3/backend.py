import logging
import os
import sys
import time
import zmq

from schemas.fmi3_messages_pb2 import (
    Fmi3Command,
    Fmi3DoStepReturn,
    Fmi3EmptyReturn,
    Fmi3StatusReturn,
    Fmi3FreeInstanceReturn,
    Fmi3SerializeFmuStateReturn,
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
    Fmi3GetBinaryReturn,
    Fmi3GetClockReturn,
    Fmi3GetIntervalDecimalReturn,
    Fmi3UpdateDiscreteStatesReturn,
    Fmi3GetIntervalFractionReturn,
    Fmi3GetShiftDecimalReturn,
    Fmi3GetShiftFractionReturn,
)
from schemas.unifmu_handshake_pb2 import (
    HandshakeStatus,
    HandshakeReply,
)

from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)


if __name__ == "__main__":

    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)

    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    socket.connect(dispatcher_endpoint)

    handshake = HandshakeReply()
    handshake.status = HandshakeStatus.OK
    socket.send(handshake.SerializeToString())

    command = Fmi3Command()
    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        logger.info(f"Command: {command}")

        # ================= FMI3 =================
        if group == "Fmi3InstantiateModelExchange":
            result = Fmi3EmptyReturn()
        elif group == "Fmi3InstantiateCoSimulation":
            model = Model(
                data.instance_name,
                data.instantiation_token,
                data.resource_path,
                data.visible,
                data.logging_on,
                data.event_mode_used,
                data.early_return_allowed,
                data.required_intermediate_variables
            )
            result = Fmi3EmptyReturn()
        elif group == "Fmi3InstantiateScheduledExecution":
            result = Fmi3EmptyReturn()
        elif group == "Fmi3EnterStepMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3EnterStepMode()
        elif group == "Fmi3EnterEventMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3EnterEventMode()
        elif group == "Fmi3DoStep":
            result = Fmi3DoStepReturn()
            (
                result.status,
                result.event_handling_needed,
                result.terminate_simulation,
                result.early_return,
                result.last_successful_time,
            ) = model.fmi3DoStep(
                data.current_communication_point,
                data.communication_step_size,
                data.no_set_fmu_state_prior_to_current_point,
            )
        elif group == "Fmi3EnterInitializationMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3EnterInitializationMode(
                data.tolerance_defined, data.tolerance, data.start_time, data.stop_time_defined, data.stop_time
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
        elif group == "Fmi3SerializeFmuState":
            result = Fmi3SerializeFmuStateReturn()
            (result.status, result.state) = model.fmi3SerializeFmuState()
        elif group == "Fmi3DeserializeFmuState":
            result = Fmi3StatusReturn()
            result.status = model.fmi3DeserializeFmuState(data.state)
        elif group == "Fmi3EnterConfigurationMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3EnterConfigurationMode()
        elif group == "Fmi3ExitConfigurationMode":
            result = Fmi3StatusReturn()
            result.status = model.fmi3ExitConfigurationMode()
        elif group == "Fmi3GetFloat32":
            result = Fmi3GetFloat32Return()
            result.status, result.values[:] = model.fmi3GetFloat32(data.value_references)
        elif group == "Fmi3GetFloat64":
            result = Fmi3GetFloat64Return()
            result.status, result.values[:] = model.fmi3GetFloat64(data.value_references)
        elif group == "Fmi3GetInt8":
            result = Fmi3GetInt8Return()
            result.status, result.values[:] = model.fmi3GetInt8(data.value_references)
        elif group == "Fmi3GetUInt8":
            result = Fmi3GetUInt8Return()
            result.status, result.values[:] = model.fmi3GetUInt8(data.value_references)
        elif group == "Fmi3GetInt16":
            result = Fmi3GetInt16Return()
            result.status, result.values[:] = model.fmi3GetInt16(data.value_references)
        elif group == "Fmi3GetUInt16":
            result = Fmi3GetUInt16Return()
            result.status, result.values[:] = model.fmi3GetUInt16(data.value_references)
        elif group == "Fmi3GetInt32":
            result = Fmi3GetInt32Return()
            result.status, result.values[:] = model.fmi3GetInt32(data.value_references)
        elif group == "Fmi3GetUInt32":
            result = Fmi3GetUInt32Return()
            result.status, result.values[:] = model.fmi3GetUInt32(data.value_references)
        elif group == "Fmi3GetInt64":
            result = Fmi3GetInt64Return()
            result.status, result.values[:] = model.fmi3GetInt64(data.value_references)
        elif group == "Fmi3GetUInt64":
            result = Fmi3GetUInt64Return()
            result.status, result.values[:] = model.fmi3GetUInt64(data.value_references)
        elif group == "Fmi3GetBoolean":
            result = Fmi3GetBooleanReturn()
            result.status, result.values[:] = model.fmi3GetBoolean(data.value_references)
        elif group == "Fmi3GetString":
            result = Fmi3GetStringReturn()
            result.status, result.values[:] = model.fmi3GetString(data.value_references)
        elif group == "Fmi3GetBinary":
            result = Fmi3GetBinaryReturn()
            result.status, result.values[:] = model.fmi3GetBinary(data.value_references)
        elif group == "Fmi3GetClock":
            result = Fmi3GetClockReturn()
            result.status, result.values[:] = model.fmi3GetClock(data.value_references)
        elif group == "Fmi3GetIntervalDecimal":
            result = Fmi3GetIntervalDecimalReturn()
            (
                result.status,
                result.intervals[:],
                result.qualifiers[:]
            ) = model.fmi3GetIntervalDecimal(
                data.value_references
            )
        elif group == "Fmi3GetIntervalFraction":
            result = Fmi3GetIntervalFractionReturn()
            (
                result.status,
                result.counters[:],
                result.resolutions[:],
                result.qualifiers[:]
            ) = model.fmi3GetIntervalFraction(
                data.value_references
            )
        elif group == "Fmi3GetShiftDecimal":
            result = Fmi3GetShiftDecimalReturn()
            (
                result.status,
                result.shifts[:],
            ) = model.fmi3GetShiftDecimal(
                data.value_references
            )
        elif group == "Fmi3GetShiftFraction":
            result = Fmi3GetShiftFractionReturn()
            (
                result.status,
                result.counters[:],
                result.resolutions[:],
            ) = model.fmi3GetShiftFraction(
                data.value_references
            )
        elif group == "Fmi3SetFloat32":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetFloat32(data.value_references, data.values)
        elif group == "Fmi3SetFloat64":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetFloat64(data.value_references, data.values)
        elif group == "Fmi3SetInt8":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetInt8(data.value_references, data.values)
        elif group == "Fmi3SetUInt8":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetUInt8(data.value_references, data.values)
        elif group == "Fmi3SetInt16":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetInt16(data.value_references, data.values)
        elif group == "Fmi3SetUInt16":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetUInt16(data.value_references, data.values)
        elif group == "Fmi3SetInt32":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetInt32(data.value_references, data.values)
        elif group == "Fmi3SetUInt32":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetUInt32(data.value_references, data.values)
        elif group == "Fmi3SetInt64":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetInt64(data.value_references, data.values)
        elif group == "Fmi3SetUInt64":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetUInt64(data.value_references, data.values)
        elif group == "Fmi3SetBoolean":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetBoolean(data.value_references, data.values)
        elif group == "Fmi3SetString":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetString(data.value_references, data.values)
        elif group == "Fmi3SetBinary":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetBinary(data.value_references, data.values)
        elif group == "Fmi3SetClock":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetClock(data.value_references, data.values)
        elif group == "Fmi3SetIntervalDecimal":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetIntervalDecimal(data.value_references, data.intervals)
        elif group == "Fmi3SetIntervalFraction":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetIntervalFraction(data.value_references, data.counters, data.resolutions)
        elif group == "Fmi3SetShiftDecimal":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetShiftDecimal(data.value_references, data.shifts)
        elif group == "Fmi3SetShiftFraction":
            result = Fmi3StatusReturn()
            result.status = model.fmi3SetShiftFraction(data.value_references, data.counters, data.resolutions)
        elif group == "Fmi3UpdateDiscreteStates":
            result = Fmi3UpdateDiscreteStatesReturn()
            (
                result.status,
                result.discrete_states_need_update,
                result.terminate_simulation,
                result.nominals_continuous_states_changed,
                result.values_continuous_states_changed,
                result.next_event_time_defined,
                result.next_event_time,
            ) = model.fmi3UpdateDiscreteStates()
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        logger.info(f"Result: {result}")
        state = result.SerializeToString()
        socket.send(state)
