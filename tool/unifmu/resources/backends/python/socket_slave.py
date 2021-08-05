import json
import logging
import sys
import xml.etree.ElementTree as ET
from argparse import ArgumentParser
from pathlib import Path

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

from fmi2 import  Fmi2FMU
from model import Model



if __name__ == "__main__":

    parser = ArgumentParser()
    parser.add_argument(
        "--dispatcher-endpoint",
        dest="dispatcher_endpoint",
        type=str,
        help="socket",
        required=True,
    )
   
    args = parser.parse_args()


    # initializing message queue
    context = zmq.Context()
    
    socket = context.socket(zmq.REQ)

    def send_serialized(obj):
        socket.send_json(obj)

    def recv_serialized():
        return socket.recv_json()

    logger.info(f"hanshake endpoint received: {args.dispatcher_endpoint}")

    socket.connect(args.dispatcher_endpoint)
    send_serialized({"Fmi2ExtHandshake":None})

    # create slave object then use model description to create a mapping between fmi value references and attribute names of FMU

    reference_to_attr = {}
    with open(Path.cwd().parent / "modelDescription.xml") as f:
        for v in ET.parse(f).find("ModelVariables"):
            reference_to_attr[int(v.attrib["valueReference"])] = v.attrib["name"]

    slave: Fmi2FMU = Model(reference_to_attr)

    # methods bound to a slave which returns status codes
    command_to_slave_methods = {
        # common
        "Fmi2SetupDebugLogging": slave.set_debug_logging,
        "Fmi2SetupExperiment": slave.setup_experiment,
        "Fmi2EnterInitializationMode": slave.enter_initialization_mode,
        "Fmi2ExitInitializationMode": slave.exit_initialization_mode,
        "Fmi2Terminate": slave.terminate,
        "Fmi2Reset": slave.reset,
        "Fmi2SetReal": slave.set_xxx,
        "Fmi2SetInteger": slave.set_xxx,
        "Fmi2SetBoolean": slave.set_xxx,
        "Fmi2SetString": slave.set_xxx,
        "Fmi2GetReal": slave.get_xxx,
        "Fmi2GetInteger": slave.get_xxx,
        "Fmi2GetBoolean": slave.get_xxx,
        "Fmi2GetString": slave.get_xxx,
        "Fmi2ExtSerializeSlaveState": slave.serialize,
        "Fmi2ExtDeserializeSlaveState": slave.deserialize,
        "Fmi2GetDirectionalDerivative": slave.get_directional_derivative,
        # model exchange
        # cosim
        "Fmi2SetInputDerivative": slave.set_input_derivatives,
        "Fmi2GetOutputDerivative": slave.get_output_derivatives,
        "Fmi2DoStep": slave.do_step,
        "Fmi2CancelStep": slave.cancel_step,
        "Fmi2GetStatus": slave.get_xxx_status,
    }

    # event loop
    while True:

        logger.info(f"slave waiting for command")

        
        obj = recv_serialized()
        logger.info(f"received command {obj} of type {type(obj)}")

        # object has format:
        # with-args: {"command_type" : {arg1: value1, arg2: value2}} 
        # no-args : "command_type"
        if type(obj) == str:
            kind = obj
            args = {}
        else:
            kind = next(iter(obj.keys()))
            args = next(iter(obj.values()))


        if kind not in command_to_slave_methods:
            logger.error(f"unrecognized message type: '{kind}', terminating")
            sys.exit(-1)
        elif kind == "Fmi2FreeInstance":
            logger.debug("freeing instance")
            sys.exit(0)
        else:
            status = command_to_slave_methods[kind](**args)
            result = {'Fmi2StatusReturn': {'status': status}}
            logger.info(f"returning value: {result}")
            send_serialized(result)

        # if kind == "Fmi2SetupExperiment":
        #     status = slave.setup_experiment(args["start_time"], args["stop_time"], args["tolerance"])
        #     send_serialized({'Fmi2StatusReturn': {'status': status}})

            
        # elif kind == "Fmi2EnterInitializationMode":
        #     status = slave.enter_initialization_mode()
        #     send_serialized({"Fmi2StatusReturn": {"status": status}})

        # elif kind == "Fmi2ExitInitializationMode":
        #     status = slave.exit_initialization_mode()
        #     send_serialized({"Fmi2StatusReturn": {"status": status}})
        

        # else:
        #     logger.error(f"unrecognized message type: '{kind}', terminating")
        #     sys.exit(-1)

        # if kind in command_to_slave_methods:
        #     result = command_to_slave_methods[kind](*args)
        #     logger.info(f"returning value: {result}")
        #     socket.send_pyobj(result)

        # elif kind == 2:
        #     logger.debug("freeing instance")
        #     socket.send_pyobj(None)
        #     sys.exit(0)
