#[cfg(test)]
use std::thread;

use static_dispatch::{
    master::Fmi2Master, slave::Fmi2SlaveInstance, Fmi2Return, SerializationFormat,
};

fn setup_preconnected(
    context: &zmq::Context,
    format: SerializationFormat,
) -> (Fmi2Master<zmq::Socket>, Fmi2SlaveInstance<zmq::Socket>) {
    let master_socket = context.socket(zmq::SocketType::REP).unwrap();
    let slave_socket = context.socket(zmq::SocketType::REQ).unwrap();

    master_socket.bind("tcp://*:0").unwrap();
    let master_endpoint = master_socket.get_last_endpoint().unwrap().unwrap();
    println!("Connected binary endpoint {:?}", master_endpoint);
    slave_socket.connect(&master_endpoint).unwrap();

    // send handshake from slave to binary
    let buf = Fmi2Return::Fmi2ExtHandshake {}.serialize_as(&format);
    slave_socket.send(&buf, 0).unwrap();
    let _ = master_socket.recv_bytes(0).unwrap();

    let master_init = move || Fmi2Master::from_connected_socket(format, master_socket);

    let slave_init = move || Fmi2SlaveInstance::from_connected_socket(format, slave_socket);

    let master_handle = thread::spawn(master_init);
    let slave_handle = thread::spawn(slave_init);

    let master = master_handle.join().unwrap();
    let slave = slave_handle.join().unwrap();

    (master, slave)
}

mod tests {

    use static_dispatch::Fmi2Command::{
        Fmi2DoStep, Fmi2EnterInitializationMode, Fmi2ExitInitializationMode, Fmi2FreeInstance,
        Fmi2GetBoolean, Fmi2GetInteger, Fmi2GetReal, Fmi2GetString, Fmi2Reset, Fmi2SetBoolean,
        Fmi2SetInteger, Fmi2SetReal, Fmi2SetString, Fmi2SetupExperiment, Fmi2Terminate,
    };

    use super::*;
    #[test]

    fn request_response_no_handshake() {
        // zmq
        let context = zmq::Context::new();

        let (mut server, mut client) = setup_preconnected(&context, SerializationFormat::Protobuf);

        // serializer
        let server_thread = thread::spawn(move || {
            let ret = server.invoke_command(&Fmi2SetupExperiment {
                start_time: 0.0,
                stop_time: Some(1.0),
                tolerance: Some(0.0),
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            let mut ret = server.invoke_command(&Fmi2EnterInitializationMode);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2ExitInitializationMode);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2EnterInitializationMode {});
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2SetReal {
                references: vec![1, 2, 3],
                values: vec![1.0, 2.0, 3.0],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2SetInteger {
                references: vec![1, 2, 3],
                values: vec![1, 2, 3],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2SetBoolean {
                references: vec![1, 2, 3],
                values: vec![false, true, false],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2SetString {
                references: vec![1, 2, 3],
                values: vec![
                    String::from("foo"),
                    String::from("bar"),
                    String::from("baz"),
                ],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2DoStep {
                current_time: 0.0,
                step_size: 1.0,
                no_step_prior: true,
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2GetReal {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetRealReturn {
                    status: 0,
                    values: (vec![0.0, 0.0, 0.0])
                }
            );

            ret = server.invoke_command(&Fmi2GetInteger {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetIntegerReturn {
                    status: 0,
                    values: (vec![0, 0, 0])
                }
            );

            ret = server.invoke_command(&Fmi2GetBoolean {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetBooleanReturn {
                    status: 0,
                    values: (vec![false, true, false])
                }
            );

            ret = server.invoke_command(&Fmi2GetString {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetStringReturn {
                    status: 0,
                    values: (vec![
                        String::from("foo"),
                        String::from("bar"),
                        String::from("baz"),
                    ]),
                }
            );

            ret = server.invoke_command(&static_dispatch::Fmi2Command::Fmi2ExtSerializeSlave);
            assert!(
                ret == Fmi2Return::Fmi2ExtSerializeSlaveReturn {
                    status: 0,
                    state: vec![0, 0, 0]
                }
            );

            let state = match ret {
                Fmi2Return::Fmi2ExtSerializeSlaveReturn { status: _, state } => state,
                _ => panic!("whoops wrong return"),
            };

            ret = server
                .invoke_command(&static_dispatch::Fmi2Command::Fmi2ExtDeserializeSlave { state });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2Terminate);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2Reset);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });

            ret = server.invoke_command(&Fmi2FreeInstance);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: 0 });
        });

        let client_thread = thread::spawn(move || while client.work() {});

        server_thread.join().unwrap();
        client_thread.join().unwrap();
    }
}
