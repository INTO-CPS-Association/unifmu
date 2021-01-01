
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
using flatbuffers; // unifmu flatbuffer

namespace Launch
{
    class Program
    {

        public static Fmi2FMU GetSlaveInstance()
        {
            return new Adder(); // Return the relevant slave to use
        }

        // Getter and setter functions
        public static List<double> GetReal(List<int> valueReferences, Dictionary<int, string> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var values = from attr in attributeNames select slave[attr];
            values = values.ToList();
            return (List<double>)values;
        }
        public static List<int> GetInt(List<int> valueReferences, Dictionary<int, string> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var values = from attr in attributeNames select slave[attr];
            values = values.ToList();
            return (List<int>)values;
        }
        public static List<bool> GetBool(List<int> valueReferences, Dictionary<int, string> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var values = from attr in attributeNames select slave[attr];
            values = values.ToList();
            return (List<bool>)values;
        }
        public static List<string> GetString(List<int> valueReferences, Dictionary<int, string> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var values = from attr in attributeNames select slave[attr];
            values = values.ToList();
            return (List<string>)values;
        }
        public static List<object> GetXXX(List<int> valueReferences, Dictionary<int, string> referenceToAttr, Fmi2FMU slave)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var values = from attr in attributeNames select slave[attr];
            return values.ToList();
        }

