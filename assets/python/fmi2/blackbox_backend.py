import logging
import os
import sys
from fmpy import read_model_description, extract
from fmpy.fmi2 import FMU2Slave
from shutil import rmtree
import ctypes

from schemas.fmi2_messages_pb2 import (
    Fmi2Return,
    Fmi2EmptyReturn,
    Fmi2FreeInstanceReturn,
    Fmi2SerializeFmuStateReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn
)

from abstract_backend import AbstractBackend

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

class BlackboxBackend(AbstractBackend):
    def __init__(self):
        super().__init__()

        fmu_filename = os.path.split(os.getcwd())[1].replace("_private","") + ".fmu"
        # read the model description
        model_description = read_model_description(fmu_filename)
        
        # extract the FMU
        self.unzipdir = extract(fmu_filename)

        self.fmu = FMU2Slave(
            guid=model_description.guid,
            unzipDirectory=self.unzipdir,
            modelIdentifier=model_description.coSimulation.modelIdentifier,
            instanceName='instance1'
        )

        self.can_handle_state = model_description.coSimulation.canGetAndSetFMUstate
        max_size = 1024
        self.ArrayType = ctypes.c_ubyte * max_size
    
    def command_reply_loop(self):
        while True:

            group, data = self.recv_command()

            match group:
                case "Fmi2Instantiate":
                    #TODO supply blackbox with callbacks
                    #      - log_callback [ ] (also set loggingOn to True)
                    #      - step_finished [ ]
                    self.fmu.instantiate()
                    self.send_reply(
                        Fmi2Return(
                            empty=Fmi2EmptyReturn()
                        )
                    )

                case "Fmi2DoStep":
                    self.fmu.doStep(
                        data.current_time,
                        data.step_size,
                        noSetFMUStatePriorToCurrentPoint=data.no_set_fmu_state_prior_to_current_point
                    )
                    self.status_reply(0)

                case "Fmi2SetDebugLogging":
                    self.status_reply(
                        self.fmu.setDebugLogging(
                            data.logging_on,
                            data.categories
                        )
                    )

                case "Fmi2SetupExperiment":
                    self.status_reply(
                        self.fmu.setupExperiment(
                            startTime=data.start_time,
                            stopTime=data.stop_time,
                            tolerance=data.tolerance
                        )
                    )

                case "Fmi2EnterInitializationMode":
                    self.fmu.enterInitializationMode()
                    self.status_reply(0)

                case "Fmi2ExitInitializationMode":
                    self.fmu.exitInitializationMode()
                    self.status_reply(0)

                case "Fmi2FreeInstance":
                    logger.info(f"Fmi2FreeInstance received, freeing blackbox and shutting down")
                    
                    self.fmu.freeInstance()
                    # Clean up unzipped temporary FMU directory
                    rmtree(self.unzipdir, ignore_errors=True)

                    self.send_reply(
                        Fmi2Return(
                            free_instance=Fmi2FreeInstanceReturn()
                        )
                    )

                    sys.exit(0)

                case "Fmi2Terminate":
                    self.fmu.terminate()
                    self.status_reply(0)

                case "Fmi2Reset":
                    self.status_reply(self.fmu.reset())

                case "Fmi2SerializeFmuState":
                    result = Fmi2SerializeFmuStateReturn()

                    if self.can_handle_state:
                        state = self.fmu.getFMUState()
                        data = ctypes.cast(
                            state,
                            ctypes.POINTER(self.ArrayType)
                        ).contents
                        bytes_data = bytes(data)
                        if isinstance(bytes_data, bytes):
                            result.status = 0
                            result.state = bytes_data
                        else:
                            result.status = 3
                    else:
                        result.status = 3

                    self.send_reply(
                        Fmi2Return(
                            serialize_fmu_state=result
                        )            
                    )

                case "Fmi2DeserializeFmuState":
                    if self.can_handle_state and isinstance(data.state, bytes):
                        self.fmu.setFMUstate(data.state)
                        self.status_reply(0)
                    else:
                        self.status_reply(3)

                case "Fmi2GetReal":
                    self.send_reply(
                        Fmi2Return(
                            get_real=Fmi2GetRealReturn(
                                status=0,
                                values=self.fmu.getReal(data.references)
                            )
                        )            
                    )

                case "Fmi2GetInteger":
                    self.send_reply(
                        Fmi2Return(
                            get_integer=Fmi2GetIntegerReturn(
                                status=0,
                                values=self.fmu.getInteger(data.references)
                            )
                        )            
                    )

                case "Fmi2GetBoolean":
                    self.send_reply(
                        Fmi2Return(
                            get_boolean=Fmi2GetBooleanReturn(
                                status=0,
                                values=self.fmu.getBoolean(data.references)
                            )
                        )            
                    )

                case "Fmi2GetString":
                    self.send_reply(
                        Fmi2Return(
                            get_string=Fmi2GetStringReturn(
                                status=0,
                                values=self.fmu.getString(data.references)
                            )
                        )            
                    )

                case "Fmi2SetReal":
                    self.fmu.setReal(data.references, data.values)
                    self.status_reply(0)

                case "Fmi2SetInteger":
                    self.fmu.setInteger(data.references, data.values)
                    self.status_reply(0)

                case "Fmi2SetBoolean":
                    self.fmu.setBoolean(data.references, data.values)
                    self.status_reply(0)

                case "Fmi2SetString":
                    self.fmu.setString(data.references,data.values)
                    self.status_reply(0)

                case _:
                    self.unknown_command(group)
    
    def unknown_command(self, command_group):
        logger.error(f"unrecognized command '{command_group}' received, freeing blackbox and shutting down")
        self.fmu.freeInstance()
        # Clean up unzipped temporary FMU directory
        rmtree(self.unzipdir, ignore_errors=True)
        sys.exit(-1)