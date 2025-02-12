import coloredlogs,logging
import colorama
import os
import sys
import zmq
import toml
from fmpy import read_model_description, extract
from fmpy.fmi2 import FMU2Slave

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

    fmu = FMU2Slave(guid=model_description.guid,
                    unzipDirectory=unzipdir,
                    modelIdentifier=model_description.coSimulation.modelIdentifier,
                    instanceName='instance1')

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
    state = Fmi2EmptyReturn().SerializeToString()
    socket.send(state)

    # dispatch commands to black-box FMU with fmpy
    command = Fmi2Command()
    while True:

        msg = socket.recv()
        command.ParseFromString(msg)

        group = command.WhichOneof("command")
        data = getattr(command, command.WhichOneof("command"))

        # ================= FMI2 =================

        if group == "Fmi2Instantiate":
            fmu.instantiate()
            result = Fmi2EmptyReturn()
            #fmu.instantiate() ## Done above (zmq never sending instantiate)
        elif group == "Fmi2DoStep":
            result = Fmi2StatusReturn()
            fmu.doStep(data.current_time, data.step_size, noSetFMUStatePriorToCurrentPoint=data.no_set_fmu_state_prior_to_current_point)
            result.status = 0
        elif group == "Fmi2SetDebugLogging":
            result = Fmi2StatusReturn()
            result.status = fmu.setDebugLogging(data.logging_on, data.categories)
        elif group == "Fmi2SetupExperiment":
            result = Fmi2StatusReturn()
            result.status = fmu.setupExperiment(
                startTime=data.start_time, stopTime=data.stop_time, tolerance=data.tolerance
            )
        elif group == "Fmi2EnterInitializationMode":
            result = Fmi2StatusReturn()
            fmu.enterInitializationMode()
            result.status = 0
        elif group == "Fmi2ExitInitializationMode":
            result = Fmi2StatusReturn()
            fmu.exitInitializationMode()
            result.status = 0
        elif group == "Fmi2FreeInstance":
            result = Fmi2FreeInstanceReturn()
            fmu.freeInstance()
            logger.info(f"Fmi2FreeInstance received, shutting down")
            sys.exit(0)
        elif group == "Fmi2Terminate":
            result = Fmi2StatusReturn()
            fmu.terminate()
            result.status = 0
        elif group == "Fmi2Reset":
            result = Fmi2StatusReturn()
            result.status = fmu.reset()
        elif group == "Fmi2SerializeFmuState":
            result = Fmi2SerializeFmuStateReturn()
            result.state = fmu.serializeFMUstate(fmu.getFMUState())
            result.status = 0
        elif group == "Fmi2DeserializeFmuState":
            result = Fmi2StatusReturn()
            result.status = fmu.deserializeFMUstate(data.state)
        elif group == "Fmi2GetReal":
            result = Fmi2GetRealReturn()            
            result.values[:] = fmu.getReal(data.references)
            result.status = 0
        elif group == "Fmi2GetInteger":
            result = Fmi2GetIntegerReturn()            
            result.values[:] = fmu.getInteger(data.references)
            result.status = 0
        elif group == "Fmi2GetBoolean":
            result = Fmi2GetBooleanReturn()            
            result.values[:] = fmu.getBoolean(data.references)
            result.status = 0
        elif group == "Fmi2GetString":
            result = Fmi2GetStringReturn()            
            result.values[:] = fmu.getString(data.references)
            result.status = 0
        elif group == "Fmi2SetReal":
            result = Fmi2StatusReturn()
            fmu.setReal(data.references,data.values)
            result.status = 0            
        elif group == "Fmi2SetInteger":
            result = Fmi2StatusReturn()
            fmu.setInteger(data.references,data.values)
            result.status = 0  
        elif group == "Fmi2SetBoolean":
            result = Fmi2StatusReturn()
            fmu.setBoolean(data.references,data.values)
            result.status = 0  
        elif group == "Fmi2SetString":
            result = Fmi2StatusReturn()
            fmu.setString(data.references,data.values)
            result.status = 0  
        else:
            logger.error(f"unrecognized command '{group}' received, shutting down")
            sys.exit(-1)

        state = result.SerializeToString()
        socket.send(state)
