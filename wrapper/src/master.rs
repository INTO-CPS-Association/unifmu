use crate::Fmi2Return;

use crate::{fmi2_proto, FramedSocket};
use crate::{Fmi2Command, SerializationFormat};

pub struct Fmi2Master<T: FramedSocket> {
    format: SerializationFormat,
    socket: T,
}

#[allow(non_snake_case)]
impl<T: FramedSocket> Fmi2Master<T> {
    /// Creates a client from a socket that has already been connected to a rpc server.
    pub fn from_connected_socket(format: SerializationFormat, command_socket: T) -> Fmi2Master<T> {
        Self {
            format: format,
            socket: command_socket,
        }
    }

    pub fn invoke_command(&mut self, command: &Fmi2Command) -> Fmi2Return {
        println!("sending command: {:?}", command);
        let buf = command.serialize_as(&self.format);
        self.socket.send(&buf);
        let res = self.socket.recv_bytes();
        Fmi2Return::deserialize_from(&res, &self.format)
    }
}

impl Into<crate::Fmi2Return> for fmi2_proto::Fmi2StatusReturn {
    fn into(self) -> crate::Fmi2Return {
        Fmi2Return::Fmi2StatusReturn {
            status: self.status,
        }
    }
}
