import coloredlogs,logging
import colorama
import os
import sys
import zmq
import toml
from fmpy import read_model_description, extract
from fmpy.fmi3 import FMU3Slave, fmi3Float64, fmi3IntervalQualifier, fmi3ValueReference
from shutil import rmtree
import ctypes

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
)
from schemas.unifmu_handshake_pb2 import (
    HandshakeStatus,
    HandshakeReply,
)

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)
coloredlogs.install(level='DEBUG')
colorama.init()
__location__ = os.path.realpath(os.path.join(os.getcwd(), os.path.dirname(__file__)))
BOLD = '\033[1m'

if __name__ == "__main__":
    fmu_filename = os.path.split(os.getcwd())[1].replace("_private","") + ".fmu"

    # read the model description
    model_description = read_model_description(fmu_filename)

    # collect the value references
    vrs = {}
    for variable in model_description.modelVariables:
        vrs[variable.name] = variable.valueReference

    # extract the FMU
    unzipdir = extract(fmu_filename)

    fmu = FMU3Slave(guid=model_description.guid,
                    unzipDirectory=unzipdir,
                    modelIdentifier=model_description.coSimulation.modelIdentifier,
                    instanceName='instance1')
    
    can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
    max_size = 1024
    ArrayType = ctypes.c_ubyte * max_size

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
            # initialize
            fmu.instantiate()
            result = Fmi3EmptyReturn()
        elif group == "Fmi3InstantiateScheduledExecution":
            result = Fmi3EmptyReturn()
        elif group == "Fmi3EnterStepMode":
            result = Fmi3StatusReturn()
            fmu.enterStepMode()
            result.status = 0
        elif group == "Fmi3EnterEventMode":
            result = Fmi3StatusReturn()
            fmu.enterEventMode()
            result.status = 0
        elif group == "Fmi3DoStep":
            result = Fmi3DoStepReturn()
            result.status = 0
            (
                result.event_handling_needed,
                result.terminate_simulation,
                result.early_return,
                result.last_successful_time,
            ) = fmu.doStep(
                data.current_communication_point,
                data.communication_step_size,
                noSetFMUStatePriorToCurrentPoint=data.no_set_fmu_state_prior_to_current_point,
            )
        elif group == "Fmi3EnterInitializationMode":
            result = Fmi3StatusReturn()
            fmu.enterInitializationMode(tolerance=data.tolerance,startTime=data.start_time,stopTime=data.stop_time)
            result.status = 0
        elif group == "Fmi3ExitInitializationMode":
            result = Fmi3StatusReturn()
            fmu.exitInitializationMode()
            result.status = 0
        elif group == "Fmi3FreeInstance":
            result = Fmi3FreeInstanceReturn()
            fmu.freeInstance()
            # Clean up unzipped temporary FMU directory
            rmtree(unzipdir, ignore_errors=True)
            logger.info(f"Fmi3FreeInstance received, shutting down")
            sys.exit(0)
        elif group == "Fmi3Terminate":
            result = Fmi3StatusReturn()
            fmu.terminate()
            result.status = 0
        elif group == "Fmi3Reset":
            result = Fmi3StatusReturn()
            fmu.reset()
            result.status = 0
        elif group == "Fmi3SerializeFmuState":
            result = Fmi3SerializeFmuStateReturn()
            if can_handle_state:
                state = fmu.getFMUState()
                data = ctypes.cast(state, ctypes.POINTER(ArrayType)).contents
                bytes_data = bytes(data)
                if isinstance(bytes_data, bytes):
                    result.status = 0
                    result.state = bytes_data
                else:
                    result.status = 3
            else:
                result.status = 3
        elif group == "Fmi3DeserializeFmuState":
            result = Fmi3StatusReturn
            if can_handle_state:
                if isinstance(data.state, bytes):
                    fmu.setFMUstate(data.state)
                    result.status = 0
                else:
                    result.status = 3
            else:
                result.status = 3 
        elif group == "Fmi3GetFloat32":
            result = Fmi3GetFloat32Return()
            result.values[:] = fmu.getFloat32(data.value_references)
            result.status = 0
        elif group == "Fmi3GetFloat64":
            result = Fmi3GetFloat64Return()
            result.values[:] = fmu.getFloat64(data.value_references)
            result.status = 0
        elif group == "Fmi3GetInt8":
            result = Fmi3GetInt8Return()
            result.values[:] = fmu.getInt8(data.value_references)
            result.status = 0
        elif group == "Fmi3GetUInt8":
            result = Fmi3GetUInt8Return()
            result.values[:] = fmu.getUInt8(data.value_references)
            result.status = 0
        elif group == "Fmi3GetInt16":
            result = Fmi3GetInt16Return()
            result.values[:] = fmu.getInt16(data.value_references)
            result.status = 0
        elif group == "Fmi3GetUInt16":
            result = Fmi3GetUInt16Return()
            result.values[:] = fmu.getUInt16(data.value_references)
            result.status = 0
        elif group == "Fmi3GetInt32":
            result = Fmi3GetInt32Return()
            result.values[:] = fmu.getInt32(data.value_references)
            result.status = 0
        elif group == "Fmi3GetUInt32":
            result = Fmi3GetUInt32Return()
            result.values[:] = fmu.getUInt32(data.value_references)
            result.status = 0
        elif group == "Fmi3GetInt64":
            result = Fmi3GetInt64Return()
            result.values[:] = fmu.getInt64(data.value_references)
            result.status = 0
        elif group == "Fmi3GetUInt64":
            result = Fmi3GetUInt64Return()
            result.values[:] = fmu.getUInt64(data.value_references)
            result.status = 0
        elif group == "Fmi3GetBoolean":
            result = Fmi3GetBooleanReturn()
            result.values[:] = fmu.getBoolean(data.value_references)
            result.status = 0
        elif group == "Fmi3GetString":
            result = Fmi3GetStringReturn()
            result.values[:] = fmu.getString(data.value_references)
            result.status = 0
        elif group == "FmiGetBinary":
            result = FmiGetBinaryReturn()
            result.values[:] = fmu.getBinary(data.value_references)
            result.status = 0
        elif group == "Fmi3GetClock":
            result = Fmi3GetClockReturn()
            result.values[:] = fmu.getClock(data.value_references)
            result.status = 0
        elif group == "Fmi3GetIntervalDecimal":
            ## To be fixed for the FMPy library
            result = Fmi3GetIntervalDecimalReturn()            
            num_vars = len(data.value_references)
            vrs = (fmi3ValueReference * num_vars)(*data.value_references)
            intervals = (fmi3Float64 * num_vars)()
            qualifiers = (fmi3IntervalQualifier * num_vars)()
            fmu.getIntervalDecimal(
                data.value_references, # if not working, use 'vrs' instead
                intervals,
                qualifiers
            )
            result.intervals[:] = intervals
            result.qualifiers[:] = qualifiers
            result.status = 0
        elif group == "Fmi3SetFloat32":
            result = Fmi3StatusReturn()
            fmu.setFloat32(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetFloat64":
            result = Fmi3StatusReturn()
            fmu.setFloat64(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetInt8":
            result = Fmi3StatusReturn()
            fmu.setInt8(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetUInt8":
            result = Fmi3StatusReturn()
            fmu.setUInt8(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetInt16":
            result = Fmi3StatusReturn()
            fmu.setInt16(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetUInt16":
            result = Fmi3StatusReturn()
            fmu.setUInt16(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetInt32":
            result = Fmi3StatusReturn()
            fmu.setInt32(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetUInt32":
            result = Fmi3StatusReturn()
            fmu.setUInt32(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetInt64":
            result = Fmi3StatusReturn()
            fmu.setInt64(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetUInt64":
            result = Fmi3StatusReturn()
            fmu.setUInt64(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetBoolean":
            result = Fmi3StatusReturn()
            fmu.setBoolean(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetString":
            result = Fmi3StatusReturn()
            fmu.setString(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetBinary":
            result = Fmi3StatusReturn()
            fmu.setBinary(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3SetClock":
            result = Fmi3StatusReturn()
            fmu.setClock(data.value_references, data.values)
            result.status = 0
        elif group == "Fmi3UpdateDiscreteStates":
            result = Fmi3UpdateDiscreteStatesReturn()
            result.status = 0
            (
                result.discrete_states_need_update,
                result.terminate_simulation,
                result.nominals_continuous_states_changed,
                result.values_continuous_states_changed,
                result.next_event_time_defined,
                result.next_event_time,
            ) = fmu.updateDiscreteStates()
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        #print('Result:\n' + repr(result))
        logger.info(f"Result: {result}")
        state = result.SerializeToString()
        socket.send(state)