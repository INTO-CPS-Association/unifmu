use crate::Fmi2Status;

#[derive(Debug)]
pub enum Fmi2CommandDispatcherError {
    DecodeError(prost::DecodeError),
    EncodeError,
    SocketError,
    Timeout,
    BackendImplementationError,
}

/// Trait implemented by objects that act like an in memory representation of an FMU.
///
/// For instance the proxy may be dispatching calls to an FMU running in a seperate process via RPC.
#[allow(non_snake_case)]
pub trait Fmi2CommandDispatcher {
    fn await_handshake(&mut self) -> Result<(), Fmi2CommandDispatcherError>;

    fn fmi2EnterInitializationMode(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2ExitInitializationMode(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2DoStep(
        &mut self,
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2ExtSerializeSlave(
        &mut self,
    ) -> Result<(Fmi2Status, Vec<u8>), Fmi2CommandDispatcherError>;

    fn fmi2ExtDeserializeSlave(
        &mut self,
        state: &[u8],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2CancelStep(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;
    fn fmi2Terminate(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;
    fn fmi2Reset(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;
    fn fmi2FreeInstance(&mut self) -> Result<(), Fmi2CommandDispatcherError>;

    fn fmi2SetReal(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2SetInteger(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2GetReal(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError>;

    fn fmi2GetInteger(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<i32>>), Fmi2CommandDispatcherError>;

    fn fmi2GetBoolean(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), Fmi2CommandDispatcherError>;

    fn fmi2GetString(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), Fmi2CommandDispatcherError>;

    fn fmi2SetDebugLogging(
        &mut self,
        categories: &[String],
        logging_on: bool,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2GetRealOutputDerivatives(
        &mut self,
        references: &[u32],
        order: &[i32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError>;

    fn fmi2SetRealInputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
        values: &[f64],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError>;

    fn fmi2GetDirectionalDerivative(
        &mut self,
        references_unknown: &[u32],
        references_known: &[u32],
        direction_known: &[f64],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError>;
}