        public static Fmi2Status SetReal(List<int> valueReferences, List<double> values, Dictionary<int, string> referenceToAttr, Fmi2FMU slave, StreamWriter sw)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var attributesNamesAndValues = attributeNames.Zip(values, (a, v) => new { AttributeName = a, Value = v });
            foreach (var av in attributesNamesAndValues)
            {
                var attrName = av.AttributeName;
                var value = av.Value;
                double testType = 0.0;
                if (value.GetType() == testType.GetType() && slave[attrName].GetType() == testType.GetType())
                {
                    slave[attrName] = value;
                }
                else
                {
                    sw.WriteLine("ERROR: The variable with name: {0}, and value: {1}, is not of type Double/Real, and can therefore not be set in fmu.", attrName, value);
                    return Fmi2Status.Error;
                }
            }
            return Fmi2Status.Ok;
        }

        public static Fmi2Status SetInt(List<int> valueReferences, List<int> values, Dictionary<int, string> referenceToAttr, Fmi2FMU slave, StreamWriter sw)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var attributesNamesAndValues = attributeNames.Zip(values, (a, v) => new { AttributeName = a, Value = v });
            foreach (var av in attributesNamesAndValues)
            {
                var attrName = av.AttributeName;
                var value = av.Value;
                int testType = 0;
                if (value.GetType() == testType.GetType() && slave[attrName].GetType() == testType.GetType())
                {
                    slave[attrName] = value;
                }
                else
                {
                    sw.WriteLine("ERROR: The variable with name: {0}, and value: {1}, is not of type Int, and can therefore not be set in fmu.", attrName, value);
                    return Fmi2Status.Error;
                }
            }
            return Fmi2Status.Ok;
        }

        public static Fmi2Status SetBool(List<int> valueReferences, List<bool> values, Dictionary<int, string> referenceToAttr, Fmi2FMU slave, StreamWriter sw)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var attributesNamesAndValues = attributeNames.Zip(values, (a, v) => new { AttributeName = a, Value = v });
            foreach (var av in attributesNamesAndValues)
            {
                var attrName = av.AttributeName;
                var value = av.Value;
                bool testType = true; // hack
                if (value.GetType() == testType.GetType() && slave[attrName].GetType() == testType.GetType())
                {
                    slave[attrName] = value;
                }
                else
                {
                    sw.WriteLine("ERROR: The variable with name: {0}, and value: {1}, is not of type Bool, and can therefore not be set in fmu.", attrName, value);
                    return Fmi2Status.Error;
                }
            }
            return Fmi2Status.Ok;
        }

        public static Fmi2Status SetString(List<int> valueReferences, List<string> values, Dictionary<int, string> referenceToAttr, Fmi2FMU slave, StreamWriter sw)
        {
            var attributeNames = from vRef in valueReferences select referenceToAttr[vRef];
            var attributesNamesAndValues = attributeNames.Zip(values, (a, v) => new { AttributeName = a, Value = v });
            foreach (var av in attributesNamesAndValues)
            {
                var attrName = av.AttributeName;
                var value = av.Value;
                string testType = "";
                if (value.GetType() == testType.GetType() && slave[attrName].GetType() == testType.GetType())
                {
                    slave[attrName] = value;
                }
                else
                {
                    sw.WriteLine("ERROR: The variable with name: {0}, and value: {1}, is not of type String, and can therefore not be set in fmu.", attrName, value);
                    return Fmi2Status.Error;
                }
            }
            return Fmi2Status.Ok;
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
                Dictionary<int, string> referenceToAttr = new Dictionary<int, string>();

                // Define path
                string parentPath = Directory.GetCurrentDirectory();
                var modelDescriptionPath = Path.Combine(parentPath, "modelDescription.xml");

                // Load modelDescription.xml
                XDocument modelDescription = XDocument.Load(modelDescriptionPath);
                var modelVariables = modelDescription.Descendants("ModelVariables");
                foreach (var scalarVariable in modelVariables.Elements("ScalarVariable"))
                {
                    int valueReference = (int)(scalarVariable.Attribute("valueReference"));
                    string name = (string)scalarVariable.Attribute("name");
                    string type = (string)scalarVariable.Elements().FirstOrDefault().Name.ToString();
                    referenceToAttr.Add(valueReference, name);
                }

                /******DEBUG CODE start ***********/
                List<int> get_values = new List<int> { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 };
                var g_values = GetXXX(get_values, referenceToAttr, slave);
                Console.WriteLine("------Initial values---------");
                foreach (var val in g_values)
                    Console.WriteLine(val);
                SetReal(new List<int> { 0, 1 }, new List<double> { 1.3, 3.8 }, referenceToAttr, slave, sw);
                SetInt(new List<int> { 3, 4 }, new List<int> { 101, 36 }, referenceToAttr, slave, sw);
                SetBool(new List<int> { 6, 7 }, new List<bool> { true, false }, referenceToAttr, slave, sw);
                SetString(new List<int> { 9, 10 }, new List<string> { "hello ", "world!" }, referenceToAttr, slave, sw);
                var new_values = GetXXX(get_values, referenceToAttr, slave);

                Console.WriteLine("--------New values-----");
                foreach (var val in new_values)
                    Console.WriteLine(val);
                /******DEBUG CODE end ***********/

                FlatBufferBuilder fbb = new FlatBufferBuilder(100);

                while (true)
                {
                    sw.WriteLine("Slave waiting for command.");

                    var bytes = commandSocket.ReceiveFrameBytes();
                    var bb = new ByteBuffer(bytes);
                    List<int> valueReferences;

                    FMI2Command command = FMI2Command.GetRootAsFMI2Command(bb);

                    sw.WriteLine("Received command of kind: {0}", command.ArgsType);

                    switch (command.ArgsType)
                    {
                        case Fmi2CommandArg.SetDebugLoggingArgs:
                            SetDebugLoggingArgs a = (SetDebugLoggingArgs)command.Args<SetDebugLoggingArgs>();
                            string[] categories = new string[a.CategoriesLength];
                            for (int i = 0; i < a.CategoriesLength; ++i)
                                categories[i] = a.Categories(i);

                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.SetDebugLogging(categories, a.LoggingOn)).Value);
                            break;
                        case Fmi2CommandArg.SetupExperimentArgs:
                            SetupExperimentArgs b = (SetupExperimentArgs)command.Args<SetupExperimentArgs>();
                            Double? stopTime = b.HasStopTime ? b.StopTime : null;
                            Double? tolerance = b.HasTolerance ? b.Tolerance : null;
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.SetupExperiment(b.StartTime, stopTime, tolerance)).Value);
                            break;
                        case Fmi2CommandArg.FreeInstanceArgs:
                            sw.WriteLine("Freeing instance");
                            Environment.Exit(0);
                            break;
                        case Fmi2CommandArg.EnterInitializationModeArgs:
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.EnterInitializationMode()).Value);
                            break;
                        case Fmi2CommandArg.ExitInitializationModeArgs:
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.ExitInitializationMode()).Value);
                            break;
                        case Fmi2CommandArg.TerminateArgs:
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.Terminate()).Value);
                            break;
                        case Fmi2CommandArg.ResetArgs:
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.Reset()).Value);
                            break;
                        case Fmi2CommandArg.SetRealArgs:
                            SetRealArgs c = (SetRealArgs)command.Args<SetRealArgs>();
                            valueReferences = new List<int>();
                            List<double> realValues = new List<double>();
                            for (int i = 0; i < c.ReferencesLength; ++i)
                            {
                                valueReferences.Add(c.References(i));
                                realValues.Add(c.Values(i));
                            }
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, SetReal(valueReferences, realValues, referenceToAttr, slave, sw)).Value);
                            break;
                        case Fmi2CommandArg.SetIntegerArgs:
                            SetIntegerArgs d = (SetIntegerArgs)command.Args<SetIntegerArgs>();
                            valueReferences = new List<int>();
                            List<int> intValues = new List<int>();
                            for (int i = 0; i < d.ReferencesLength; ++i)
                            {
                                valueReferences.Add(d.References(i));
                                intValues.Add(d.Values(i));
                            }
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, SetInt(valueReferences, intValues, referenceToAttr, slave, sw)).Value);
                            break;
                        case Fmi2CommandArg.SetBooleanArgs:
                            SetBooleanArgs e = (SetBooleanArgs)command.Args<SetBooleanArgs>();
                            valueReferences = new List<int>();
                            List<bool> boolValues = new List<bool>();
                            for (int i = 0; i < e.ReferencesLength; ++i)
                            {
                                valueReferences.Add(e.References(i));
                                boolValues.Add(e.Values(i));
                            }
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, SetBool(valueReferences, boolValues, referenceToAttr, slave, sw)).Value);
                            break;
                        case Fmi2CommandArg.SetStringArgs:
                            SetStringArgs f = (SetStringArgs)command.Args<SetStringArgs>();
                            valueReferences = new List<int>();
                            List<string> stringValues = new List<string>();
                            for (int i = 0; i < f.ReferencesLength; ++i)
                            {
                                valueReferences.Add(f.References(i));
                                stringValues.Add(f.Values(i));
                            }
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, SetString(valueReferences, stringValues, referenceToAttr, slave, sw)).Value);
                            break;
                        case Fmi2CommandArg.GetRealArgs:
                            GetRealArgs g = (GetRealArgs)command.Args<GetRealArgs>();
                            valueReferences = new List<int>();
                            List<double> gotRealValues = GetReal(valueReferences, referenceToAttr, slave);
                            for (int i = 0; i < g.ReferencesLength; ++i)
                                valueReferences.Add(g.References(i));
                            GetRealReturn.StartGetRealReturn(fbb);
                            GetRealReturn.StartValuesVector(fbb, g.ReferencesLength);
                            for (int i = 0; i < g.ReferencesLength; ++i)
                                fbb.PutDouble(gotRealValues[i]);
                            GetRealReturn.EndGetRealReturn(fbb);
                            fbb.Finish(GetRealReturn.CreateGetRealReturn(fbb).Value);
                            break;
                        case Fmi2CommandArg.GetIntegerArgs:
                            GetIntegerArgs gi = (GetIntegerArgs)command.Args<GetIntegerArgs>();
                            valueReferences = new List<int>();
                            List<int> gotIntValues = GetInt(valueReferences, referenceToAttr, slave);
                            for (int i = 0; i < gi.ReferencesLength; ++i)
                                valueReferences.Add(gi.References(i));
                            GetIntegerReturn.StartGetIntegerReturn(fbb);
                            GetIntegerReturn.StartValuesVector(fbb, gi.ReferencesLength);
                            for (int i = 0; i < gi.ReferencesLength; ++i)
                                fbb.PutInt(gotIntValues[i]);
                            GetIntegerReturn.EndGetIntegerReturn(fbb);
                            fbb.Finish(GetIntegerReturn.CreateGetIntegerReturn(fbb).Value);
                            break;
                        case Fmi2CommandArg.GetBooleanArgs:
                            GetBooleanArgs gb = (GetBooleanArgs)command.Args<GetBooleanArgs>();
                            valueReferences = new List<int>();
                            List<bool> gotBoolValues = GetBool(valueReferences, referenceToAttr, slave);
                            for (int i = 0; i < gb.ReferencesLength; ++i)
                                valueReferences.Add(gb.References(i));
                            GetBooleanReturn.StartGetBooleanReturn(fbb);
                            GetBooleanReturn.StartValuesVector(fbb, gb.ReferencesLength);
                            for (int i = 0; i < gb.ReferencesLength; ++i)
                                fbb.PutBool(gotBoolValues[i]);
                            GetBooleanReturn.EndGetBooleanReturn(fbb);
                            fbb.Finish(GetBooleanReturn.CreateGetBooleanReturn(fbb).Value);
                            break;
                        case Fmi2CommandArg.GetStringArgs:
                            GetStringArgs gd = (GetStringArgs)command.Args<GetStringArgs>();
                            valueReferences = new List<int>();
                            List<string> gotStringValues = GetString(valueReferences, referenceToAttr, slave);
                            for (int i = 0; i < gd.ReferencesLength; ++i)
                                valueReferences.Add(gd.References(i));
                            GetStringReturn.StartGetStringReturn(fbb);
                            GetStringReturn.StartValuesVector(fbb, gd.ReferencesLength);
                            for (int i = 0; i < gd.ReferencesLength; ++i)
                                fbb.Put<char>(gotStringValues[i].ToCharArray());
                            GetStringReturn.EndGetStringReturn(fbb);
                            fbb.Finish(GetStringReturn.CreateGetStringReturn(fbb).Value);
                            break;
                        case Fmi2CommandArg.SerializeArgs:
                            byte[] stateA;
                            Fmi2Status status;
                            (stateA, status) = slave.Serialize();
                            if (status == Fmi2Status.Ok)
                            {
                                SerializeReturn.StartSerializeReturn(fbb);
                                SerializeReturn.StartStateVector(fbb, stateA.Length);
                                for (int i = 0; i < stateA.Length; ++i)
                                    fbb.PutByte(stateA[i]);
                                SerializeReturn.EndSerializeReturn(fbb);
                                fbb.Finish(SerializeReturn.EndSerializeReturn(fbb).Value);
                            }
                            else
                            {
                                throw new Exception("The serialization of the fmu failed.");
                            }
                            break;
                        case Fmi2CommandArg.DeserializeArgs:
                            DeserializeArgs h = (DeserializeArgs)command.Args<DeserializeArgs>();
                            byte[] stateB = new byte[h.StateLength];
                            for (int i = 0; i < h.StateLength; ++i)
                                stateB[i] = (byte)h.State(i);
                            SerializeReturn.EndSerializeReturn(fbb);
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.Deserialize(stateB)).Value);
                            break;
                        case Fmi2CommandArg.GetDirectionalDerivativesArgs:
                            throw new NotImplementedException();
                        //break;
                        case Fmi2CommandArg.SetInputDerivativesArgs:
                            throw new NotImplementedException();
                        //break;
                        case Fmi2CommandArg.GetOutputDerivitivesArgs:
                            throw new NotImplementedException();
                        //break;
                        case Fmi2CommandArg.DoStepArgs:
                            DoStepArgs l = (DoStepArgs)command.Args<DoStepArgs>();
                            fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.DoStep(l.CurrentTime, l.StepSize, l.NoStepPrior)).Value);
                            break;
                        case Fmi2CommandArg.CancelStepArgs:
                            throw new NotImplementedException();
                        //break;
                        case Fmi2CommandArg.GetXXXStatusArgs:
                            GetXXXStatusArgs m = (GetXXXStatusArgs)command.Args<GetXXXStatusArgs>();
                            //fbb.Finish(StatusReturn.CreateStatusReturn(fbb, slave.GetXXXStatus(m.)));
                            throw new NotImplementedException();
                            //break;
                    }
                }

            }
        }
    }


}