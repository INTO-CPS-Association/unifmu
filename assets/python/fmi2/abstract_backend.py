import logging
import zmq
from abc import ABC, abstractmethod

from schemas.fmi2_messages_pb2 import (
    Fmi2Command,
    Fmi2Return,
    Fmi2StatusReturn,
    Fmi2LogReturn,
)
from schemas.unifmu_handshake_pb2 import (
    HandshakeStatus,
    HandshakeReply,
)

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)

class AbstractBackend(ABC):
    def __init__(self):
        self.context = zmq.Context()
        self.socket = self.context.socket(zmq.REQ)
    
    def connect_to_endpoint(self, endpoint):
        self.socket.connect(endpoint)

    def send_reply(self, reply):
        self.socket.send(reply.SerializeToString())
    
    def recv_command(self):
        msg = self.socket.recv()
        command = Fmi2Command()
        command.ParseFromString(msg)

        return (
            command.WhichOneof("command"),
            getattr(command, command.WhichOneof("command"))
        )
    
    def status_reply(self, status):
        self.send_reply(
            Fmi2Return(
                status=Fmi2StatusReturn(
                    status=status
                )
            )            
        )

    def log_callback(self, status, category, message):
        self.send_reply(
            Fmi2Return(
                log=Fmi2LogReturn(
                    status=status,
                    category=category,
                    log_message=message
                )
            )
        )

        (command_group, _) = self.recv_command()

        match command_group:
            case "Fmi2CallbackContinue":
                return
            case _:
                self.unknown_command(command_group)

    def handshake(self):
        self.send_reply(HandshakeReply(status=HandshakeStatus.OK))

    @abstractmethod
    def command_reply_loop(self):
        pass
    
    @abstractmethod
    def unknown_command(self, command_group):
        pass