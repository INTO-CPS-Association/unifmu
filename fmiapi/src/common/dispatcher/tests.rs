use super::*;
use crate::fmi2::{self, fmi2_messages as m2, fmi2_types::*};
use crate::fmi3::{self, fmi3_messages as m3, fmi3_types::*};
use std::{
    ffi::CString,
    fs,
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tempfile::TempDir;
use zeromq::{ReqSocket, Socket, SocketRecv, SocketSend};

fn fixture_command(name: &str) -> Vec<String> {
    vec![
        std::env::current_exe()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
        "--exact".into(),
        format!("{}::{name}", module_path!().split_once("::").unwrap().1),
        "--nocapture".into(),
    ]
}

fn resources(version: u8, stage: &str) -> TempDir {
    let directory = TempDir::new().unwrap();
    fs::write(
        directory.path().join("exit-scenario"),
        format!("{version}:{stage}"),
    )
    .unwrap();
    let command = fixture_command("backend_exit_fixture");
    let config = ["linux", "macos", "windows"]
        .into_iter()
        .map(|os| (os.to_owned(), command.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    fs::write(
        directory.path().join("launch.toml"),
        toml::to_string(&config).unwrap(),
    )
    .unwrap();
    directory
}

fn exit_backend() -> ! {
    eprintln!("backend missing required data file");
    std::io::stderr().flush().unwrap();
    // Deliberately skip socket destructors to emulate a crash or forced kill.
    std::process::exit(7)
}

#[test]
fn backend_exit_fixture() {
    if std::env::var("UNIFMU_DISPATCHER_ENDPOINT").is_err() {
        return;
    }
    let scenario = fs::read_to_string("exit-scenario").unwrap();
    let (version, stage) = scenario.split_once(':').unwrap();
    if stage == "startup" {
        exit_backend();
    }
    Runtime::new().unwrap().block_on(async {
        let mut socket = ReqSocket::new();
        socket
            .connect(&std::env::var("UNIFMU_DISPATCHER_ENDPOINT").unwrap())
            .await
            .unwrap();
        if stage == "handshake" {
            exit_backend();
        }
        socket
            .send(
                HandshakeReply {
                    status: HandshakeStatus::Ok as i32,
                }
                .encode_to_vec()
                .into(),
            )
            .await
            .unwrap();
        loop {
            let request = tokio::time::timeout(Duration::from_secs(10), socket.recv())
                .await
                .expect("importer stopped communicating with backend")
                .unwrap();
            let request = request.get(0).unwrap();
            if version == "2" {
                use m2::fmi2_command::Command as C;
                use m2::fmi2_return::ReturnMessage as R;
                let command = m2::Fmi2Command::decode(request.as_ref())
                    .unwrap()
                    .command
                    .unwrap();
                let reply = match command {
                    C::Fmi2Instantiate(_) if stage == "instantiate" => exit_backend(),
                    C::Fmi2Instantiate(_) => R::Empty(m2::Fmi2EmptyReturn {}),
                    C::Fmi2EnterInitializationMode(_) if stage == "initialize" => exit_backend(),
                    C::Fmi2DoStep(_) if stage == "step" => exit_backend(),
                    C::Fmi2FreeInstance(_) => return,
                    _ => R::Status(m2::Fmi2StatusReturn { status: 0 }),
                };
                socket
                    .send(
                        m2::Fmi2Return {
                            return_message: Some(reply),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .await
                    .unwrap();
            } else {
                use m3::fmi3_command::Command as C;
                use m3::fmi3_return::ReturnMessage as R;
                let command = m3::Fmi3Command::decode(request.as_ref())
                    .unwrap()
                    .command
                    .unwrap();
                let reply = match command {
                    C::Fmi3InstantiateCoSimulation(_) if stage == "instantiate" => exit_backend(),
                    C::Fmi3InstantiateCoSimulation(_) => R::Empty(m3::Fmi3EmptyReturn {}),
                    C::Fmi3EnterInitializationMode(_) if stage == "initialize" => exit_backend(),
                    C::Fmi3DoStep(_) if stage == "step" => exit_backend(),
                    C::Fmi3DoStep(_) => R::DoStep(m3::Fmi3DoStepReturn {
                        status: 0,
                        last_successful_time: 0.1,
                        ..Default::default()
                    }),
                    C::Fmi3FreeInstance(_) => return,
                    _ => R::Status(m3::Fmi3StatusReturn { status: 0 }),
                };
                socket
                    .send(
                        m3::Fmi3Return {
                            return_message: Some(reply),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .await
                    .unwrap();
            }
        }
    });
}

extern "C" {
    fn fmi2_test_logger(
        environment: *const ComponentEnvironment,
        instance_name: Fmi2String,
        status: Fmi2Status,
        category: Fmi2String,
        format: Fmi2String,
        ...
    );
}
use fmi2_logger_test as _;

unsafe extern "C" fn log3(
    _: *const Fmi3InstanceEnvironment,
    _: Fmi3Status,
    _: Fmi3String,
    _: Fmi3String,
) {
}

unsafe extern "C" fn intermediate_update(
    _: *const Fmi3InstanceEnvironment,
    _: f64,
    _: bool,
    _: bool,
    _: bool,
    _: bool,
    _: *const bool,
    _: *const f64,
) {
}

fn instantiate2(directory: &TempDir) -> Option<Box<fmi2::fmi2_slave::Fmi2Slave>> {
    let location = CString::new(
        url::Url::from_directory_path(directory.path())
            .unwrap()
            .as_str(),
    )
    .unwrap();
    let callbacks = Fmi2CallbackFunctions {
        logger: fmi2_test_logger,
        _allocate_memory: None,
        _free_memory: None,
        step_finished: None,
        component_environment: unsafe { std::mem::zeroed() },
    };
    unsafe {
        fmi2::fmi2Instantiate(
            b"test\0".as_ptr().cast(),
            Fmi2Type::Fmi2CoSimulation,
            b"guid\0".as_ptr().cast(),
            location.as_ptr(),
            &callbacks,
            0,
            0,
        )
    }
}

fn instantiate3(directory: &TempDir) -> Option<fmi3::fmi3_slave::Fmi3SlaveType> {
    let path = CString::new(directory.path().to_str().unwrap()).unwrap();
    unsafe {
        fmi3::fmi3InstantiateCoSimulation(
            b"test\0".as_ptr().cast(),
            b"token\0".as_ptr().cast(),
            path.as_ptr(),
            false,
            false,
            false,
            false,
            std::ptr::NonNull::dangling().as_ptr(),
            0,
            std::ptr::null(),
            log3,
            intermediate_update,
        )
    }
}

#[test]
fn importer_exit_fixture() {
    let Ok(scenario) = std::env::var("UNIFMU_EXIT_SCENARIO") else {
        return;
    };
    let (version, stage) = scenario.split_once(':').unwrap();
    let version = version.parse::<u8>().unwrap();
    let healthy_resources = resources(version, "healthy");
    let failing_resources = resources(version, stage);
    if version == 2 {
        let mut healthy = instantiate2(&healthy_resources).unwrap();
        let failing = instantiate2(&failing_resources);
        if ["startup", "handshake", "instantiate"].contains(&stage) {
            assert!(failing.is_none());
        } else {
            let mut failing = failing.unwrap();
            let status = if stage == "initialize" {
                fmi2::fmi2EnterInitializationMode(&mut failing)
            } else {
                assert_eq!(
                    fmi2::fmi2EnterInitializationMode(&mut failing),
                    Fmi2Status::Ok
                );
                fmi2::fmi2DoStep(&mut failing, 0.0, 0.1, 1)
            };
            assert_eq!(status, Fmi2Status::Error);
            assert_eq!(
                fmi2::fmi2DoStep(&mut failing, 0.0, 0.1, 1),
                Fmi2Status::Error
            );
            drop(failing);
        }
        assert_eq!(
            fmi2::fmi2EnterInitializationMode(&mut healthy),
            Fmi2Status::Ok
        );
        assert_eq!(fmi2::fmi2DoStep(&mut healthy, 0.0, 0.1, 1), Fmi2Status::Ok);
        drop(healthy);
    } else {
        let mut healthy = instantiate3(&healthy_resources).unwrap();
        let failing = instantiate3(&failing_resources);
        let step = |slave: &mut fmi3::fmi3_slave::Fmi3Slave| {
            let (mut event, mut terminate, mut early, mut time) = (false, false, false, 0.0);
            unsafe {
                fmi3::fmi3DoStep(
                    slave,
                    0.0,
                    0.1,
                    true,
                    &mut event,
                    &mut terminate,
                    &mut early,
                    &mut time,
                )
            }
        };
        if ["startup", "handshake", "instantiate"].contains(&stage) {
            assert!(failing.is_none());
        } else {
            let mut failing = failing.unwrap();
            let status = if stage == "initialize" {
                fmi3::fmi3EnterInitializationMode(&mut failing, false, 0.0, 0.0, false, 0.0)
            } else {
                assert_eq!(
                    fmi3::fmi3EnterInitializationMode(&mut failing, false, 0.0, 0.0, false, 0.0),
                    Fmi3Status::Fmi3OK
                );
                step(&mut failing)
            };
            assert_eq!(status, Fmi3Status::Fmi3Error);
            assert_eq!(step(&mut failing), Fmi3Status::Fmi3Error);
            drop(failing);
        }
        assert_eq!(
            fmi3::fmi3EnterInitializationMode(&mut healthy, false, 0.0, 0.0, false, 0.0),
            Fmi3Status::Fmi3OK
        );
        assert_eq!(step(&mut healthy), Fmi3Status::Fmi3OK);
        drop(healthy);
    }
}

#[test]
fn backend_exits_do_not_abort_or_hang_importer() {
    for version in [2, 3] {
        for stage in ["startup", "handshake", "instantiate", "initialize", "step"] {
            let command = fixture_command("importer_exit_fixture");
            let output = tempfile::tempfile().unwrap();
            let mut importer = Command::new(&command[0])
                .args(&command[1..])
                .env("UNIFMU_EXIT_SCENARIO", format!("{version}:{stage}"))
                .stdout(Stdio::from(output.try_clone().unwrap()))
                .stderr(Stdio::from(output.try_clone().unwrap()))
                .spawn()
                .unwrap();
            let start = Instant::now();
            let status = loop {
                if let Some(status) = importer.try_wait().unwrap() {
                    break status;
                }
                if start.elapsed() > Duration::from_secs(15) {
                    importer.kill().unwrap();
                    importer.wait().unwrap();
                    panic!("FMI{version} importer hung after backend exit during {stage}");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            if !status.success() {
                use std::io::{Read, Seek, SeekFrom};
                let mut output = output;
                output.seek(SeekFrom::Start(0)).unwrap();
                let mut message = String::new();
                output.read_to_string(&mut message).unwrap();
                panic!("FMI{version} importer aborted during {stage}: {status}\n{message}");
            }
        }
    }
}

#[test]
fn backend_exit_diagnostics_are_preserved() {
    for stage in ["startup", "instantiate"] {
        let directory = resources(2, stage);
        let mut dispatcher =
            Dispatcher::local(directory.path(), &fixture_command("backend_exit_fixture")).unwrap();
        let error = if stage == "startup" {
            dispatcher.await_handshake().unwrap_err()
        } else {
            dispatcher.await_handshake().unwrap();
            let instantiate = m2::Fmi2Command {
                command: Some(m2::fmi2_command::Command::Fmi2Instantiate(
                    Default::default(),
                )),
            };
            dispatcher
                .send_and_recv::<_, m2::Fmi2Return>(&instantiate)
                .unwrap_err()
        }
        .to_string();
        assert!(error.contains("exit status 7"), "{error}");
        assert!(
            error.contains("backend missing required data file"),
            "{error}"
        );
        assert!(matches!(
            dispatcher.await_handshake(),
            Err(DispatcherError::ConnectionFailed)
        ));
    }
}

#[test]
fn socket_disconnect_returns_error_without_process_monitor() {
    let directory = resources(2, "step");
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        let mut socket = BackendSocket::create("tcp://127.0.0.1:0").await.unwrap();
        let _backend = BackendSubprocess::create(
            socket.endpoint.to_string(),
            &fixture_command("backend_exit_fixture"),
            directory.path(),
        )
        .unwrap();
        let result = tokio::time::timeout(Duration::from_secs(10), async {
            socket.recv::<HandshakeReply>().await.unwrap();
            let instantiate = m2::Fmi2Command {
                command: Some(m2::fmi2_command::Command::Fmi2Instantiate(
                    Default::default(),
                )),
            };
            socket
                .send_and_recv::<_, m2::Fmi2Return>(&instantiate)
                .await
                .unwrap();
            let step = m2::Fmi2Command {
                command: Some(m2::fmi2_command::Command::Fmi2DoStep(Default::default())),
            };
            socket.send_and_recv::<_, m2::Fmi2Return>(&step).await
        })
        .await
        .expect("socket hung after peer disconnected");
        assert!(
            matches!(
                result,
                Err(SocketError::ZmqReceive(_) | SocketError::Disconnected)
            ),
            "{result:?}"
        );
    });
}

#[test]
fn communication_panics_are_contained_and_connection_is_not_reused() {
    let runtime = Runtime::new().unwrap();
    let mut failed = false;
    let result: DispatcherResult<()> = guarded_communication(&mut failed, || {
        runtime.block_on(async { panic!("simulated dependency panic") })
    });
    assert!(
        matches!(result, Err(DispatcherError::CommunicationPanic(message))
        if message == "simulated dependency panic")
    );
    assert!(failed);
    let result = guarded_communication(&mut failed, || -> DispatcherResult<()> {
        panic!("failed connection must not be used again")
    });
    assert!(matches!(result, Err(DispatcherError::ConnectionFailed)));
}
