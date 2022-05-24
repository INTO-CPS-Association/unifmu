
using System.IO;
using System;
using Fmi2Messages;
using System.Collections.Generic;
using NetMQ.Sockets;
using Google.Protobuf;
using NetMQ;


namespace Launch
{
    class Program
    {


        public static void Main(string[] args)
        {
            var references_to_attr = new Dictionary<uint, string>();
            var model = new Model();

            string dispatcher_endpoint = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT");
            if (dispatcher_endpoint == null)
            {
                Console.Error.WriteLine("Environment variable 'UNIFMU_DISPATCHER_ENDPOINT' is not set in the current enviornment.");
                Environment.Exit(-1);
            }


            var socket = new RequestSocket();
            socket.Connect(dispatcher_endpoint);


            IMessage message = new Fmi2EmptyReturn();


            socket.SendFrame(message.ToByteArray(), false);
            Fmi2Command command;

            while (true)
            {
                command = Fmi2Command.Parser.ParseFrom(socket.ReceiveFrameBytes());

                switch (command.CommandCase)
                {

                    case Fmi2Command.CommandOneofCase.Fmi2Instantiate:
                        {
                            var result = new Fmi2EmptyReturn();
                            message = result;
                        }
                        break;
                    case Fmi2Command.CommandOneofCase.Fmi2SetupExperiment:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2SetupExperiment(
                                 command.Fmi2SetupExperiment.StartTime,
                                 command.Fmi2SetupExperiment.HasStopTime ? command.Fmi2SetupExperiment.StopTime : null,
                                 command.Fmi2SetupExperiment.HasTolerance ? command.Fmi2SetupExperiment.Tolerance : null
                             );
                            message = result;
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2EnterInitializationMode:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2EnterInitializationMode();
                            message = result;
                        }

                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2ExitInitializationMode:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2ExitInitializationMode();
                            message = result;
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DoStep:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2DoStep(command.Fmi2DoStep.CurrentTime, command.Fmi2DoStep.StepSize, command.Fmi2DoStep.NoSetFmuStatePriorToCurrentPoint);
                            message = result;
                        }

                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetReal:
                        {
                            var result = new Fmi2StatusReturn();
                            result = new Fmi2StatusReturn();
                            result.Status = model.FmiSetReal(command.Fmi2SetReal.References, command.Fmi2SetReal.Values);
                            message = result;
                        }
                        break;


                    case Fmi2Command.CommandOneofCase.Fmi2SetInteger:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2SetInteger(
                                command.Fmi2SetInteger.References,
                                command.Fmi2SetInteger.Values);
                            message = result;

                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetBoolean:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2SetBoolean(
                                command.Fmi2SetBoolean.References,
                                command.Fmi2SetBoolean.Values
                            );
                            message = result;


                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetString:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2SetString(command.Fmi2SetString.References, command.Fmi2SetString.Values);
                            message = result;
                        }
                        break;


                    case Fmi2Command.CommandOneofCase.Fmi2GetReal:
                        {
                            var result = new Fmi2GetRealReturn();
                            (var status, var values) = model.Fmi2GetReal(command.Fmi2GetReal.References);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetInteger:
                        {
                            var result = new Fmi2GetIntegerReturn();
                            (var status, var values) = model.Fmi2GetInteger(command.Fmi2GetInteger.References);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetBoolean:
                        {
                            var result = new Fmi2GetBooleanReturn();
                            (var status, var values) = model.Fmi2GetBoolean(command.Fmi2GetBoolean.References);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetString:
                        {
                            var result = new Fmi2GetStringReturn();
                            (var status, var values) = model.Fmi2GetString(command.Fmi2GetString.References);
                            result.Values.AddRange(values);
                            result.Status = status;
                            message = result;

                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2CancelStep:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2CancelStep();
                            message = result;

                        }

                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Reset:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2Reset();
                            message = result;

                        }

                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Terminate:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2Terminate();
                            message = result;
                        }
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SerializeFmuState:
                        {
                            var result = new Fmi2SerializeFmuStateReturn();
                            (var status, var state) = model.Fmi2SerializeFmuState();
                            result.Status = status;
                            result.State = ByteString.CopyFrom(state);
                            message = result;
                        }
                        break;
                    case Fmi2Command.CommandOneofCase.Fmi2DeserializeFmuState:
                        {
                            var result = new Fmi2StatusReturn();
                            result.Status = model.Fmi2DeserializeFmuState(command.Fmi2DeserializeFmuState.State.ToByteArray());
                            message = result;
                        }

                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2FreeInstance:
                        {
                            Console.WriteLine("received fmi2FreeInstance, exiting with status code 0");
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
    }
}