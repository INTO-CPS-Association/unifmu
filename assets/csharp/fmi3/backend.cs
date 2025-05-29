
using System.IO;
using System;
using Fmi3Messages;
using UnifmuHandshake;
using System.Collections.Generic;
using NetMQ.Sockets;
using Google.Protobuf;
using NetMQ;
using System;
using System.Diagnostics;
using System.Linq;

namespace Launch
{
    class Program
    {


        public static void Main(string[] args)
        {
            Trace.Listeners.Add(new ConsoleTraceListener()); // Logs to console
            var references_to_attr = new Dictionary<uint, string>();
            Model model = null;

            string dispatcher_endpoint = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT");
            if (dispatcher_endpoint == null)
            {
                Console.Error.WriteLine("Environment variable 'UNIFMU_DISPATCHER_ENDPOINT' is not set in the current enviornment.");
                Environment.Exit(-1);
            }


            var socket = new RequestSocket();
            socket.Connect(dispatcher_endpoint);


            IMessage message = new HandshakeReply{
                Status = HandshakeStatus.Ok
            };


            socket.SendFrame(message.ToByteArray(), false);
            Fmi3Command command;

            while (true)
            {
                command = Fmi3Command.Parser.ParseFrom(socket.ReceiveFrameBytes());
                Trace.TraceInformation("Command: " + command);

                switch (command.CommandCase)
                {

                    case Fmi3Command.CommandOneofCase.Fmi3InstantiateCoSimulation:
                        {
                            model = new Model(
                                command.Fmi3InstantiateCoSimulation.InstanceName,
                                command.Fmi3InstantiateCoSimulation.InstantiationToken,
                                command.Fmi3InstantiateCoSimulation.ResourcePath,
                                command.Fmi3InstantiateCoSimulation.Visible,
                                command.Fmi3InstantiateCoSimulation.LoggingOn,
                                command.Fmi3InstantiateCoSimulation.EventModeUsed,
                                command.Fmi3InstantiateCoSimulation.EarlyReturnAllowed,
                                command.Fmi3InstantiateCoSimulation.RequiredIntermediateVariables.ToList()
                            );
                            var result = new Fmi3EmptyReturn();
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterInitializationMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3EnterInitializationMode();
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3ExitInitializationMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3ExitInitializationMode();
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterConfigurationMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3EnterConfigurationMode();
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3ExitConfigurationMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3ExitConfigurationMode();
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterEventMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3EnterEventMode();
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3EnterStepMode:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3EnterStepMode();
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3DoStep:
                        {
                            var result = new Fmi3DoStepReturn();
                            (var status, var event_handling_needed, var terminate_simulation, var early_return, var last_successful_time) = model.Fmi3DoStep(command.Fmi3DoStep.CurrentCommunicationPoint, command.Fmi3DoStep.CommunicationStepSize, command.Fmi3DoStep.NoSetFmuStatePriorToCurrentPoint);
                            result.Status = status;
                            result.EventHandlingNeeded = event_handling_needed;
                            result.TerminateSimulation = terminate_simulation;
                            result.EarlyReturn = early_return;
                            result.LastSuccessfulTime = last_successful_time;
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3UpdateDiscreteStates:
                        {
                            var result = new Fmi3UpdateDiscreteStatesReturn();
                            (var status, var discrete_states_need_update, var terminate_simulation, var nominals_continuous_states_changed, var values_continuous_states_changed, var next_event_time_defined, var next_event_time) = model.Fmi3UpdateDiscreteStates();
                            result.Status = status;
                            result.DiscreteStatesNeedUpdate = discrete_states_need_update;
                            result.TerminateSimulation = terminate_simulation;
                            result.NominalsContinuousStatesChanged = nominals_continuous_states_changed;
                            result.ValuesContinuousStatesChanged = values_continuous_states_changed;
                            result.NextEventTimeDefined = next_event_time_defined;
                            result.NextEventTime = next_event_time;
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetFloat32:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetFloat32(command.Fmi3SetFloat32.ValueReferences, command.Fmi3SetFloat32.Values);
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetFloat64:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetFloat64(command.Fmi3SetFloat64.ValueReferences, command.Fmi3SetFloat64.Values);
                            message = result;
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3SetInt8:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetInt8(
                                command.Fmi3SetInt8.ValueReferences,
                                command.Fmi3SetInt8.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt8:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetUInt8(
                                command.Fmi3SetUInt8.ValueReferences,
                                command.Fmi3SetUInt8.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt16:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetInt16(
                                command.Fmi3SetInt16.ValueReferences,
                                command.Fmi3SetInt16.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt16:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetUInt16(
                                command.Fmi3SetUInt16.ValueReferences,
                                command.Fmi3SetUInt16.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt32:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetInt32(
                                command.Fmi3SetInt32.ValueReferences,
                                command.Fmi3SetInt32.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt32:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetUInt32(
                                command.Fmi3SetUInt32.ValueReferences,
                                command.Fmi3SetUInt32.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetInt64:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetInt64(
                                command.Fmi3SetInt64.ValueReferences,
                                command.Fmi3SetInt64.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetUInt64:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetUInt64(
                                command.Fmi3SetUInt64.ValueReferences,
                                command.Fmi3SetUInt64.Values);
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetBoolean:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetBoolean(
                                command.Fmi3SetBoolean.ValueReferences,
                                command.Fmi3SetBoolean.Values
                            );
                            message = result;


                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetBinary:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetBinary(
                                command.Fmi3SetBinary.ValueReferences,
                                command.Fmi3SetBinary.ValueSizes,
                                ConvertToByteArrayList(command.Fmi3SetBinary.Values)
                            );
                            message = result;


                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetClock:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetClock(
                                command.Fmi3SetClock.ValueReferences,
                                command.Fmi3SetClock.Values
                            );
                            message = result;


                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetIntervalDecimal:
                        {
                            var c = command.Fmi3SetIntervalDecimal;
                            var res = model.Fmi3SetIntervalDecimal(c.ValueReferences, c.Intervals);
                            var result = new Fmi3StatusReturn
                            {
                                Status = res,
                            };
                            message = result;
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3SetIntervalFraction:
                        {
                            var c = command.Fmi3SetIntervalFraction;
                            var res = model.Fmi3SetIntervalFraction(c.ValueReferences, c.Counters, c.Resolutions);
                            var result = new Fmi3StatusReturn
                            {
                                Status = res,
                            };
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetShiftDecimal:
                        {
                            var c = command.Fmi3SetShiftDecimal;
                            var res = model.Fmi3SetShiftDecimal(c.ValueReferences, c.Shifts);
                            var result = new Fmi3StatusReturn
                            {
                                Status = res,
                            };
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SetShiftFraction:
                        {
                            var c = command.Fmi3SetShiftFraction;
                            var res = model.Fmi3SetShiftFraction(c.ValueReferences, c.Counters, c.Resolutions);
                            var result = new Fmi3StatusReturn
                            {
                                Status = res,
                            };
                            message = result;                        
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3SetString:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3SetString(command.Fmi3SetString.ValueReferences, command.Fmi3SetString.Values);
                            message = result;
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3GetFloat32:
                        {
                            var result = new Fmi3GetFloat32Return();
                            (var status, var values) = model.Fmi3GetFloat32(command.Fmi3GetFloat32.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetFloat64:
                        {
                            var result = new Fmi3GetFloat64Return();
                            (var status, var values) = model.Fmi3GetFloat64(command.Fmi3GetFloat64.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt8:
                        {
                            var result = new Fmi3GetInt8Return();
                            (var status, var values) = model.Fmi3GetInt8(command.Fmi3GetInt8.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt8:
                        {
                            var result = new Fmi3GetUInt8Return();
                            (var status, var values) = model.Fmi3GetUInt8(command.Fmi3GetUInt8.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt16:
                        {
                            var result = new Fmi3GetInt16Return();
                            (var status, var values) = model.Fmi3GetInt16(command.Fmi3GetInt16.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt16:
                        {
                            var result = new Fmi3GetUInt16Return();
                            (var status, var values) = model.Fmi3GetUInt16(command.Fmi3GetUInt16.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt32:
                        {
                            var result = new Fmi3GetInt32Return();
                            (var status, var values) = model.Fmi3GetInt32(command.Fmi3GetInt32.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt32:
                        {
                            var result = new Fmi3GetUInt32Return();
                            (var status, var values) = model.Fmi3GetUInt32(command.Fmi3GetUInt32.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetInt64:
                        {
                            var result = new Fmi3GetInt64Return();
                            (var status, var values) = model.Fmi3GetInt64(command.Fmi3GetInt64.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetUInt64:
                        {
                            var result = new Fmi3GetUInt64Return();
                            (var status, var values) = model.Fmi3GetUInt64(command.Fmi3GetUInt64.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetBoolean:
                        {
                            var result = new Fmi3GetBooleanReturn();
                            (var status, var values) = model.Fmi3GetBoolean(command.Fmi3GetBoolean.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetBinary:
                        {
                            var result = new Fmi3GetBinaryReturn();
                            (var status, var values) = model.Fmi3GetBinary(command.Fmi3GetBinary.ValueReferences);
                            result.Values.AddRange(ConvertToByteStringList(values));
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetClock:
                        {
                            var result = new Fmi3GetClockReturn();
                            (var status, var values) = model.Fmi3GetClock(command.Fmi3GetClock.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetString:
                        {
                            var result = new Fmi3GetStringReturn();
                            (var status, var values) = model.Fmi3GetString(command.Fmi3GetString.ValueReferences);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetIntervalDecimal:
                        {
                            var c = command.Fmi3GetIntervalDecimal;
                            (var status, var intervals, var qualifiers) = model.Fmi3GetIntervalDecimal(c.ValueReferences);
                            var result = new Fmi3GetIntervalDecimalReturn();
                            result.Status = status;
                            result.Intervals.AddRange(intervals);
                            result.Qualifiers.AddRange(qualifiers);
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetIntervalFraction:
                        {
                            var c = command.Fmi3GetIntervalFraction;
                            (var status, var counters, var resolutions, var qualifiers) = model.Fmi3GetIntervalFraction(c.ValueReferences);
                            var result = new Fmi3GetIntervalFractionReturn();
                            result.Status = status;
                            result.Counters.AddRange(counters);
                            result.Resolutions.AddRange(resolutions);
                            result.Qualifiers.AddRange(qualifiers);
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetShiftDecimal:
                        {
                            var c = command.Fmi3GetShiftDecimal;
                            (var status, var shifts) = model.Fmi3GetShiftDecimal(c.ValueReferences);
                            var result = new Fmi3GetShiftDecimalReturn();
                            result.Status = status;
                            result.Shifts.AddRange(shifts);
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3GetShiftFraction:
                        {
                            var c = command.Fmi3GetShiftFraction;
                            (var status, var counters, var resolutions) = model.Fmi3GetShiftFraction(c.ValueReferences);
                            var result = new Fmi3GetShiftFractionReturn();
                            result.Status = status;
                            result.Counters.AddRange(counters);
                            result.Resolutions.AddRange(resolutions);
                            message = result;
                        }
                        break;


                    case Fmi3Command.CommandOneofCase.Fmi3Reset:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3Reset();
                            message = result;

                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3Terminate:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3Terminate();
                            message = result;
                        }
                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3SerializeFmuState:
                        {
                            var result = new Fmi3SerializeFmuStateReturn();
                            (var status, var state) = model.Fmi3SerializeFmuState();
                            result.Status = status;
                            result.State = ByteString.CopyFrom(state);
                            message = result;
                        }
                        break;
                    case Fmi3Command.CommandOneofCase.Fmi3DeserializeFmuState:
                        {
                            var result = new Fmi3StatusReturn();
                            result.Status = model.Fmi3DeserializeFmuState(command.Fmi3DeserializeFmuState.State.ToByteArray());
                            message = result;
                        }

                        break;

                    case Fmi3Command.CommandOneofCase.Fmi3FreeInstance:
                        {
                            Console.WriteLine("received fmi3FreeInstance, exiting with status code 0");
                            Environment.Exit(0);
                        }


                        break;

                    default:
                        Console.Error.WriteLine("unrecognized command {0}, exiting with status code -1", command.CommandCase);
                        Environment.Exit(-1);
                        break;
                }


                socket.SendFrame(message.ToByteArray(), false);
            }

        }

        public static IEnumerable<byte[]> ConvertToByteArrayList(IEnumerable<ByteString> byteStrings)
        {
            var byteArrayList = new List<byte[]>();
            foreach (var bs in byteStrings)
            {
                byteArrayList.Add(bs.ToByteArray());
            }
            return byteArrayList;
        }

        public static IEnumerable<ByteString> ConvertToByteStringList(IEnumerable<byte[]> byteArrays)
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