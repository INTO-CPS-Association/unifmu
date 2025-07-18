using System;
using Fmi3Messages;
using UnifmuHandshake;
using System.Collections.Generic;
using NetMQ.Sockets;
using Google.Protobuf;
using NetMQ;

namespace Launch
{
    partial class Program
    {
        private static RequestSocket socket = new RequestSocket();

        private static void ConnectToEndpoint(string dispatcher_endpoint)
        {
            socket.Connect(dispatcher_endpoint);
        }

        private static void SendReply(IMessage reply)
        {
            socket.SendFrame(reply.ToByteArray(), false);
        }

        private static Fmi3Command RecvCommand()
        {
            return Fmi3Command.Parser.ParseFrom(socket.ReceiveFrameBytes());
        }

        private static void SendStatusReply(Fmi3Status status)
        {
            SendReply(
                new Fmi3Return{
                    Status = new Fmi3StatusReturn{Status = status}
                }
            );
        }

        private static void Handshake()
        {
            SendReply(new HandshakeReply{Status = HandshakeStatus.Ok});
        }

        private static void CommandReplyLoop()
        {
            Model model = null;

            while (true)
            {
                Fmi3Command command = RecvCommand();

                switch (command.CommandCase)
                {
                    case Fmi3Command.CommandOneofCase.Fmi3InstantiateCoSimulation:
                        uint[] req_int_vars_array = new uint[command.Fmi3InstantiateCoSimulation.RequiredIntermediateVariables.Count];
                        command.Fmi3InstantiateCoSimulation.RequiredIntermediateVariables.CopyTo(req_int_vars_array, 0);
                        List<uint> req_int_vars = new();
                        req_int_vars.AddRange(req_int_vars_array);

                        model = new Model(
                            command.Fmi3InstantiateCoSimulation.InstanceName,
                            command.Fmi3InstantiateCoSimulation.InstantiationToken,
                            command.Fmi3InstantiateCoSimulation.ResourcePath,
                            command.Fmi3InstantiateCoSimulation.Visible,
                            command.Fmi3InstantiateCoSimulation.LoggingOn,
                            command.Fmi3InstantiateCoSimulation.EventModeUsed,
                            command.Fmi3InstantiateCoSimulation.EarlyReturnAllowed,
                            req_int_vars
                        );

                        SendReply(new Fmi3Return{Empty = new Fmi3EmptyReturn()});
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterInitializationMode:
                        SendStatusReply(model.Fmi3EnterInitializationMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3ExitInitializationMode:
                        SendStatusReply(model.Fmi3ExitInitializationMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterConfigurationMode:
                        SendStatusReply(model.Fmi3EnterConfigurationMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3ExitConfigurationMode:
                        SendStatusReply(model.Fmi3ExitConfigurationMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterEventMode:
                        SendStatusReply(model.Fmi3EnterEventMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterStepMode:
                        SendStatusReply(model.Fmi3EnterStepMode());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3DoStep:
                        {
                            Fmi3Return result = new Fmi3Return {
                                DoStep = new Fmi3DoStepReturn()
                            };
                            (
                                var status,
                                var event_handling_needed,
                                var terminate_simulation,
                                var early_return,
                                var last_successful_time
                            ) = model.Fmi3DoStep(
                                command.Fmi3DoStep.CurrentCommunicationPoint,
                                command.Fmi3DoStep.CommunicationStepSize,
                                command.Fmi3DoStep.NoSetFmuStatePriorToCurrentPoint
                            );
                            result.DoStep.Status = status;
                            result.DoStep.EventHandlingNeeded = event_handling_needed;
                            result.DoStep.TerminateSimulation = terminate_simulation;
                            result.DoStep.EarlyReturn = early_return;
                            result.DoStep.LastSuccessfulTime = last_successful_time;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3UpdateDiscreteStates:
                        {
                            Fmi3Return result = new Fmi3Return {
                                UpdateDiscreteStates = new Fmi3UpdateDiscreteStatesReturn()
                            };
                            (
                                var status,
                                var discrete_states_need_update,
                                var terminate_simulation,
                                var nominals_continuous_states_changed,
                                var values_continuous_states_changed,
                                var next_event_time_defined,
                                var next_event_time
                            ) = model.Fmi3UpdateDiscreteStates();
                            result.UpdateDiscreteStates.Status = status;
                            result.UpdateDiscreteStates.DiscreteStatesNeedUpdate = discrete_states_need_update;
                            result.UpdateDiscreteStates.TerminateSimulation = terminate_simulation;
                            result.UpdateDiscreteStates.NominalsContinuousStatesChanged = nominals_continuous_states_changed;
                            result.UpdateDiscreteStates.ValuesContinuousStatesChanged = values_continuous_states_changed;
                            result.UpdateDiscreteStates.NextEventTimeDefined = next_event_time_defined;
                            result.UpdateDiscreteStates.NextEventTime = next_event_time;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetFloat32:
                        SendStatusReply(
                            model.Fmi3SetFloat32(
                                command.Fmi3SetFloat32.ValueReferences,
                                command.Fmi3SetFloat32.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetFloat64:
                        SendStatusReply(
                            model.Fmi3SetFloat64(
                                command.Fmi3SetFloat64.ValueReferences,
                                command.Fmi3SetFloat64.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt8:
                        SendStatusReply(
                            model.Fmi3SetInt8(
                                command.Fmi3SetInt8.ValueReferences,
                                command.Fmi3SetInt8.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt8:
                        SendStatusReply(
                            model.Fmi3SetUInt8(
                                command.Fmi3SetUInt8.ValueReferences,
                                command.Fmi3SetUInt8.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt16:
                        SendStatusReply(
                            model.Fmi3SetInt16(
                                command.Fmi3SetInt16.ValueReferences,
                                command.Fmi3SetInt16.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt16:
                        SendStatusReply(
                            model.Fmi3SetUInt16(
                                command.Fmi3SetUInt16.ValueReferences,
                                command.Fmi3SetUInt16.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt32:
                        SendStatusReply(
                            model.Fmi3SetInt32(
                                command.Fmi3SetInt32.ValueReferences,
                                command.Fmi3SetInt32.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt32:
                        SendStatusReply(
                            model.Fmi3SetUInt32(
                                command.Fmi3SetUInt32.ValueReferences,
                                command.Fmi3SetUInt32.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt64:
                        SendStatusReply(
                            model.Fmi3SetInt64(
                                command.Fmi3SetInt64.ValueReferences,
                                command.Fmi3SetInt64.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt64:
                        SendStatusReply(
                            model.Fmi3SetUInt64(
                                command.Fmi3SetUInt64.ValueReferences,
                                command.Fmi3SetUInt64.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetBoolean:
                        SendStatusReply(
                            model.Fmi3SetBoolean(
                                command.Fmi3SetBoolean.ValueReferences,
                                command.Fmi3SetBoolean.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetBinary:
                        SendStatusReply(
                            model.Fmi3SetBinary(
                                command.Fmi3SetBinary.ValueReferences,
                                command.Fmi3SetBinary.ValueSizes,
                                ConvertToByteArrayList(command.Fmi3SetBinary.Values)
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetClock:
                        SendStatusReply(
                            model.Fmi3SetClock(
                                command.Fmi3SetClock.ValueReferences,
                                command.Fmi3SetClock.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetIntervalDecimal:
                        SendStatusReply(
                            model.Fmi3SetIntervalDecimal(
                                command.Fmi3SetIntervalDecimal.ValueReferences,
                                command.Fmi3SetIntervalDecimal.Intervals
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetIntervalFraction:
                        SendStatusReply(
                            model.Fmi3SetIntervalFraction(
                                command.Fmi3SetIntervalFraction.ValueReferences,
                                command.Fmi3SetIntervalFraction.Counters,
                                command.Fmi3SetIntervalFraction.Resolutions
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetShiftDecimal:
                        SendStatusReply(
                            model.Fmi3SetShiftDecimal(
                                command.Fmi3SetShiftDecimal.ValueReferences,
                                command.Fmi3SetShiftDecimal.Shifts
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetShiftFraction:
                        SendStatusReply(
                            model.Fmi3SetShiftFraction(
                                command.Fmi3SetShiftFraction.ValueReferences,
                                command.Fmi3SetShiftFraction.Counters,
                                command.Fmi3SetShiftFraction.Resolutions
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetString:
                        SendStatusReply(
                            model.Fmi3SetString(
                                command.Fmi3SetString.ValueReferences,
                                command.Fmi3SetString.Values
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetFloat32:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetFloat32 = new Fmi3GetFloat32Return()
                            };
                            (var status, var values) = model.Fmi3GetFloat32(
                                command.Fmi3GetFloat32.ValueReferences
                            );
                            result.GetFloat32.Values.AddRange(values);
                            result.GetFloat32.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetFloat64:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetFloat64 = new Fmi3GetFloat64Return()
                            };
                            (var status, var values) = model.Fmi3GetFloat64(
                                command.Fmi3GetFloat64.ValueReferences
                            );
                            result.GetFloat64.Values.AddRange(values);
                            result.GetFloat64.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt8:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetInt8 = new Fmi3GetInt8Return()
                            };
                            (var status, var values) = model.Fmi3GetInt8(
                                command.Fmi3GetInt8.ValueReferences
                            );
                            result.GetInt8.Values.AddRange(values);
                            result.GetInt8.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt8:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetUInt8 = new Fmi3GetUInt8Return()
                            };
                            (var status, var values) = model.Fmi3GetUInt8(
                                command.Fmi3GetUInt8.ValueReferences
                            );
                            result.GetUInt8.Values.AddRange(values);
                            result.GetUInt8.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt16:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetInt16 = new Fmi3GetInt16Return()
                            };
                            (var status, var values) = model.Fmi3GetInt16(
                                command.Fmi3GetInt16.ValueReferences
                            );
                            result.GetInt16.Values.AddRange(values);
                            result.GetInt16.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt16:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetUInt16 = new Fmi3GetUInt16Return()
                            };
                            (var status, var values) = model.Fmi3GetUInt16(
                                command.Fmi3GetUInt16.ValueReferences
                            );
                            result.GetUInt16.Values.AddRange(values);
                            result.GetUInt16.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt32:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetInt32 = new Fmi3GetInt32Return()
                            };
                            (var status, var values) = model.Fmi3GetInt32(
                                command.Fmi3GetInt32.ValueReferences
                            );
                            result.GetInt32.Values.AddRange(values);
                            result.GetInt32.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt32:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetUInt32 = new Fmi3GetUInt32Return()
                            };
                            (var status, var values) = model.Fmi3GetUInt32(
                                command.Fmi3GetUInt32.ValueReferences
                            );
                            result.GetUInt32.Values.AddRange(values);
                            result.GetUInt32.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt64:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetInt64 = new Fmi3GetInt64Return()
                            };
                            (var status, var values) = model.Fmi3GetInt64(
                                command.Fmi3GetInt64.ValueReferences
                            );
                            result.GetInt64.Values.AddRange(values);
                            result.GetInt64.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt64:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetUInt64 = new Fmi3GetUInt64Return()
                            };
                            (var status, var values) = model.Fmi3GetUInt64(
                                command.Fmi3GetUInt64.ValueReferences
                            );
                            result.GetUInt64.Values.AddRange(values);
                            result.GetUInt64.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetBoolean:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetBoolean = new Fmi3GetBooleanReturn()
                            };
                            (var status, var values) = model.Fmi3GetBoolean(
                                command.Fmi3GetBoolean.ValueReferences
                            );
                            result.GetBoolean.Values.AddRange(values);
                            result.GetBoolean.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetBinary:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetBinary = new Fmi3GetBinaryReturn()
                            };
                            (var status, var values) = model.Fmi3GetBinary(
                                command.Fmi3GetBinary.ValueReferences
                            );
                            result.GetBinary.Values.AddRange(ConvertToByteStringList(values));
                            result.GetBinary.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetClock:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetClock = new Fmi3GetClockReturn()
                            };
                            (var status, var values) = model.Fmi3GetClock(
                                command.Fmi3GetClock.ValueReferences
                            );
                            result.GetClock.Values.AddRange(values);
                            result.GetClock.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetString:
                        {
                            Fmi3Return result = new Fmi3Return {
                                GetString = new Fmi3GetStringReturn()
                            };
                            (var status, var values) = model.Fmi3GetString(
                                command.Fmi3GetString.ValueReferences
                            );
                            result.GetString.Values.AddRange(values);
                            result.GetString.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetIntervalDecimal:
                        {
                            (
                                var status,
                                var intervals,
                                var qualifiers
                            ) = model.Fmi3GetIntervalDecimal(
                                command.Fmi3GetIntervalDecimal.ValueReferences
                            );
                            Fmi3Return result = new Fmi3Return {
                                GetIntervalDecimal = new Fmi3GetIntervalDecimalReturn()
                            };
                            result.GetIntervalDecimal.Status = status;
                            result.GetIntervalDecimal.Intervals.AddRange(intervals);
                            result.GetIntervalDecimal.Qualifiers.AddRange(qualifiers);
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetIntervalFraction:
                        {
                            (
                                var status,
                                var counters,
                                var resolutions,
                                var qualifiers
                            ) = model.Fmi3GetIntervalFraction(
                                command.Fmi3GetIntervalFraction.ValueReferences
                            );
                            Fmi3Return result = new Fmi3Return {
                                GetIntervalFraction = new Fmi3GetIntervalFractionReturn()
                            };
                            result.GetIntervalFraction.Status = status;
                            result.GetIntervalFraction.Counters.AddRange(counters);
                            result.GetIntervalFraction.Resolutions.AddRange(resolutions);
                            result.GetIntervalFraction.Qualifiers.AddRange(qualifiers);
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetShiftDecimal:
                        {
                            (var status, var shifts) = model.Fmi3GetShiftDecimal(
                                command.Fmi3GetShiftDecimal.ValueReferences
                            );
                            Fmi3Return result = new Fmi3Return {
                                GetShiftDecimal = new Fmi3GetShiftDecimalReturn()
                            };
                            result.GetShiftDecimal.Status = status;
                            result.GetShiftDecimal.Shifts.AddRange(shifts);
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetShiftFraction:
                        {
                            (
                                var status,
                                var counters,
                                var resolutions
                            ) = model.Fmi3GetShiftFraction(
                                command.Fmi3GetShiftFraction.ValueReferences
                            );
                            Fmi3Return result = new Fmi3Return {
                                GetShiftFraction = new Fmi3GetShiftFractionReturn()
                            };
                            result.GetShiftFraction.Status = status;
                            result.GetShiftFraction.Counters.AddRange(counters);
                            result.GetShiftFraction.Resolutions.AddRange(resolutions);
                            SendReply(result);
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3Reset:
                        SendStatusReply(model.Fmi3Reset());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3Terminate:
                        SendStatusReply(model.Fmi3Terminate());
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SerializeFmuState:
                        {
                            Fmi3Return result = new Fmi3Return {
                                SerializeFmuState = new Fmi3SerializeFmuStateReturn()
                            };
                            (var status, var state) = model.Fmi3SerializeFmuState();
                            result.SerializeFmuState.Status = status;
                            result.SerializeFmuState.State = ByteString.CopyFrom(state);
                            SendReply(result);
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3DeserializeFmuState:
                        SendStatusReply(
                            model.Fmi3DeserializeFmuState(
                                command.Fmi3DeserializeFmuState.State.ToByteArray()
                            )
                        );
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3FreeInstance:
                        {
                            Console.WriteLine("received fmi3FreeInstance, exiting with status code 0");
                            Environment.Exit(0);
                        }
                        break;

                    default:
                        HandleUnexpectedCommand(command);
                        break;
                }
            }
        }

        private static void HandleUnexpectedCommand(Fmi3Command command)
        {
            Console.Error.WriteLine(
                "unexpected command {0}, exiting with status code -1",
                command.CommandCase
            );
            Environment.Exit(-1);
        }

        private static IEnumerable<byte[]> ConvertToByteArrayList(IEnumerable<ByteString> byteStrings)
        {
            var byteArrayList = new List<byte[]>();
            foreach (var bs in byteStrings)
            {
                byteArrayList.Add(bs.ToByteArray());
            }
            return byteArrayList;
        }

        private static IEnumerable<ByteString> ConvertToByteStringList(IEnumerable<byte[]> byteArrays)
        {
            var byteStringList = new List<ByteString>();
            foreach (var arr in byteArrays)
            {
                byteStringList.Add(ByteString.CopyFrom(arr));
            }
            return byteStringList;
        }
    }
}