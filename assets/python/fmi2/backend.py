import logging
import sys

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
from model import Model

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

class Backend(AbstractBackend):
    def command_reply_loop(self):
        while True:

            group, data = self.recv_command()
        
            match group:
                case "Fmi2Instantiate":
                    model = Model(_log_callback=self.log_callback)
                    self.send_reply(
                        Fmi2Return(
                            empty=Fmi2EmptyReturn()
                        )
                    )

                case "Fmi2DoStep":
                    self.status_reply(
                        model.fmi2DoStep(
                            data.current_time,
                            data.step_size,
                            data.no_set_fmu_state_prior_to_current_point
                        )
                    )

                case "Fmi2SetDebugLogging":
                    self.status_reply(
                        model.fmi2SetDebugLogging(
                            data.categories, data.logging_on
                        )
                    )

                case "Fmi2SetupExperiment":
                    self.status_reply(
                        model.fmi2SetupExperiment(
                            data.start_time, data.stop_time, data.tolerance
                        )
                    )

                case "Fmi2EnterInitializationMode":
                    self.status_reply(model.fmi2EnterInitializationMode())

                case "Fmi2ExitInitializationMode":
                    self.status_reply(model.fmi2ExitInitializationMode())

                case "Fmi2FreeInstance":
                    self.send_reply(
                        Fmi2Return(
                            free_instance=Fmi2FreeInstanceReturn()
                        )
                    )
                    logger.info(f"Fmi2FreeInstance received, shutting down")
                    sys.exit(0)

                case "Fmi2Terminate":
                    self.status_reply(model.fmi2Terminate())

                case "Fmi2Reset":
                    self.status_reply(model.fmi2Reset())

                case "Fmi2SerializeFmuState":
                    status, state = model.fmi2SerializeFmuState()
                    self.send_reply(
                        Fmi2Return(
                            serialize_fmu_state=Fmi2SerializeFmuStateReturn(
                                status=status,
                                state=state
                            )
                        )            
                    )

                case "Fmi2DeserializeFmuState":
                    self.status_reply(model.fmi2DeserializeFmuState(data.state))

                case "Fmi2GetReal":
                    status, values = model.fmi2GetReal(data.references)
                    self.send_reply(
                        Fmi2Return(
                            get_real=Fmi2GetRealReturn(
                                status=status,
                                values=values
                            )
                        )            
                    )

                case "Fmi2GetInteger":
                    status, values = model.fmi2GetInteger(data.references)
                    self.send_reply(
                        Fmi2Return(
                            get_integer=Fmi2GetIntegerReturn(
                                status=status,
                                values=values
                            )
                        )            
                    )

                case "Fmi2GetBoolean":
                    status, values = model.fmi2GetBoolean(data.references)
                    self.send_reply(
                        Fmi2Return(
                            get_boolean=Fmi2GetBooleanReturn(
                                status=status,
                                values=values
                            )
                        )            
                    )

                case "Fmi2GetString":
                    status, values = model.fmi2GetString(data.references)
                    self.send_reply(
                        Fmi2Return(
                            get_string=Fmi2GetStringReturn(
                                status=status,
                                values=values
                            )
                        )            
                    )

                case "Fmi2SetReal":
                    self.status_reply(model.fmi2SetReal(data.references, data.values))

                case "Fmi2SetInteger":
                    self.status_reply(model.fmi2SetInteger(data.references, data.values))

                case "Fmi2SetBoolean":
                    self.status_reply(model.fmi2SetBoolean(data.references, data.values))

                case "Fmi2SetString":
                    self.status_reply(model.fmi2SetString(data.references, data.values))

                case _:
                    self.unknown_command(group)
    
    def unknown_command(self, command_group):
        logger.error(f"unrecognized command '{command_group}' received, shutting down")
        sys.exit(-1)
