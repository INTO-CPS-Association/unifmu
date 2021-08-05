use common::Fmi2Status;
use rpc::{
    socket_dispatcher::{FramedSocket, SerializationFormat},
    Fmi2Command, Fmi2Return,
};

/// Implementing the slave side of the RPC
pub struct Fmi2SlaveInstance<T: FramedSocket> {
    format: SerializationFormat,
    socket: T,
}

impl<T: FramedSocket> Fmi2SlaveInstance<T> {
    pub fn from_connected_socket(format: SerializationFormat, command_socket: T) -> Self {
        Self {
            format,
            socket: command_socket,
        }
    }

    pub fn work(&mut self) -> bool {
        let buf = self.socket.recv_bytes();
        let command = Fmi2Command::deserialize_from(&buf, &self.format);

        println!("received command: {:?}, nbytes {:?}", command, buf.len());
        let mut keep_running = true;

        let response = match command {
            Fmi2Command::Fmi2DoStep {
                current_time: _,
                step_size: _,
                no_step_prior: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2EnterInitializationMode {} => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2ExitInitializationMode {} => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2FreeInstance {} => {
                keep_running = false;
                Fmi2Return::Fmi2StatusReturn {
                    status: Fmi2Status::Fmi2OK,
                }
            }
            Fmi2Command::Fmi2SetupExperiment {
                start_time: _,
                stop_time: _,
                tolerance: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2SetReal {
                references: _,
                values: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2SetInteger {
                references: _,
                values: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2SetBoolean {
                references: _,
                values: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2SetString {
                references: _,
                values: _,
            } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2GetReal { references: _ } => Fmi2Return::Fmi2GetRealReturn {
                status: Fmi2Status::Fmi2OK,
                values: vec![0.0, 0.0, 0.0],
            },
            Fmi2Command::Fmi2GetInteger { references: _ } => Fmi2Return::Fmi2GetIntegerReturn {
                status: Fmi2Status::Fmi2OK,
                values: vec![0, 0, 0],
            },
            Fmi2Command::Fmi2GetBoolean { references: _ } => Fmi2Return::Fmi2GetBooleanReturn {
                status: Fmi2Status::Fmi2OK,
                values: vec![false, true, false],
            },
            Fmi2Command::Fmi2GetString { references: _ } => Fmi2Return::Fmi2GetStringReturn {
                status: Fmi2Status::Fmi2OK,
                values: vec![
                    String::from("foo"),
                    String::from("bar"),
                    String::from("baz"),
                ],
            },
            Fmi2Command::Fmi2Reset => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },

            Fmi2Command::Fmi2Terminate => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2CancelStep => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
            Fmi2Command::Fmi2ExtSerializeSlave => Fmi2Return::Fmi2ExtSerializeSlaveReturn {
                status: Fmi2Status::Fmi2OK,
                state: vec![0, 0, 0],
            },
            Fmi2Command::Fmi2ExtDeserializeSlave { state: _ } => Fmi2Return::Fmi2StatusReturn {
                status: Fmi2Status::Fmi2OK,
            },
        };

        let buf = response.serialize_as(&self.format);
        self.socket.send(&buf);
        keep_running
    }
}
