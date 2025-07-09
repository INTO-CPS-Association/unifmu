using System;
using Fmi2Messages;
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

        private static Fmi2Command RecvCommand()
        {
            return Fmi2Command.Parser.ParseFrom(socket.ReceiveFrameBytes());
        }

        private static void SendStatusReply(Fmi2Status status)
        {
            SendReply(
                new Fmi2Return{
                    Status = new Fmi2StatusReturn{Status = status}
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

            LogCallback logCallback = (status, category, message) => {
                SendReply(new Fmi2Return{Log = new Fmi2LogReturn{
                    Status = status,
                    Category = category,
                    LogMessage = message
                }});

                Fmi2Command command = RecvCommand();

                switch (command.CommandCase)
                {
                    case Fmi2Command.CommandOneofCase.Fmi2CallbackContinue:
                        break;
                    default:
                        HandleUnexpectedCommand(command);
                        break;
                }
            };

            while (true)
            {
                Fmi2Command command = RecvCommand();

                switch (command.CommandCase)
                {
                    case Fmi2Command.CommandOneofCase.Fmi2Instantiate:
                        model = new Model(logCallback);
                        SendReply(new Fmi2Return{Empty = new Fmi2EmptyReturn()});
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetupExperiment:
                        SendStatusReply(model.Fmi2SetupExperiment(
                            command.Fmi2SetupExperiment.StartTime,
                            command.Fmi2SetupExperiment.HasStopTime ? command.Fmi2SetupExperiment.StopTime : null,
                            command.Fmi2SetupExperiment.HasTolerance ? command.Fmi2SetupExperiment.Tolerance : null
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2EnterInitializationMode:
                        SendStatusReply(model.Fmi2EnterInitializationMode());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2ExitInitializationMode:
                        SendStatusReply(model.Fmi2ExitInitializationMode());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DoStep:
                        SendStatusReply(model.Fmi2DoStep(
                            command.Fmi2DoStep.CurrentTime,
                            command.Fmi2DoStep.StepSize,
                            command.Fmi2DoStep.NoSetFmuStatePriorToCurrentPoint
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetReal:
                        SendStatusReply(model.Fmi2SetReal(
                            command.Fmi2SetReal.References,
                            command.Fmi2SetReal.Values
                        ));
                        break;


                    case Fmi2Command.CommandOneofCase.Fmi2SetInteger:
                        SendStatusReply(model.Fmi2SetInteger(
                            command.Fmi2SetInteger.References,
                            command.Fmi2SetInteger.Values
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetBoolean:
                        SendStatusReply(model.Fmi2SetBoolean(
                            command.Fmi2SetBoolean.References,
                            command.Fmi2SetBoolean.Values
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetString:
                        SendStatusReply(model.Fmi2SetString(
                            command.Fmi2SetString.References,
                            command.Fmi2SetString.Values
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetReal:
                        {
                            Fmi2Return result = new Fmi2Return{GetReal = new Fmi2GetRealReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetReal(command.Fmi2GetReal.References);
                            result.GetReal.Values.AddRange(values);
                            result.GetReal.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetInteger:
                        {
                            Fmi2Return result = new Fmi2Return{GetInteger = new Fmi2GetIntegerReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetInteger(command.Fmi2GetInteger.References);
                            result.GetInteger.Values.AddRange(values);
                            result.GetInteger.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetBoolean:
                        {
                            Fmi2Return result = new Fmi2Return{GetBoolean = new Fmi2GetBooleanReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetBoolean(command.Fmi2GetBoolean.References);
                            result.GetBoolean.Values.AddRange(values);
                            result.GetBoolean.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetString:
                        {
                            Fmi2Return result = new Fmi2Return{GetString = new Fmi2GetStringReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetString(command.Fmi2GetString.References);
                            result.GetString.Values.AddRange(values);
                            result.GetString.Status = status;
                            SendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2CancelStep:
                        SendStatusReply(model.Fmi2CancelStep());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Reset:
                        SendStatusReply(model.Fmi2Reset());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Terminate:
                        SendStatusReply(model.Fmi2Terminate());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SerializeFmuState:
                        {
                            Fmi2Return result = new Fmi2Return{SerializeFmuState = new Fmi2SerializeFmuStateReturn()};
                            (Fmi2Status status, var state) = model.Fmi2SerializeFmuState();
                            result.SerializeFmuState.Status = status;
                            result.SerializeFmuState.State = ByteString.CopyFrom(state);
                            SendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DeserializeFmuState:
                        SendStatusReply(model.Fmi2DeserializeFmuState(command.Fmi2DeserializeFmuState.State.ToByteArray()));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2FreeInstance:
                        SendReply(new Fmi2Return{FreeInstance = new Fmi2FreeInstanceReturn()});
                        Console.WriteLine("received fmi2FreeInstance, exiting with status code 0");
                        Environment.Exit(0);
                        break;

                    default:
                        HandleUnexpectedCommand(command);
                        break;
                }
            }
        }

        private static void HandleUnexpectedCommand(Fmi2Command command)
        {
            Console.Error.WriteLine("unexpected command {0}, exiting with status code -1", command.CommandCase);
            Environment.Exit(-1);
        }
    }
}