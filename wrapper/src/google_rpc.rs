use std::{convert::TryInto, future::Future, task::Poll};
use subprocess::{Popen, PopenConfig};
use tokio::runtime::{Builder, Runtime};

use crate::{
    fmi2_proto::{
        self, handshaker_server::Handshaker, handshaker_server::HandshakerServer,
        send_command_client::SendCommandClient, HandshakeInfo, Void,
    },
    rpc::Fmi2CommandRPC,
    Fmi2Status,
};
use mpsc::{sync_channel, SyncSender};
use std::path::PathBuf;
use std::sync::mpsc;
use tonic::{
    transport::{Channel, Server},
    Request, Response, Status,
};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GRPCConfig {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
    pub macos: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum SerializationFormat {
    Protobuf,
}

/// Implementation of grpc 'handshake' service used by slaves to
/// when they have started notify the wrapper
struct HandshakeService {
    handshake_tx: SyncSender<HandshakeInfo>,
}

#[tonic::async_trait]
impl Handshaker for HandshakeService {
    async fn perform_handshake(
        &self,
        handshake_info: Request<HandshakeInfo>,
    ) -> Result<Response<Void>, Status> {
        self.handshake_tx.send(handshake_info.into_inner()).unwrap();

        Ok(Response::new(Void {}))
    }
}

/// Perform rpc using through grpc and protobuf, a schema based serialization format.
/// https://developers.google.com/protocol-buffers
pub struct ProtobufGRPC {
    client: SendCommandClient<Channel>,
    rt: Runtime,
    process: Popen,
}

// To make a blocking grpc client, we follow: https://github.com/hyperium/tonic/blob/master/examples/src/blocking/client.rs
type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = StdError> = ::std::result::Result<T, E>;

use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

struct OnceFuture {
    finished: AtomicBool,
}

impl Future for OnceFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.finished.load(SeqCst) {
            true => Poll::Ready(()),
            false => Poll::Pending,
        }
    }
}

impl OnceFuture {
    fn finish(&self) {
        self.finished.store(true, SeqCst);
    }
}

impl ProtobufGRPC {
    // pub fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    // where
    //     D: std::convert::TryInto<tonic::transport::Endpoint>,
    //     D::Error: Into<StdError>,
    // {
    //     let rt = Builder::new_multi_thread()
    //         .worker_threads(1)
    //         .enable_all()
    //         .build()
    //         .unwrap();
    //     let client = rt.block_on(SendCommandClient::connect(dst))?;

    //     Ok(Self { rt, client })
    // }

    pub fn new(
        config: &GRPCConfig,
        resources_dir: &PathBuf,
    ) -> Result<Box<dyn Fmi2CommandRPC>, String> {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        let (tx, rx) = sync_channel::<HandshakeInfo>(1);
        let svc = HandshakerServer::new(HandshakeService { handshake_tx: tx });

        let server_task = Server::builder()
            .add_service(svc)
            .serve("127.0.0.1:50051".parse().unwrap());
        rt.spawn(server_task);

        // ################################ HANDSHAKE SLAVE TO WRAPPER #######################################
        let mut command = match std::env::consts::OS {
            "windows" => config.windows.clone(),
            "macos" => config.macos.clone(),
            _ => config.linux.clone(),
        };

        command.push("--handshake-endpoint".to_string());
        let endpoint = format!("127.0.0.1:{:?}", 50051);
        command.push(endpoint.to_string());

        let process = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect(&format!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", command));

        // ################################ CONNECT SLAVE TO WRAPPER #######################################
        let info = rx.recv().unwrap();
        let connection_str = format!("http://{}:{}", info.ip_address, info.port);
        let client = rt
            .block_on(SendCommandClient::connect(connection_str))
            .unwrap();
        let rpc = Box::new(ProtobufGRPC {
            client,
            rt,
            process,
        });

        println!("Connected to slave from wrapper");

        Ok(rpc)
    }
}

trait CoerceToEmpty {
    fn to_optional(self) -> Option<Self>
    where
        Self: Sized;
}

impl<T> CoerceToEmpty for prost::alloc::vec::Vec<T> {
    fn to_optional(self) -> Option<Vec<T>> {
        match self.is_empty() {
            true => None,
            false => Some(self),
        }
    }
}

impl Fmi2CommandRPC for ProtobufGRPC {
    fn fmi2DoStep(&mut self, current_time: f64, step_size: f64, no_step_prior: bool) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::DoStep {
            current_time: current_time,
            step_size: step_size,
            no_step_prior: no_step_prior,
        });

