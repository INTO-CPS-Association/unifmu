use tokio::runtime::Runtime;
use zeromq::{RepSocket, Socket};

use crate::{fmi2::Fmi2Status, fmi3::Fmi3Status};

pub struct CommandDispatcher {
    socket: zeromq::RepSocket,
    rt: Runtime,
    pub endpoint: String,
}

pub enum DispatcherError {
    DecodeError(prost::DecodeError),
    EncodeError,
    SocketError,
    Timeout,
    BackendImplementationError,
}

impl CommandDispatcher {
    // ================= Common ====================

    pub fn new(endpoint: &str) -> Self {
        let mut socket = RepSocket::new();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        let endpoint = rt.block_on(socket.bind(endpoint)).unwrap();

        Self {
            socket,
            rt,
            endpoint: endpoint.to_string(),
        }
    }

    pub fn await_handshake(&mut self) -> Result<(), DispatcherError> {
        todo!()
    }

    // ================= FMI3 ======================
    pub fn Fmi3DoStep() -> Result<Fmi3Status, DispatcherError> {
        todo!()
    }

    // ================= FMI2 ======================

    pub fn fmi2EnterInitializationMode(&mut self) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }
    pub fn fmi2ExitInitializationMode(&mut self) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }
    pub fn fmi2DoStep(
        &mut self,
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2ExtSerializeSlave(&mut self) -> Result<(Fmi2Status, Vec<u8>), DispatcherError> {
        todo!()
    }

    pub fn fmi2ExtDeserializeSlave(&mut self, state: &[u8]) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2CancelStep(&mut self) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }
    pub fn fmi2Terminate(&mut self) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }
    pub fn fmi2Reset(&mut self) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }
    pub fn fmi2FreeInstance(&mut self) -> Result<(), DispatcherError> {
        todo!()
    }

    pub fn fmi2SetReal(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2SetInteger(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2GetReal(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
        todo!()
    }

    pub fn fmi2GetInteger(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<i32>>), DispatcherError> {
        todo!()
    }

    pub fn fmi2GetBoolean(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), DispatcherError> {
        todo!()
    }

    pub fn fmi2GetString(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), DispatcherError> {
        todo!()
    }

    pub fn fmi2SetDebugLogging(
        &mut self,
        categories: &[String],
        logging_on: bool,
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2GetRealOutputDerivatives(
        &mut self,
        references: &[u32],
        order: &[i32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
        todo!()
    }

    pub fn fmi2SetRealInputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
        values: &[f64],
    ) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    pub fn fmi2GetDirectionalDerivative(
        &mut self,
        references_unknown: &[u32],
        references_known: &[u32],
        direction_known: &[f64],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
        todo!()
    }
}
