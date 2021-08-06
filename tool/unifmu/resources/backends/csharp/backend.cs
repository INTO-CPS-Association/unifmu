
using System.IO;
using System;
using CommandLine;

using Fmi2Proto;
using System.Collections.Generic;

namespace Launch
{
    class Program
    {
        // Argparse options
        class Options
        {
            [Option('e', "dispatcher-endpoint", Required = true, HelpText = "The ip and port used to perform the handshake between the wrapper and this slave.")]
            public string HandshakeEndpoint { get; set; }
        }

        public static void Main(string[] args)
        {
            // Set up logging
            StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
            sw.AutoFlush = true;
            Console.SetOut(sw);

            var references_to_attr = new Dictionary<uint, string>();

            var model = new Model(references_to_attr);

            string dispatcher_endpoint = "";


            byte[] bytes = new byte[0x20];
            Fmi2Command command = Fmi2Command.Parser.ParseFrom(bytes);
            var result = new Fmi2Return();



            while (true)
            {
                result.ClearResult();
                switch (command.CommandCase)
                {
                    case Fmi2Command.CommandOneofCase.Fmi2SetupExperiment:


                        var start_time = command.Fmi2SetupExperiment.StartTime;
                        double? stopTime = command.Fmi2SetupExperiment.HasStopTime ? command.Fmi2SetupExperiment.StopTime : null;
                        double? tolerance = command.Fmi2SetupExperiment.HasTolerance ? command.Fmi2SetupExperiment.Tolerance : null;
                        result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.SetupExperiment(start_time, stopTime, tolerance);
                        break;


                    case Fmi2Command.CommandOneofCase.Fmi2EnterInitializationMode:
                        result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.EnterInitializationMode();

                    case Fmi2Command.CommandOneofCase.Fmi2ExitInitializationMode:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2DoStep:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetReal:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetInteger:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetBoolean:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2SetString:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetReal:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetInteger:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetBoolean:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2GetString:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2CancelStep:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Reset:
                        break;

                    case Fmi2Command.CommandOneofCase.Fmi2Terminate:
                        break;

                    default:
                        break;
                }

            }

        }
    }
}