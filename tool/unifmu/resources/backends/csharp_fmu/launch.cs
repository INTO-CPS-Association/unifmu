using System.Collections.Generic;
using System.Xml.Linq;
using System.Linq;
using System.IO;
using System;
using NetMQ.Sockets;
using NetMQ;
using CommandLine;
using Newtonsoft.Json;
using FlatBuffers;

using flatbuffers;

namespace Launch
{
    class Program
    {

        public static Fmi2FMU GetSlaveInstance()
        {
            return new Adder(); // Return the relevant slave to use
        }

        // Getter and setter functions
        public static List<object> GetXXX(List<int> valueReferences, Dictionary<int, Tuple<string, string>> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef].Item1;
            var values = from attr in attributeNames select slave[attr];
            return values.ToList();
        }

        public static Fmi2Status SetXXX(List<int> valueReferences, List<object> values, Dictionary<int, Tuple<string, string>> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef].Item1;
            var attributesNamesAndValues = attributeNames.Zip(values, (a, v) => new { AttributeName = a, Value = v });
            foreach (var av in attributesNamesAndValues)
            {
                var attrName = av.AttributeName;
                var value = av.Value;
                slave[attrName] = value;
            }
            return Fmi2Status.Ok;
        }

        public static void FreeInstance(StreamWriter sw)
        {
            sw.WriteLine("Freeing instance");
        }

        // Argparse options
        class Options
        {
            [Option('e', "handshake-endpoint", Required = true, HelpText = "socket")]
            public string handshake_endpoint { get; set; }
        }

        public static void Main(string[] args)
        {
            // Set up loggiing
            StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
            sw.AutoFlush = true;
            Console.SetOut(sw);

            // Obtain arguments
            string handshake_endpoint = null;
            Options options = new Options();
            Parser.Default.ParseArguments<Options>(args).WithParsed<Options>(o =>
            {
                handshake_endpoint = o.handshake_endpoint;
            });
            if (handshake_endpoint == null)
                throw new Exception("The handshake endpoint is not defined.");
            Console.WriteLine("handshake_endpoint: " + handshake_endpoint); // TODO: remove debug code

            // Initialize message queue
            using (var handshakeSocket = new PushSocket(handshake_endpoint))
            using (var commandSocket = new ResponseSocket())
            {
                var commandPort = commandSocket.BindRandomPort("tcp://127.0.0.1");

                Dictionary<string, string> handshakeInfo = new Dictionary<string, string>
                {
                    {"serialization_format", "FlatBuffers"},
                    {"command_endpoint", commandSocket.Options.LastEndpoint}
                };

                string handshakeJson = JsonConvert.SerializeObject(handshakeInfo, Newtonsoft.Json.Formatting.Indented);
                handshakeSocket.SendFrame(handshakeJson);


                // Create slave object then use model description to create a mapping between fmi value references and attribute names of FMU
                Fmi2FMU slave = GetSlaveInstance();
                Dictionary<int, Tuple<string, string>> referenceToAttr = new Dictionary<int, Tuple<string, string>>();

                // Define path
                string parentPath = Directory.GetCurrentDirectory();
                var modelDescriptionPath = Path.Combine(parentPath, "modelDescription.xml");
                Console.WriteLine("path: " + modelDescriptionPath); // TODO: remove debug code

                // Load modelDescription.xml
                XDocument modelDescription = XDocument.Load(modelDescriptionPath);
                var modelVariables = modelDescription.Descendants("ModelVariables");
                foreach (var scalarVariable in modelVariables.Elements("ScalarVariable"))
                {
                    int valueReference = (int)(scalarVariable.Attribute("valueReference"));
                    string name = (string)scalarVariable.Attribute("name");
                    string type = (string)scalarVariable.Elements().FirstOrDefault().Name.ToString();
                    referenceToAttr.Add(valueReference, Tuple.Create(name, type));
                }

                foreach (var item in referenceToAttr) // TODO: remove debug code
                {
                    Console.WriteLine(item.Key + ": " + item.Value);
                }

                // Getter and setter functions
                List<int> get_values = new List<int> { 0, 1, 2, 3, 4, 5, 6, 7, 8 };// TODO: remove debug code
                List<object> set_values = new List<object> { 1.2f, 3.8f, 32, 100, true, true };// TODO: remove debug code
                List<int> set_values_vref = new List<int> { 0, 1, 3, 4, 6, 7 }; // TODO: remove debug code
                var g_values = GetXXX(get_values, referenceToAttr, slave);

                foreach (var val in g_values)// TODO: remove debug code
                    Console.WriteLine(val);// TODO: remove debug code

                SetXXX(set_values_vref, set_values, referenceToAttr, slave);

                var new_values = GetXXX(get_values, referenceToAttr, slave);// TODO: remove debug code
                foreach (var val in new_values)// TODO: remove debug code
                    Console.WriteLine(val);// TODO: remove debug code

                FlatBufferBuilder fbb = new FlatBufferBuilder(100);

                while (true)
                {
                    sw.WriteLine("Slave waiting for command.");

                    var bytes = commandSocket.ReceiveFrameBytes();
                    var bb = new ByteBuffer(bytes);

                    FMI2Command command = FMI2Command.GetRootAsFMI2Command(bb);

                    sw.WriteLine("Received command of kind: {0}", command.ArgsType);

                    Fmi2Status status;
                    List<int> valueReferences;

                    switch (command.ArgsType)
                    {
                        case Fmi2CommandArg.SetDebugLoggingArgs:
                            SetDebugLoggingArgs a = (SetDebugLoggingArgs)command.Args<SetDebugLoggingArgs>();
                            string[] categories = new string[a.CategoriesLength];
                            foreach (int i in Enumerable.Range(0, a.CategoriesLength))
                                categories[i] = a.Categories(i);

                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.SetDebugLogging(categories, a.LoggingOn)).Value);
                            break;
                        case Fmi2CommandArg.SetupExperimentArgs:
                            double startTime = 0;
                            status = slave.SetupExperiment(startTime);
                            break;
                        case Fmi2CommandArg.FreeInstanceArgs:
                            FreeInstance(sw);
                            break;
                        case Fmi2CommandArg.EnterInitializationModeArgs:
                            status = slave.EnterInitializationMode();
                            break;
                        case Fmi2CommandArg.ExitInitializationModeArgs:
                            status = slave.ExitInitializationMode();
                            break;
                        case Fmi2CommandArg.TerminateArgs:
                            status = slave.Terminate();
                            break;
                        case Fmi2CommandArg.ResetArgs:
                            status = slave.Reset();
                            break;
                        case Fmi2CommandArg.SetRealArgs:
                            valueReferences = new List<int>();
                            List<object> values = new List<object>(); // recvArgs
                            status = SetXXX(valueReferences, values, referenceToAttr, slave);
                            break;
                        case Fmi2CommandArg.SetIntegerArgs:
                            break;
                        case Fmi2CommandArg.SetBooleanArgs:
                            break;
                        case Fmi2CommandArg.SetStringArgs:
                            break;
                        case Fmi2CommandArg.GetRealArgs:
                            valueReferences = new List<int>();
                            var slaveAttrValues = GetXXX(valueReferences, referenceToAttr, slave);
                            break;
                        case Fmi2CommandArg.GetIntegerArgs:
                            break;
                        case Fmi2CommandArg.GetBooleanArgs:
                            break;
                        case Fmi2CommandArg.GetStringArgs:
                            break;
                        case Fmi2CommandArg.SerializeArgs:
                            string serializedSlave;
                            (serializedSlave, status) = slave.Serialize();
                            break;
                        case Fmi2CommandArg.DeserializeArgs:
                            break;
                        case Fmi2CommandArg.GetDirectionalDerivativesArgs:
                            break;
                        case Fmi2CommandArg.SetInputDerivativesArgs:
                            break;
                        case Fmi2CommandArg.GetOutputDerivitivesArgs:
                            break;
                        case Fmi2CommandArg.DoStepArgs:
                            break;
                        case Fmi2CommandArg.CancelStepArgs:
                            break;
                        case Fmi2CommandArg.GetXXXStatusArgs:
                            break;
                    }
                }

            }
        }
    }


}