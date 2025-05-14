import logging
import os
import sys
import zmq

from schemas.fmi2_messages_pb2 import (
    Fmi2Command,
    Fmi2Return,
    Fmi2EmptyReturn,
    Fmi2StatusReturn,
    Fmi2FreeInstanceReturn,
    Fmi2SerializeFmuStateReturn,
    Fmi2GetRealReturn,
    Fmi2GetIntegerReturn,
    Fmi2GetBooleanReturn,
    Fmi2GetStringReturn,
    Fmi2LogReturn,
)
from schemas.unifmu_handshake_pb2 import (
    HandshakeStatus,
    HandshakeReply,
)
from model import Model

def unknown_command(command_group):
    logger.error(f"unrecognized command '{command_group}' received, shutting down")
    sys.exit(-1)

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

if __name__ == "__main__":
    
    # initializing message queue
    context = zmq.Context()
    socket = context.socket(zmq.REQ)

    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")

    socket.connect(dispatcher_endpoint)

    # send handshake
    handshake = HandshakeReply()
    handshake.status = HandshakeStatus.OK
    socket.send(handshake.SerializeToString())

    def recv_command():
        msg = socket.recv()
        command = Fmi2Command()
        command.ParseFromString(msg)

        return (
            command.WhichOneof("command"),
            getattr(command, command.WhichOneof("command"))
        )

    def send_reply(reply):
        socket.send(reply.SerializeToString())

    def status_reply(status):
        send_reply(
            Fmi2Return(
                Fmi2StatusReturn=Fmi2StatusReturn(
                    status=status
                )
            )            
        )

    def log_callback(status, category, message):
        send_reply(
            Fmi2Return(
                Fmi2LogReturn=Fmi2LogReturn(
                    status=status,
                    category=category,
                    log_message=message
                )
            )
        )

        (command_group, _) = recv_command()

        match command_group:
            case "Fmi2CallbackContinue":
                return
            case _:
                unknown_command(command_group)

    # dispatch commands to model
    while True:

        group, data = recv_command()

        # ================= FMI2 =================
        
        match group:
            case "Fmi2Instantiate":
                model = Model(_log_callback=log_callback)
                send_reply(
                    Fmi2Return(
                        Fmi2EmptyReturn=Fmi2EmptyReturn()
                    )
                )

            case "Fmi2DoStep":
                status_reply(
                    model.fmi2DoStep(
                        data.current_time,
                        data.step_size,
                        data.no_set_fmu_state_prior_to_current_point
                    )
                )

            case "Fmi2SetDebugLogging":
                status_reply(
                    model.fmi2SetDebugLogging(
                        data.categories, data.logging_on
                    )
                )

            case "Fmi2SetupExperiment":
                status_reply(
                    model.fmi2SetupExperiment(
                        data.start_time, data.stop_time, data.tolerance
                    )
                )

            case "Fmi2EnterInitializationMode":
                status_reply(model.fmi2EnterInitializationMode())

            case "Fmi2ExitInitializationMode":
                status_reply(model.fmi2ExitInitializationMode())

            case "Fmi2FreeInstance":
                send_reply(
                    Fmi2Return(
                        Fmi2FreeInstanceReturn=Fmi2FreeInstanceReturn()
                    )
                )
                logger.info(f"Fmi2FreeInstance received, shutting down")
                sys.exit(0)

            case "Fmi2Terminate":
                status_reply(model.fmi2Terminate())

            case "Fmi2Reset":
                status_reply(model.fmi2Reset())

            case "Fmi2SerializeFmuState":
                status, state = model.fmi2SerializeFmuState()
                send_reply(
                    Fmi2Return(
                        Fmi2SerializeFmuStateReturn=Fmi2SerializeFmuStateReturn(
                            status=status,
                            state=state
                        )
                    )            
                )

            case "Fmi2DeserializeFmuState":
                status_reply(model.fmi2DeserializeFmuState(data.state))

            case "Fmi2GetReal":
                status, values = model.fmi2GetReal(data.references)
                send_reply(
                    Fmi2Return(
                        Fmi2GetRealReturn=Fmi2GetRealReturn(
                            status=status,
                            values=values
                        )
                    )            
                )

            case "Fmi2GetInteger":
                status, values = model.fmi2GetInteger(data.references)
                send_reply(
                    Fmi2Return(
                        Fmi2GetIntegerReturn=Fmi2GetIntegerReturn(
                            status=status,
                            values=values
                        )
                    )            
                )

            case "Fmi2GetBoolean":
                status, values = model.fmi2GetBoolean(data.references)
                send_reply(
                    Fmi2Return(
                        Fmi2GetBooleanReturn=Fmi2GetBooleanReturn(
                            status=status,
                            values=values
                        )
                    )            
                )

            case "Fmi2GetString":
                status, values = model.fmi2GetString(data.references)
                send_reply(
                    Fmi2Return(
                        Fmi2GetStringReturn=Fmi2GetStringReturn(
                            status=status,
                            values=values
                        )
                    )            
                )

            case "Fmi2SetReal":
                status_reply(model.fmi2SetReal(data.references, data.values))

            case "Fmi2SetInteger":
                status_reply(model.fmi2SetInteger(data.references, data.values))

            case "Fmi2SetBoolean":
                status_reply(model.fmi2SetBoolean(data.references, data.values))

            case "Fmi2SetString":
                status_reply(model.fmi2SetString(data.references, data.values))

            case _:
                unknown_command(group)