        match self.rt.block_on(self.client.fmi2_do_step(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2CancelStep(&mut self) -> Fmi2Status {
        todo!()
    }

    fn fmi2SetDebugLogging(&mut self, categories: &[&str], logging_on: bool) -> Fmi2Status {
        todo!()
    }

    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Fmi2Status {
        let has_stop_time: bool;
        let has_tolerance: bool;
        let sp_time: f64;
        let tol: f64;
        if stop_time != None {
            has_stop_time = true;
            sp_time = stop_time.unwrap();
        } else {
            has_stop_time = false;
            sp_time = 0.0;
        }
        if tolerance != None {
            has_tolerance = true;
            tol = tolerance.unwrap();
        } else {
            has_tolerance = false;
            tol = 0.0;
        }

        let args = tonic::Request::new(fmi2_proto::SetupExperiment {
            start_time: start_time,
            stop_time: sp_time,
            tolerance: tol,
            has_stop_time: has_stop_time,
            has_tolerance: has_tolerance,
        });

        match self.rt.block_on(self.client.fmi2_setup_experiment(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2EnterInitializationMode(&mut self) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::EnterInitializationMode {});
        match self
            .rt
            .block_on(self.client.fmi2_enter_initialization_mode(args))
        {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2ExitInitializationMode(&mut self) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::ExitInitializationMode {});
        match self
            .rt
            .block_on(self.client.fmi2_exit_initialization_mode(args))
        {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2Terminate(&mut self) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::Terminate {});
        match self.rt.block_on(self.client.fmi2_terminate(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2Reset(&mut self) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::Reset {});
        match self.rt.block_on(self.client.fmi2_reset(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2SetReal(&mut self, references: &[u32], values: &[f64]) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::SetReal {
            references: Vec::from(references),
            values: Vec::from(values),
        });

        match self.rt.block_on(self.client.fmi2_set_real(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2SetInteger(&mut self, references: &[u32], values: &[i32]) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::SetInteger {
            references: Vec::from(references),
            values: Vec::from(values),
        });

        match self.rt.block_on(self.client.fmi2_set_integer(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2SetBoolean(&mut self, references: &[u32], values: &[bool]) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::SetBoolean {
            references: Vec::from(references),
            values: Vec::from(values),
        });

        match self.rt.block_on(self.client.fmi2_set_boolean(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2SetString(&mut self, references: &[u32], values: &[&str]) -> Fmi2Status {
        let s_values = values.iter().map(|s| s.to_string()).collect();
        let args = tonic::Request::new(fmi2_proto::SetString {
            references: Vec::from(references),
            values: s_values,
        });

        match self.rt.block_on(self.client.fmi2_set_string(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2GetReal(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<f64>>) {
        let args = tonic::Request::new(fmi2_proto::GetXxx {
            references: Vec::from(references),
        });

        let res = self.rt.block_on(self.client.fmi2_get_real(args));

        match res {
            Ok(s) => {
                let a = s.into_inner();
                let status = a.status.try_into().unwrap();
                let values = a.values;
                (status, values.to_optional())
            }
            Err(_) => (Fmi2Status::Fmi2Error, None),
        }
    }

    fn fmi2GetInteger(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<i32>>) {
        let args = tonic::Request::new(fmi2_proto::GetXxx {
            references: Vec::from(references),
        });

        let res = self.rt.block_on(self.client.fmi2_get_integer(args));

        match res {
            Ok(s) => {
                let a = s.into_inner();
                let status = a.status.try_into().unwrap();
                let values = a.values;
                (status, values.to_optional())
            }
            Err(_) => (Fmi2Status::Fmi2Error, None),
        }
    }

    fn fmi2GetBoolean(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<bool>>) {
        let args = tonic::Request::new(fmi2_proto::GetXxx {
            references: Vec::from(references),
        });

        let res = self.rt.block_on(self.client.fmi2_get_boolean(args));

        match res {
            Ok(s) => {
                let a = s.into_inner();
                let status = a.status.try_into().unwrap();
                let values = a.values;
                (status, values.to_optional())
            }
            Err(_) => (Fmi2Status::Fmi2Error, None),
        }
    }

    fn fmi2GetString(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<String>>) {
        let args = tonic::Request::new(fmi2_proto::GetXxx {
            references: Vec::from(references),
        });

        let res = self.rt.block_on(self.client.fmi2_get_string(args));

        match res {
            Ok(s) => {
                let a = s.into_inner();
                let status = a.status.try_into().unwrap();
                let values = a.values;
                (status, values.to_optional())
            }
            Err(_) => (Fmi2Status::Fmi2Error, None),
        }
    }

    fn serialize(&mut self) -> (Fmi2Status, Option<Vec<u8>>) {
        let args = tonic::Request::new(fmi2_proto::SerializeMessage {});
        match self.rt.block_on(self.client.serialize(args)) {
            Ok(s) => {
                let a = s.into_inner();
                let status = a.status.try_into().unwrap();
                let state = a.state;
                (status, state.to_optional())
            }
            Err(_) => (Fmi2Status::Fmi2Error, None),
        }
    }

    fn deserialize(&mut self, bytes: &[u8]) -> Fmi2Status {
        let args = tonic::Request::new(fmi2_proto::DeserializeMessage {
            state: bytes.to_vec(),
        });
        match self.rt.block_on(self.client.deserialize(args)) {
            Ok(s) => s.into_inner().status.try_into().unwrap(),
            Err(_) => Fmi2Status::Fmi2Error,
        }
    }

    fn fmi2FreeInstance(&mut self) {
        let args = tonic::Request::new(fmi2_proto::FreeInstance {});
        let _res = self.rt.block_on(self.client.fmi2_free_instance(args));
    }
}
