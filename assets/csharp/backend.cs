
using System.IO;
using System;
using Fmi2Messages;
using UnifmuHandshake;
using System.Collections.Generic;
using NetMQ.Sockets;
using Google.Protobuf;
using NetMQ;


namespace Launch
{
    delegate Fmi2Command RecvCommand();

    delegate void SendReply(IMessage reply);

    delegate void SendStatusReply(Fmi2Status status);

    class Program
    {
        public static void Main(string[] args)
        {
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

            SendReply sendReply = reply => socket.SendFrame(reply.ToByteArray(), false);

            sendReply(new HandshakeReply{Status = HandshakeStatus.Ok});

            RecvCommand recvCommand = () => Fmi2Command.Parser.ParseFrom(socket.ReceiveFrameBytes());

            SendStatusReply sendStatusReply = status => sendReply(
                new Fmi2Return{Status = new Fmi2StatusReturn{Status = status}}
            );

            LogCallback logCallback = (status, category, message) => {
                sendReply(new Fmi2Return{Log = new Fmi2LogReturn{
                    Status = status,
                    Category = category,
                    LogMessage = message
                }});

                Fmi2Command command = recvCommand();

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
                Fmi2Command command = recvCommand();

                switch (command.CommandCase)
                {
                    case Fmi2Command.CommandOneofCase.Fmi2Instantiate:
                        model = new Model(logCallback);
                        sendReply(new Fmi2Return{Empty = new Fmi2EmptyReturn()});
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetupExperiment:
                        sendStatusReply(model.Fmi2SetupExperiment(
                            command.Fmi2SetupExperiment.StartTime,
                            command.Fmi2SetupExperiment.HasStopTime ? command.Fmi2SetupExperiment.StopTime : null,
                            command.Fmi2SetupExperiment.HasTolerance ? command.Fmi2SetupExperiment.Tolerance : null
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2EnterInitializationMode:
                        sendStatusReply(model.Fmi2EnterInitializationMode());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2ExitInitializationMode:
                        sendStatusReply(model.Fmi2ExitInitializationMode());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DoStep:
                        sendStatusReply(model.Fmi2DoStep(command.Fmi2DoStep.CurrentTime, command.Fmi2DoStep.StepSize, command.Fmi2DoStep.NoSetFmuStatePriorToCurrentPoint));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetReal:
                        sendStatusReply(model.Fmi2SetReal(command.Fmi2SetReal.References, command.Fmi2SetReal.Values));
                        break;


                    case Fmi2Command.CommandOneofCase.Fmi2SetInteger:
                        sendStatusReply(model.Fmi2SetInteger(
                            command.Fmi2SetInteger.References,
                            command.Fmi2SetInteger.Values
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetBoolean:
                        sendStatusReply(model.Fmi2SetBoolean(
                            command.Fmi2SetBoolean.References,
                            command.Fmi2SetBoolean.Values
                        ));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetString:
                        sendStatusReply(model.Fmi2SetString(command.Fmi2SetString.References, command.Fmi2SetString.Values));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetReal:
                        {
                            Fmi2Return result = new Fmi2Return{GetReal = new Fmi2GetRealReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetReal(command.Fmi2GetReal.References);
                            result.GetReal.Values.AddRange(values);
                            result.GetReal.Status = status;
                            sendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetInteger:
                        {
                            Fmi2Return result = new Fmi2Return{GetInteger = new Fmi2GetIntegerReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetInteger(command.Fmi2GetInteger.References);
                            result.GetInteger.Values.AddRange(values);
                            result.GetInteger.Status = status;
                            sendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetBoolean:
                        {
                            Fmi2Return result = new Fmi2Return{GetBoolean = new Fmi2GetBooleanReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetBoolean(command.Fmi2GetBoolean.References);
                            result.GetBoolean.Values.AddRange(values);
                            result.GetBoolean.Status = status;
                            sendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetString:
                        {
                            Fmi2Return result = new Fmi2Return{GetString = new Fmi2GetStringReturn()};
                            (Fmi2Status status, var values) = model.Fmi2GetString(command.Fmi2GetString.References);
                            result.GetString.Values.AddRange(values);
                            result.GetString.Status = status;
                            sendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2CancelStep:
                        sendStatusReply(model.Fmi2CancelStep());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Reset:
                        sendStatusReply(model.Fmi2Reset());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Terminate:
                        sendStatusReply(model.Fmi2Terminate());
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SerializeFmuState:
                        {
                            Fmi2Return result = new Fmi2Return{SerializeFmuState = new Fmi2SerializeFmuStateReturn()};
                            (Fmi2Status status, var state) = model.Fmi2SerializeFmuState();
                            result.SerializeFmuState.Status = status;
                            result.SerializeFmuState.State = ByteString.CopyFrom(state);
                            sendReply(result);
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DeserializeFmuState:
                        sendStatusReply(model.Fmi2DeserializeFmuState(command.Fmi2DeserializeFmuState.State.ToByteArray()));
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2FreeInstance:
                        sendReply(new Fmi2Return{FreeInstance = new Fmi2FreeInstanceReturn()});
                        Console.WriteLine("received fmi2FreeInstance, exiting with status code 0");
                        Environment.Exit(0);
                        break;

                    default:
                        HandleUnexpectedCommand(command);
                        break;
                }
            }
        }

        private static void HandleUnexpectedCommand(Fmi2Command command) {
            Console.Error.WriteLine("unexpected command {0}, exiting with status code -1", command.CommandCase);
            Environment.Exit(-1);
        }
    }
}