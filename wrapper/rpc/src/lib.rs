pub mod fmi2_proto;
mod protobuf_compatability;
pub mod socket_dispatcher;

use common::Fmi2Status;
use serde::{Deserialize, Serialize};

/// Trait implemented by objects that act like an in memory representation of an FMU.
///
/// For instance the proxy may be dispatching calls to an FMU running in a seperate process via RPC.
pub trait Fmi2CommandDispatcher {
    fn invoke_command(&mut self, command: &Fmi2Command) -> Fmi2Return;
    fn await_handshake(&mut self);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Fmi2Command {
    Fmi2DoStep {
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    },

    Fmi2EnterInitializationMode,
    Fmi2ExitInitializationMode,
    Fmi2FreeInstance,
    Fmi2SetupExperiment {
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    },
    Fmi2SetReal {
        references: Vec<u32>,
        values: Vec<f64>,
    },
    Fmi2SetInteger {
        references: Vec<u32>,
        values: Vec<i32>,
    },
    Fmi2SetBoolean {
        references: Vec<u32>,
        values: Vec<bool>,
    },
    Fmi2SetString {
        references: Vec<u32>,
        values: Vec<String>,
    },
    Fmi2GetReal {
        references: Vec<u32>,
    },
    Fmi2GetInteger {
        references: Vec<u32>,
    },
    Fmi2GetBoolean {
        references: Vec<u32>,
    },
    Fmi2GetString {
        references: Vec<u32>,
    },

    Fmi2Reset,
    Fmi2Terminate,
    Fmi2CancelStep,

    Fmi2ExtSerializeSlave,
    Fmi2ExtDeserializeSlave {
        #[serde(with = "serde_bytes")]
        state: Vec<u8>,
    },
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Fmi2Return {
    Fmi2StatusReturn {
        status: Fmi2Status,
    },
    Fmi2GetRealReturn {
        status: Fmi2Status,
        values: Vec<f64>,
    },
    Fmi2GetIntegerReturn {
        status: Fmi2Status,
        values: Vec<i32>,
    },
    Fmi2GetBooleanReturn {
        status: Fmi2Status,
        values: Vec<bool>,
    },
    Fmi2GetStringReturn {
        status: Fmi2Status,
        values: Vec<String>,
    },
    Fmi2ExtSerializeSlaveReturn {
        status: Fmi2Status,
        state: Vec<u8>,
    },
    Fmi2ExtHandshake,
}
