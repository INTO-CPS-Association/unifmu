use crate::{fmi2_proto, Fmi2Command, Fmi2Return};

/// Defines trait facilitating the conversion between generated 'protobuf' code and handwritten rust code.

// return: rust -> pb-inner
impl From<Fmi2Return> for fmi2_proto::fmi2_return::Result {
    fn from(val: Fmi2Return) -> Self {
        match val {
            Fmi2Return::Fmi2StatusReturn { status } => {
                fmi2_proto::fmi2_return::Result::Fmi2StatusReturn(fmi2_proto::Fmi2StatusReturn {
                    status,
                })
            }
            Fmi2Return::Fmi2GetRealReturn { status, values } => {
                fmi2_proto::fmi2_return::Result::Fmi2GetRealReturn(fmi2_proto::Fmi2GetRealReturn {
                    status,
                    values,
                })
            }
            Fmi2Return::Fmi2GetIntegerReturn { status, values } => {
                fmi2_proto::fmi2_return::Result::Fmi2GetIntegerReturn(
                    fmi2_proto::Fmi2GetIntegerReturn { status, values },
                )
            }
            Fmi2Return::Fmi2GetBooleanReturn { status, values } => {
                fmi2_proto::fmi2_return::Result::Fmi2GetBooleanReturn(
                    fmi2_proto::Fmi2GetBooleanReturn { status, values },
                )
            }
            Fmi2Return::Fmi2GetStringReturn { status, values } => {
                fmi2_proto::fmi2_return::Result::Fmi2GetStringReturn(
                    fmi2_proto::Fmi2GetStringReturn { status, values },
                )
            }
            Fmi2Return::Fmi2ExtHandshake => {
                fmi2_proto::fmi2_return::Result::Fmi2ExtHandshake(fmi2_proto::Fmi2ExtHandshake {})
            }
            Fmi2Return::Fmi2ExtSerializeSlaveReturn { status, state } => {
                fmi2_proto::fmi2_return::Result::Fmi2ExtSerializeSlaveReturn(
                    fmi2_proto::Fmi2ExtSerializeSlaveReturn { status, state },
                )
            }
        }
    }
}

// return: pb-outer -> rust
impl From<fmi2_proto::Fmi2Return> for Fmi2Return {
    fn from(val: fmi2_proto::Fmi2Return) -> Self {
        match val.result.unwrap() {
            fmi2_proto::fmi2_return::Result::Fmi2StatusReturn(res) => {
                Fmi2Return::Fmi2StatusReturn { status: res.status }
            }
            fmi2_proto::fmi2_return::Result::Fmi2GetRealReturn(res) => {
                Fmi2Return::Fmi2GetRealReturn {
                    status: res.status,
                    values: res.values,
                }
            }
            fmi2_proto::fmi2_return::Result::Fmi2GetIntegerReturn(res) => {
                Fmi2Return::Fmi2GetIntegerReturn {
                    status: res.status,
                    values: res.values,
                }
            }
            fmi2_proto::fmi2_return::Result::Fmi2GetBooleanReturn(res) => {
                Fmi2Return::Fmi2GetBooleanReturn {
                    status: res.status,
                    values: res.values,
                }
            }
            fmi2_proto::fmi2_return::Result::Fmi2GetStringReturn(res) => {
                Fmi2Return::Fmi2GetStringReturn {
                    status: res.status,
                    values: res.values,
                }
            }
            fmi2_proto::fmi2_return::Result::Fmi2ExtHandshake(_) => Fmi2Return::Fmi2ExtHandshake {},
            fmi2_proto::fmi2_return::Result::Fmi2ExtSerializeSlaveReturn(res) => {
                Fmi2Return::Fmi2ExtSerializeSlaveReturn {
                    status: res.status,
                    state: res.state,
                }
            }
        }
    }
}

/// return: rust -> pb-outer
impl From<Fmi2Return> for fmi2_proto::Fmi2Return {
    fn from(val: Fmi2Return) -> Self {
        let result = Some(fmi2_proto::fmi2_return::Result::from(val));
        Self { result }
    }
}

