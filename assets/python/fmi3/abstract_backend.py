import logging
import zmq
from abc import ABC, abstractmethod

from schemas.fmi3_messages_pb2 import (
    Fmi3Command,
    Fmi3Return,
    Fmi3LogReturn,
    Fmi3StatusReturn,
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
        command = Fmi3Command()
        command.ParseFromString(msg)

        return (
            command.WhichOneof("command"),
            getattr(command, command.WhichOneof("command"))
        )
    
    def status_reply(self, status):
        self.send_reply(
            Fmi3Return(
                status=Fmi3StatusReturn(
                    status=status
                )
            )            
        )

    def log_callback(self, status, category, message):
        self.send_reply(
            Fmi3Return(
                log=Fmi3LogReturn(
                    status=status,
                    category=category,
                    log_message=message
                )
            )
        )

        (command_group, _) = self.recv_command()

        match command_group:
            case "Fmi3CallbackContinue":
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