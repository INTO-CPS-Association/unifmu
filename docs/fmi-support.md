# FMI function support

[User guide](../README.md) | [Development guide](../development/README.md)

UniFMU supports FMI2 and FMI3 Co-Simulation with Python, C#, and Java backends.
FMI1, Model Exchange, and Scheduled Execution are not supported.

The tables below retain the documented native API support. A model must also
implement the behavior it advertises through capability flags in
`modelDescription.xml`; an available API function does not make every model
support that capability automatically.

## FMI2

| Function | Supported |
| --- | --- |
| fmi2GetTypesPlatform | Yes |
| fmi2GetVersion | Yes |
| fmi2SetDebugLogging | Yes |
| fmi2Instantiate | Yes |
| fmi2FreeInstance | Yes |
| fmi2SetupExperiment | Yes |
| fmi2EnterInitializationMode | Yes |
| fmi2ExitInitializationMode | Yes |
| fmi2Terminate | Yes |
| fmi2Reset | Yes |
| fmi2GetReal | Yes |
| fmi2GetInteger | Yes |
| fmi2GetBoolean | Yes |
| fmi2GetString | Yes |
| fmi2SetReal | Yes |
| fmi2SetInteger | Yes |
| fmi2SetBoolean | Yes |
| fmi2SetString | Yes |
| fmi2GetFMUstate | Yes |
| fmi2SetFMUstate | Yes |
| fmi2FreeFMUstate | Yes |
| fmi2SerializedFMUstateSize | Yes |
| fmi2SerializeFMUstate | Yes |
| fmi2DeSerializeFMUstate | Yes |
| fmi2GetDirectionalDerivative | No |
| fmi2EnterEventMode | No |
| fmi2NewDiscreteStates | No |
| fmi2EnterContinuousTimeMode | No |
| fmi2CompletedIntegratorStep | No |
| fmi2SetTime | No |
| fmi2SetContinuousStates | No |
| fmi2GetDerivatives | No |
| fmi2GetEventIndicators | No |
| fmi2GetContinuousStates | No |
| fmi2GetNominalsOfContinuousStates | No |
| fmi2SetRealInputDerivatives | No |
| fmi2GetRealOutputDerivatives | No |
| fmi2DoStep | Yes |
| fmi2CancelStep | No |
| fmi2GetStatus | No |
| fmi2GetRealStatus | No |
| fmi2GetIntegerStatus | No |
| fmi2GetBooleanStatus | No |
| fmi2GetStringStatus | No |

## FMI3

| Function | Supported |
| --- | --- |
| fmi3GetVersion | Yes |
| fmi3SetDebugLogging | Yes |
| fmi3InstantiateModelExchange | No |
| fmi3InstantiateCoSimulation | Yes |
| fmi3InstantiateScheduledExecution | No |
| fmi3FreeInstance | Yes |
| fmi3EnterInitializationMode | Yes |
| fmi3ExitInitializationMode | Yes |
| fmi3EnterEventMode | Yes |
| fmi3Terminate | Yes |
| fmi3Reset | Yes |
| fmi3GetFloat32 | Yes |
| fmi3GetFloat64 | Yes |
| fmi3GetInt8 | Yes |
| fmi3GetUInt8 | Yes |
| fmi3GetInt16 | Yes |
| fmi3GetUInt16 | Yes |
| fmi3GetInt32 | Yes |
| fmi3GetUInt32 | Yes |
| fmi3GetInt64 | Yes |
| fmi3GetUInt64 | Yes |
| fmi3GetBoolean | Yes |
| fmi3GetString | Yes |
| fmi3GetBinary | Yes |
| fmi3GetClock | Yes |
| fmi3SetFloat32 | Yes |
| fmi3SetFloat64 | Yes |
| fmi3SetInt8 | Yes |
| fmi3SetUInt8 | Yes |
| fmi3SetInt16 | Yes |
| fmi3SetUInt16 | Yes |
| fmi3SetInt32 | Yes |
| fmi3SetUInt32 | Yes |
| fmi3SetInt64 | Yes |
| fmi3SetUInt64 | Yes |
| fmi3SetBoolean | Yes |
| fmi3SetString | Yes |
| fmi3SetBinary | Yes |
| fmi3SetClock | Yes |
| fmi3GetNumberOfVariableDependencies | No |
| fmi3GetVariableDependencies | No |
| fmi3GetFMUState | Yes |
| fmi3SetFMUState | Yes |
| fmi3FreeFMUState | Yes |
| fmi3SerializedFMUStateSize | Yes |
| fmi3SerializeFMUState | Yes |
| fmi3DeserializeFMUState | Yes |
| fmi3GetDirectionalDerivative | No |
| fmi3GetAdjointDerivative | No |
| fmi3EnterConfigurationMode | Yes |
| fmi3ExitConfigurationMode | Yes |
| fmi3GetIntervalDecimal | Yes |
| fmi3GetIntervalFraction | Yes |
| fmi3GetShiftDecimal | Yes |
| fmi3GetShiftFraction | Yes |
| fmi3SetIntervalDecimal | Yes |
| fmi3SetIntervalFraction | Yes |
| fmi3SetShiftDecimal | Yes |
| fmi3SetShiftFraction | Yes |
| fmi3EvaluateDiscreteStates | No |
| fmi3UpdateDiscreteStates | Yes |
| fmi3EnterContinuousTimeMode | No |
| fmi3CompletedIntegratorStep | No |
| fmi3SetTime | No |
| fmi3SetContinuousStates | No |
| fmi3GetContinuousStateDerivatives | No |
| fmi3GetEventIndicators | No |
| fmi3GetContinuousStates | No |
| fmi3GetNominalsOfContinuousStates | No |
| fmi3GetNumberOfEventIndicators | No |
| fmi3GetNumberOfContinuousStates | No |
| fmi3EnterStepMode | Yes |
| fmi3GetOutputDerivatives | No |
| fmi3DoStep | Yes |
| fmi3ActivateModelPartition | No |