// command: pb-outer -> rust
impl From<fmi2_proto::Fmi2Command> for Fmi2Command {
    fn from(val: fmi2_proto::Fmi2Command) -> Self {
        match val.command.unwrap() {
            fmi2_proto::fmi2_command::Command::Fmi2DoStep(cmd) => Fmi2Command::Fmi2DoStep {
                current_time: cmd.current_time,
                step_size: cmd.step_size,
                no_step_prior: cmd.no_step_prior,
            },
            fmi2_proto::fmi2_command::Command::Fmi2SetReal(cmd) => Fmi2Command::Fmi2SetReal {
                references: cmd.references,
                values: cmd.values,
            },
            fmi2_proto::fmi2_command::Command::Fmi2SetInteger(cmd) => Fmi2Command::Fmi2SetInteger {
                references: cmd.references,
                values: cmd.values,
            },
            fmi2_proto::fmi2_command::Command::Fmi2SetBoolean(cmd) => Fmi2Command::Fmi2SetBoolean {
                references: cmd.references,
                values: cmd.values,
            },
            fmi2_proto::fmi2_command::Command::Fmi2SetString(cmd) => Fmi2Command::Fmi2SetString {
                references: cmd.references,
                values: cmd.values,
            },

            fmi2_proto::fmi2_command::Command::Fmi2EnterInitializationMode(_) => {
                Fmi2Command::Fmi2EnterInitializationMode {}
            }
            fmi2_proto::fmi2_command::Command::Fmi2ExitInitializationMode(_) => {
                Fmi2Command::Fmi2ExitInitializationMode
            }
            fmi2_proto::fmi2_command::Command::Fmi2SetupExperiment(cmd) => {
                Fmi2Command::Fmi2SetupExperiment {
                    start_time: cmd.start_time,
                    stop_time: match cmd.has_stop_time {
                        true => Some(cmd.stop_time),
                        false => None,
                    },
                    tolerance: match cmd.has_tolerance {
                        true => Some(cmd.tolerance),
                        false => None,
                    },
                }
            }
            fmi2_proto::fmi2_command::Command::Fmi2FreeInstance(_) => {
                Fmi2Command::Fmi2FreeInstance {}
            }
            fmi2_proto::fmi2_command::Command::Fmi2GetReal(cmd) => Fmi2Command::Fmi2GetReal {
                references: cmd.references,
            },
            fmi2_proto::fmi2_command::Command::Fmi2GetInteger(cmd) => Fmi2Command::Fmi2GetInteger {
                references: cmd.references,
            },
            fmi2_proto::fmi2_command::Command::Fmi2GetBoolean(cmd) => Fmi2Command::Fmi2GetBoolean {
                references: cmd.references,
            },
            fmi2_proto::fmi2_command::Command::Fmi2GetString(cmd) => Fmi2Command::Fmi2GetString {
                references: cmd.references,
            },
            fmi2_proto::fmi2_command::Command::Fmi2Reset(_) => Fmi2Command::Fmi2Reset,
            fmi2_proto::fmi2_command::Command::Fmi2Terminate(_) => Fmi2Command::Fmi2Terminate,
            fmi2_proto::fmi2_command::Command::Fmi2CancelStep(_) => Fmi2Command::Fmi2CancelStep,
            fmi2_proto::fmi2_command::Command::Fmi2ExtSerializeSlave(_) => {
                Fmi2Command::Fmi2ExtSerializeSlave
            }
            fmi2_proto::fmi2_command::Command::Fmi2ExtDeserializeSlave(cmd) => {
                Fmi2Command::Fmi2ExtDeserializeSlave { state: cmd.state }
            }
        }
    }
}

