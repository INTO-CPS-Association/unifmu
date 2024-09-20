import coloredlogs,logging
import colorama
import os
import sys
import zmq
import toml

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
    FmiGetBinaryReturn,
    Fmi3GetClockReturn,
    Fmi3GetIntervalDecimalReturn,
    Fmi3UpdateDiscreteStatesReturn,
)
from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)
coloredlogs.install(level='DEBUG')
colorama.init()
__location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))

if __name__ == "__main__":

    input_ok = False
    if sys.argv is not None:
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
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    socket.connect("tcp://" + dispatcher_endpoint)
    logger.info(f"Socket connected successfully.")

    # send handshake
    state = Fmi3EmptyReturn().SerializeToString()
    socket.send(state)

    # dispatch commands to model
    command = Fmi3Command()
    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        logger.info(f"Command: {command}")
        #print('Command:\n' + repr(command))

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
            result = Fmi3StatusReturn
            result.status = model.fmi3DeserializeFmuState(data.state)
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
        elif group == "FmiGetBinary":
            result = FmiGetBinaryReturn()
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

        #print('Result:\n' + repr(result))
        logger.info(f"Result: {result}")
        state = result.SerializeToString()
        socket.send(state)
