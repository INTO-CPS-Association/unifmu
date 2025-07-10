import logging
import os
import sys
from fmpy import read_model_description, extract
from fmpy.fmi3 import FMU3Slave, fmi3Float64, fmi3IntervalQualifier, fmi3ValueReference
from shutil import rmtree
import ctypes

from schemas.fmi3_messages_pb2 import (
    Fmi3Return,
    Fmi3DoStepReturn,
    Fmi3EmptyReturn,
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

        self.fmu = FMU3Slave(
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
                case "Fmi3InstantiateModelExchange":
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3InstantiateCoSimulation":
                    #TODO supply blackbox with callbacks
                    self.fmu.instantiate()
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3InstantiateScheduledExecution":
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3EnterStepMode":
                    self.fmu.enterStepMode()
                    self.status_reply(0)

                case "Fmi3EnterEventMode":
                    self.fmu.enterEventMode()
                    self.status_reply(0)

                case "Fmi3DoStep":
                    (
                        event_handling_needed,
                        terminate_simulation,
                        early_return,
                        last_successful_time
                    ) = self.fmu.doStep(
                        data.current_communication_point,
                        data.communication_step_size,
                        noSetFMUStatePriorToCurrentPoint=data.no_set_fmu_state_prior_to_current_point,
                    )
                    self.send_reply(
                        Fmi3Return(
                            do_step=Fmi3DoStepReturn(
                                status=0,
                                event_handling_needed=event_handling_needed,
                                terminate_simulation=terminate_simulation,
                                early_return=early_return,
                                last_successful_time=last_successful_time
                            )
                        )
                    )
                    
                case "Fmi3EnterInitializationMode":
                    self.fmu.enterInitializationMode(
                        tolerance=data.tolerance,
                        startTime=data.start_time,
                        stopTime=data.stop_time
                    )
                    self.status_reply(0)

                case "Fmi3ExitInitializationMode":
                    self.fmu.exitInitializationMode()
                    self.status_reply(0)

                case "Fmi3FreeInstance":
                    self.fmu.freeInstance()
                    self.send_reply(
                        Fmi3Return(
                            free_instance=Fmi3FreeInstanceReturn()
                        )
                    )
                    rmtree(self.unzipdir, ignore_errors=True)
                    logger.info(f"Fmi3FreeInstance received, shutting down")        
                    sys.exit(0)

                case "Fmi3Terminate":
                    self.fmu.terminate()
                    self.status_reply(0)

                case "Fmi3Reset":
                    self.fmu.reset()
                    self.status_reply(0)

                case "Fmi3SerializeFmuState":
                    result = Fmi3SerializeFmuStateReturn()

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
                        Fmi3Return(
                            serialize_fmu_state=result
                        )
                    )

                case "Fmi3DeserializeFmuState":
                    if self.can_handle_state and isinstance(data.state, bytes):
                        self.fmu.setFMUstate(data.state)
                        self.status_reply(0)
                    else:
                        self.status_reply(3)

                case "Fmi3GetFloat32":
                    self.send_reply(
                        Fmi3Return(
                            get_float_32=Fmi3GetFloat32Return(
                                status=0,
                                values=self.fmu.getFloat32(data.value_references)
                            )
                        )
                    )
                case "Fmi3GetFloat64":
                    self.send_reply(
                        Fmi3Return(
                            get_float_64=Fmi3GetFloat64Return(
                                status=0,
                                values=self.fmu.getFloat64(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetInt8":
                    self.send_reply(
                        Fmi3Return(
                            get_int_8=Fmi3GetInt8Return(
                                status=0,
                                values=self.fmu.getInt8(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetUInt8":
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_8=Fmi3GetUInt8Return(
                                status=0,
                                values=self.fmu.getUInt8(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetInt16":
                    self.send_reply(
                        Fmi3Return(
                            get_int_16=Fmi3GetInt16Return(
                                status=0,
                                values=self.fmu.getInt16(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetUInt16":
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_16=Fmi3GetUInt16Return(
                                status=0,
                                values=self.fmu.getUInt16(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetInt32":
                    self.send_reply(
                        Fmi3Return(
                            get_int_32=Fmi3GetInt32Return(
                                status=0,
                                values=self.fmu.getInt32(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetUInt32":
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_32=Fmi3GetUInt32Return(
                                status=0,
                                values=self.fmu.getUInt32(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetInt64":
                    self.send_reply(
                        Fmi3Return(
                            get_int_64=Fmi3GetInt64Return(
                                status=0,
                                values=self.fmu.getInt64(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetUInt64":
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_64=Fmi3GetUInt64Return(
                                status=0,
                                values=self.fmu.getUInt64(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetBoolean":
                    self.send_reply(
                        Fmi3Return(
                            get_boolean=Fmi3GetBooleanReturn(
                                status=0,
                                values=self.fmu.getBoolean(data.value_references)
                            )
                        )
                    )
                case "Fmi3GetString":
                    self.send_reply(
                        Fmi3Return(
                            get_string=Fmi3GetStringReturn(
                                status=0,
                                values=self.fmu.getString(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetBinary":
                    self.send_reply(
                        Fmi3Return(
                            get_binary=Fmi3GetBinaryReturn(
                                status=0,
                                values=self.fmu.getBinary(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetClock":
                    self.send_reply(
                        Fmi3Return(
                            get_clock=Fmi3GetClockReturn(
                                status=0,
                                values=self.fmu.getClock(data.value_references)
                            )
                        )
                    )

                case "Fmi3GetIntervalDecimal":
                    ## To be fixed for the FMPy library
                    num_vars = len(data.value_references)
                    vrs = (fmi3ValueReference * num_vars)(*data.value_references)
                    intervals = (fmi3Float64 * num_vars)()
                    qualifiers = (fmi3IntervalQualifier * num_vars)()
                    (
                        new_intervals,
                        new_qualifiers
                    ) = self.fmu.getIntervalDecimal(
                        data.value_references, # if not working, use 'vrs' instead
                        intervals,
                        qualifiers
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_interval_decimal=Fmi3GetIntervalDecimalReturn(
                                status=0,
                                intervals=new_intervals,
                                qualifiers=new_qualifiers
                            )
                        )
                    )
                    
                case "Fmi3SetFloat32":
                    self.fmu.setFloat32(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetFloat64":
                    self.fmu.setFloat64(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetInt8":
                    self.fmu.setInt8(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetUInt8":
                    self.fmu.setUInt8(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetInt16":
                    self.fmu.setInt16(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetUInt16":
                    self.fmu.setUInt16(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetInt32":
                    self.fmu.setInt32(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetUInt32":
                    self.fmu.setUInt32(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetInt64":
                    self.fmu.setInt64(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetUInt64":
                    self.fmu.setUInt64(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetBoolean":
                    self.fmu.setBoolean(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetString":
                    self.fmu.setString(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetBinary":
                    self.fmu.setBinary(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3SetClock":
                    self.fmu.setClock(data.value_references, data.values)
                    self.status_reply(0)

                case "Fmi3UpdateDiscreteStates":
                    (
                        discrete_states_need_update,
                        terminate_simulation,
                        nominals_continuous_states_changed,
                        values_continuous_states_changed,
                        next_event_time_defined,
                        next_event_time
                    ) = self.fmu.updateDiscreteStates()
                    self.send_reply(
                        Fmi3Return(
                            update_discrete_states=Fmi3UpdateDiscreteStatesReturn(
                                status=0,
                                discrete_states_need_update=discrete_states_need_update,
                                terminate_simulation=terminate_simulation,
                                nominals_continuous_states_changed=nominals_continuous_states_changed,
                                values_continuous_states_changed=values_continuous_states_changed,
                                next_event_time_defined=next_event_time_defined,
                                next_event_time=next_event_time
                            )
                        )
                    )

                case _:
                    self.unknown_command(group)
    
    def unknown_command(self, command_group):
        logger.error(f"unrecognized command '{command_group}' received, freeing blackbox and shutting down")
        self.fmu.freeInstance()
        # Clean up unzipped temporary FMU directory
        rmtree(self.unzipdir, ignore_errors=True)
        sys.exit(-1)