// command: rust -> pb-inner
impl From<Fmi2Command> for fmi2_proto::fmi2_command::Command {
    fn from(val: Fmi2Command) -> Self {
        match val {
            Fmi2Command::Fmi2DoStep {
                current_time,
                step_size,
                no_step_prior,
            } => fmi2_proto::fmi2_command::Command::Fmi2DoStep({
                fmi2_proto::Fmi2DoStep {
                    current_time,
                    step_size,
                    no_step_prior,
                }
            }),

            Fmi2Command::Fmi2EnterInitializationMode => {
                fmi2_proto::fmi2_command::Command::Fmi2EnterInitializationMode({
                    fmi2_proto::Fmi2EnterInitializationMode {}
                })
            }
            Fmi2Command::Fmi2ExitInitializationMode => {
                fmi2_proto::fmi2_command::Command::Fmi2ExitInitializationMode({
                    fmi2_proto::Fmi2ExitInitializationMode {}
                })
            }
            Fmi2Command::Fmi2FreeInstance => {
                fmi2_proto::fmi2_command::Command::Fmi2FreeInstance(fmi2_proto::Fmi2FreeInstance {})
            }
            Fmi2Command::Fmi2SetupExperiment {
                start_time,
                stop_time,
                tolerance,
            } => fmi2_proto::fmi2_command::Command::Fmi2SetupExperiment(
                fmi2_proto::Fmi2SetupExperiment {
                    start_time,
                    stop_time: stop_time.unwrap_or_default(),
                    tolerance: tolerance.unwrap_or_default(),
                    has_stop_time: stop_time.is_some(),
                    has_tolerance: tolerance.is_some(),
                },
            ),
            Fmi2Command::Fmi2SetReal { references, values } => {
                fmi2_proto::fmi2_command::Command::Fmi2SetReal({
                    fmi2_proto::Fmi2SetReal {
                        references: references,
                        values: values,
                    }
                })
            }
            Fmi2Command::Fmi2SetInteger { references, values } => {
                fmi2_proto::fmi2_command::Command::Fmi2SetInteger({
                    fmi2_proto::Fmi2SetInteger { references, values }
                })
            }
            Fmi2Command::Fmi2SetBoolean { references, values } => {
                fmi2_proto::fmi2_command::Command::Fmi2SetBoolean({
                    fmi2_proto::Fmi2SetBoolean { references, values }
                })
            }
            Fmi2Command::Fmi2SetString { references, values } => {
                fmi2_proto::fmi2_command::Command::Fmi2SetString({
                    fmi2_proto::Fmi2SetString { references, values }
                })
            }
            Fmi2Command::Fmi2GetReal { references } => {
                fmi2_proto::fmi2_command::Command::Fmi2GetReal({
                    fmi2_proto::Fmi2GetReal { references }
                })
            }
            Fmi2Command::Fmi2GetInteger { references } => {
                fmi2_proto::fmi2_command::Command::Fmi2GetInteger({
                    fmi2_proto::Fmi2GetInteger { references }
                })
            }
            Fmi2Command::Fmi2GetBoolean { references } => {
                fmi2_proto::fmi2_command::Command::Fmi2GetBoolean({
                    fmi2_proto::Fmi2GetBoolean { references }
                })
            }
            Fmi2Command::Fmi2GetString { references } => {
                fmi2_proto::fmi2_command::Command::Fmi2GetString({
                    fmi2_proto::Fmi2GetString { references }
                })
            }
            Fmi2Command::Fmi2Reset => {
                fmi2_proto::fmi2_command::Command::Fmi2Reset(fmi2_proto::Fmi2Reset {})
            }
            Fmi2Command::Fmi2Terminate => {
                fmi2_proto::fmi2_command::Command::Fmi2Terminate(fmi2_proto::Fmi2Terminate {})
            }
            Fmi2Command::Fmi2CancelStep => {
                fmi2_proto::fmi2_command::Command::Fmi2CancelStep(fmi2_proto::Fmi2CancelStep {})
            }
            Fmi2Command::Fmi2ExtSerializeSlave => {
                fmi2_proto::fmi2_command::Command::Fmi2ExtSerializeSlave(
                    fmi2_proto::Fmi2ExtSerializeSlave {},
                )
            }
            Fmi2Command::Fmi2ExtDeserializeSlave { state } => {
                fmi2_proto::fmi2_command::Command::Fmi2ExtDeserializeSlave(
                    fmi2_proto::Fmi2ExtDeserializeSlave { state },
                )
            }
        }
    }
}

// command: rust -> pb-outer
impl From<Fmi2Command> for fmi2_proto::Fmi2Command {
    fn from(val: Fmi2Command) -> Self {
        let command = Some(fmi2_proto::fmi2_command::Command::from(val));
        Self { command }
    }
}
