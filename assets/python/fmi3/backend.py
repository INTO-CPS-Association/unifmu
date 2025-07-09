import logging
import sys

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
from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

class Backend(AbstractBackend):
    def command_reply_loop(self):
        while True:

            group, data = self.recv_command()
        
            match group:
                case "Fmi3InstantiateModelExchange":
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3InstantiateCoSimulation":
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
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3InstantiateScheduledExecution":
                    self.send_reply(Fmi3Return(empty=Fmi3EmptyReturn()))

                case "Fmi3EnterStepMode":
                    self.status_reply(model.fmi3EnterStepMode())

                case "Fmi3EnterEventMode":
                    self.status_reply(model.fmi3EnterEventMode())

                case "Fmi3DoStep":
                    (
                        status,
                        event_handling_needed,
                        terminate_simulation,
                        early_return,
                        last_successful_time
                    ) = model.fmi3DoStep(
                        data.current_communication_point,
                        data.communication_step_size,
                        data.no_set_fmu_state_prior_to_current_point,
                    )
                    self.send_reply(
                        Fmi3Return(
                            do_step=Fmi3DoStepReturn(
                                status=status,
                                event_handling_needed=event_handling_needed,
                                terminate_simulation=terminate_simulation,
                                early_return=early_return,
                                last_successful_time=last_successful_time
                            )
                        )
                    )
                    
                case "Fmi3EnterInitializationMode":
                    self.status_reply(
                        model.fmi3EnterInitializationMode(
                            data.tolerance_defined,
                            data.tolerance,
                            data.start_time,
                            data.stop_time_defined,
                            data.stop_time
                        )
                    )

                case "Fmi3ExitInitializationMode":
                    self.status_reply(model.fmi3ExitInitializationMode())

                case "Fmi3FreeInstance":
                    self.send_reply(
                        Fmi3Return(
                            free_instance=Fmi3FreeInstanceReturn()
                        )
                    )
                    logger.info(f"Fmi3FreeInstance received, shutting down")        
                    sys.exit(0)

                case "Fmi3Terminate":
                    self.status_reply(model.fmi3Terminate())

                case "Fmi3Reset":
                    self.status_reply(model.fmi3Reset())

                case "Fmi3SerializeFmuState":
                    status, state = model.fmi3SerializeFmuState()
                    self.send_reply(
                        Fmi3Return(
                            serialize_fmu_state=Fmi3SerializeFmuStateReturn(
                                status=status,
                                state=state
                            )
                        )
                    )

                case "Fmi3DeserializeFmuState":
                    self.status_reply(
                        model.fmi3DeserializeFmuState(data.state)
                    )

                case "Fmi3EnterConfigurationMode":
                    self.status_reply(model.fmi3EnterConfigurationMode())

                case "Fmi3ExitConfigurationMode":
                    self.status_reply(model.fmi3ExitConfigurationMode())

                case "Fmi3GetFloat32":
                    status, values = model.fmi3GetFloat32(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_float_32=Fmi3GetFloat32Return(
                                status=status,
                                values=values
                            )
                        )
                    )
                case "Fmi3GetFloat64":
                    status, values = model.fmi3GetFloat64(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_float_64=Fmi3GetFloat64Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetInt8":
                    status, values = model.fmi3GetInt8(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_int_8=Fmi3GetInt8Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetUInt8":
                    status, values = model.fmi3GetUInt8(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_8=Fmi3GetUInt8Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetInt16":
                    status, values = model.fmi3GetInt16(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_int_16=Fmi3GetInt16Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetUInt16":
                    status, values = model.fmi3GetUInt16(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_16=Fmi3GetUInt16Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetInt32":
                    status, values = model.fmi3GetInt32(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_int_32=Fmi3GetInt32Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetUInt32":
                    status, values = model.fmi3GetUInt32(
                        data.value_reference
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_32=Fmi3GetUInt32Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetInt64":
                    status, values = model.fmi3GetInt64(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_int_64=Fmi3GetInt64Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetUInt64":
                    status, values = model.fmi3GetUInt64(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_u_int_64=Fmi3GetUInt64Return(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetBoolean":
                    status, values = model.fmi3GetBoolean(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_boolean=Fmi3GetBooleanReturn(
                                status=status,
                                values=values
                            )
                        )
                    )
                case "Fmi3GetString":
                    status, values = model.fmi3GetString(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_string=Fmi3GetStringReturn(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetBinary":
                    status, values = model.fmi3GetBinary(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_binary=Fmi3GetBinaryReturn(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetClock":
                    status, values = model.fmi3GetClock(data.value_references)
                    self.send_reply(
                        Fmi3Return(
                            get_clock=Fmi3GetClockReturn(
                                status=status,
                                values=values
                            )
                        )
                    )

                case "Fmi3GetIntervalDecimal":
                    (
                        status,
                        intervals,
                        qualifiers
                    ) = model.fmi3GetIntervalDecimal(data.value_references)
                    self.send_reply(
                        Fmi3Return(
                            get_interval_decimal=Fmi3GetIntervalDecimalReturn(
                                status=status,
                                intervals=intervals,
                                qualifiers=qualifiers
                            )
                        )
                    )

                case "Fmi3GetIntervalFraction":
                    (
                        status,
                        counters,
                        resolutions,
                        qualifiers
                    ) = model.fmi3GetIntervalFraction(data.value_references)
                    self.send_reply(
                        Fmi3Return(
                            get_interval_fraction=Fmi3GetIntervalFractionReturn(
                                status=status,
                                counters=counters,
                                resolutions=resolutions,
                                qualifiers=qualifiers
                            )
                        )
                    )

                case "Fmi3GetShiftDecimal":
                    status, shifts = model.fmi3GetShiftDecimal(
                        data.value_references
                    )
                    self.send_reply(
                        Fmi3Return(
                            get_shift_decimal=Fmi3GetShiftDecimalReturn(
                                status=status,
                                shifts=shifts
                            )
                        )
                    )
                    
                case "Fmi3GetShiftFraction":
                    (
                        status,
                        counters,
                        resolutions
                    ) = model.fmi3GetShiftFraction(data.value_references)
                    self.send_reply(
                        Fmi3Return(
                            get_shift_fraction=Fmi3GetShiftFractionReturn(
                                status=status,
                                counters=counters,
                                resolutions=resolutions
                            )
                        )
                    )
                    
                case "Fmi3SetFloat32":
                    self.status_reply(
                        model.fmi3SetFloat32(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetFloat64":
                    self.status_reply(
                        model.fmi3SetFloat64(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetInt8":
                    self.status_reply(
                        model.fmi3SetInt8(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetUInt8":
                    self.status_reply(
                        model.fmi3SetUInt8(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetInt16":
                    self.status_reply(
                        model.fmi3SetInt16(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetUInt16":
                    self.status_reply(
                        model.fmi3SetUInt16(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetInt32":
                    self.status_reply(
                        model.fmi3SetInt32(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetUInt32":
                    self.status_reply(
                        model.fmi3SetUInt32(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetInt64":
                    self.status_reply(
                        model.fmi3SetInt64(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetUInt64":
                    self.status_reply(
                        model.fmi3SetUInt64(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetBoolean":
                    self.status_reply(
                        model.fmi3SetBoolean(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetString":
                    self.status_reply(
                        model.fmi3SetString(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetBinary":
                    self.status_reply(
                        model.fmi3SetBinary(
                            data.value_references,
                            data.value_sizes, data.values
                        )
                    )

                case "Fmi3SetClock":
                    self.status_reply(
                        model.fmi3SetClock(
                            data.value_references,
                            data.values
                        )
                    )

                case "Fmi3SetIntervalDecimal":
                    self.status_reply(
                        model.fmi3SetIntervalDecimal(
                            data.value_references,
                            data.intervals
                        )
                    )

                case "Fmi3SetIntervalFraction":
                    self.status_reply(
                        model.fmi3SetIntervalFraction(
                            data.value_references,
                            data.counters,
                            data.resolutions
                        )
                    )

                case "Fmi3SetShiftDecimal":
                    self.status_reply(
                        model.fmi3SetShiftDecimal(
                            data.value_references,
                            data.shifts
                        )
                    )

                case "Fmi3SetShiftFraction":
                    self.status_reply(
                        model.fmi3SetShiftFraction(
                            data.value_references,
                            data.counters,
                            data.resolutions
                        )
                    )

                case "Fmi3UpdateDiscreteStates":
                    (
                        status,
                        discrete_states_need_update,
                        terminate_simulation,
                        nominals_continuous_states_changed,
                        values_continuous_states_changed,
                        next_event_time_defined,
                        next_event_time
                    ) = model.fmi3UpdateDiscreteStates()
                    self.send_reply(
                        Fmi3Return(
                            update_discrete_states=Fmi3UpdateDiscreteStatesReturn(
                                status=status,
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
        logger.error(f"unrecognized command '{command_group}' received, shutting down")
        sys.exit(-1)
