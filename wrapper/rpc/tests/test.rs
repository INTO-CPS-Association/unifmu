#[cfg(test)]
use std::thread;

mod slave;
use common::Fmi2Status::Fmi2OK;
use rpc::{
    socket_dispatcher::{Fmi2SocketDispatcher, SerializationFormat},
    Fmi2CommandDispatcher, Fmi2Return,
};
use slave::Fmi2SlaveInstance;

fn setup_preconnected(
    context: &zmq::Context,
    format: SerializationFormat,
) -> (
    Fmi2SocketDispatcher<zmq::Socket>,
    Fmi2SlaveInstance<zmq::Socket>,
) {
    let slave_socket = context.socket(zmq::SocketType::REQ).unwrap();

    let (endpoint, mut dispatcher) = Fmi2SocketDispatcher::<zmq::Socket>::new(format);

    println!("Connected binary endpoint {:?}", endpoint);
    slave_socket.connect(&endpoint).unwrap();

    // send handshake from slave to dispatcher
    let buf = Fmi2Return::Fmi2ExtHandshake {}.serialize_as(&format);
    slave_socket.send(&buf, 0).unwrap();

    dispatcher.await_handshake();

    let slave = Fmi2SlaveInstance::from_connected_socket(format, slave_socket);

    (dispatcher, slave)
}

mod tests {
    use rpc::{
        Fmi2Command::{
            Fmi2DoStep, Fmi2EnterInitializationMode, Fmi2ExitInitializationMode,
            Fmi2ExtDeserializeSlave, Fmi2ExtSerializeSlave, Fmi2FreeInstance, Fmi2GetBoolean,
            Fmi2GetInteger, Fmi2GetReal, Fmi2GetString, Fmi2Reset, Fmi2SetBoolean, Fmi2SetInteger,
            Fmi2SetReal, Fmi2SetString, Fmi2SetupExperiment, Fmi2Terminate,
        },
        Fmi2CommandDispatcher,
    };

    fn test_request_response(format: SerializationFormat) {
        let context = zmq::Context::new();

        let (mut dispatcher, mut worker) = setup_preconnected(&context, format);

        let server_thread = thread::spawn(move || {
            let ret = dispatcher.invoke_command(&Fmi2SetupExperiment {
                start_time: 0.0,
                stop_time: Some(1.0),
                tolerance: Some(0.0),
            });
            assert!(
                ret == Fmi2Return::Fmi2StatusReturn {
                    status: common::Fmi2Status::Fmi2OK
                }
            );

            let mut ret = dispatcher.invoke_command(&Fmi2EnterInitializationMode);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2ExitInitializationMode);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2EnterInitializationMode {});
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2SetReal {
                references: vec![1, 2, 3],
                values: vec![1.0, 2.0, 3.0],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2SetInteger {
                references: vec![1, 2, 3],
                values: vec![1, 2, 3],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2SetBoolean {
                references: vec![1, 2, 3],
                values: vec![false, true, false],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2SetString {
                references: vec![1, 2, 3],
                values: vec![
                    String::from("foo"),
                    String::from("bar"),
                    String::from("baz"),
                ],
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2DoStep {
                current_time: 0.0,
                step_size: 1.0,
                no_step_prior: true,
            });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2GetReal {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetRealReturn {
                    status: Fmi2OK,
                    values: (vec![0.0, 0.0, 0.0])
                }
            );

            ret = dispatcher.invoke_command(&Fmi2GetInteger {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetIntegerReturn {
                    status: Fmi2OK,
                    values: (vec![0, 0, 0])
                }
            );

            ret = dispatcher.invoke_command(&Fmi2GetBoolean {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetBooleanReturn {
                    status: Fmi2OK,
                    values: (vec![false, true, false])
                }
            );

            ret = dispatcher.invoke_command(&Fmi2GetString {
                references: vec![1, 2, 3],
            });
            assert!(
                ret == Fmi2Return::Fmi2GetStringReturn {
                    status: Fmi2OK,
                    values: (vec![
                        String::from("foo"),
                        String::from("bar"),
                        String::from("baz"),
                    ]),
                }
            );

            ret = dispatcher.invoke_command(&Fmi2ExtSerializeSlave);
            assert!(
                ret == Fmi2Return::Fmi2ExtSerializeSlaveReturn {
                    status: Fmi2OK,
                    state: vec![0, 0, 0]
                }
            );

            let state = match ret {
                Fmi2Return::Fmi2ExtSerializeSlaveReturn { status: _, state } => state,
                _ => panic!("whoops wrong return"),
            };

            ret = dispatcher.invoke_command(&Fmi2ExtDeserializeSlave { state });
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2Terminate);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2Reset);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });

            ret = dispatcher.invoke_command(&Fmi2FreeInstance);
            assert!(ret == Fmi2Return::Fmi2StatusReturn { status: Fmi2OK });
        });

        let client_thread = thread::spawn(move || while worker.work() {});

        server_thread.join().unwrap();
        client_thread.join().unwrap();
    }

    use super::*;
    #[test]

    fn request_response_json() {
        test_request_response(SerializationFormat::Json);
    }

    #[test]
    fn request_response_pickle() {
        test_request_response(SerializationFormat::Pickle);
    }

    #[test]
    fn request_response_protobuf() {
        test_request_response(SerializationFormat::Protobuf);
    }
}
