using System.Net;
using System.IO;
using System;
using CommandLine;
using Grpc.Core;
using schemas.Fmi2Proto;
using System.Collections.Generic;
using System.Xml.Linq;
using System.Linq;

namespace Launch
{
    class Program
    {
        // Argparse options
        class Options
        {
            [Option('e', "handshake-endpoint", Required = true, HelpText = "The ip and port used to perform the handshake between the wrapper and this slave.")]
            public string HandshakeEndpoint { get; set; }
        }

        public static void Main(string[] args)
        {
            // Set up logging
            StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
            sw.AutoFlush = true;
            Console.SetOut(sw);

            // Obtain handshake arguments
            string HandshakeEndpoint = null;
            Options options = new Options();
            Parser.Default.ParseArguments<Options>(args).WithParsed<Options>(o =>
            {
                HandshakeEndpoint = o.HandshakeEndpoint;
            });
            if (HandshakeEndpoint == null)
                throw new Exception("The handshake endpoint is not defined.");

            // Get value references of attributes in modelDescription file of fmu
            Dictionary<uint, string> referenceToAttr = new Dictionary<uint, string>();
            string curPath = Directory.GetCurrentDirectory();
            string parentPath = Directory.GetParent(curPath).ToString();
            var modelDescriptionPath = Path.Combine(parentPath, "modelDescription.xml");
            XDocument modelDescription = XDocument.Load(modelDescriptionPath);
            var modelVariables = modelDescription.Descendants("ModelVariables");
            foreach (var scalarVariable in modelVariables.Elements("ScalarVariable"))
            {
                uint valueReference = (uint)(scalarVariable.Attribute("valueReference"));
                string name = (string)scalarVariable.Attribute("name");
                string type = (string)scalarVariable.Elements().FirstOrDefault().Name.ToString();
                referenceToAttr.Add(valueReference, name);
            }

            Fmi2FMU slave = new Adder(referenceToAttr);


            string uri = "localhost";
            Server server = new Server
            {
                Services = { SendCommand.BindService(new CommandService.CommandServicer(slave)) },
                Ports = { new ServerPort(uri, 0, ServerCredentials.Insecure) }
            };
            server.Start();

            // Get Connected port
            var enumerator = server.Ports.GetEnumerator();
            enumerator.MoveNext();
            string BoundPort = (enumerator.Current as ServerPort).BoundPort.ToString();
            Console.WriteLine("Started fmu slave on port " + BoundPort);
            Console.WriteLine("Waiting!");

            {
                Channel HandshakeChannel = new Channel(HandshakeEndpoint, ChannelCredentials.Insecure);
                var HandshakeClient = new Handshaker.HandshakerClient(HandshakeChannel);
                sw.WriteLine("Connected handshake client to handshake endpoint {0}", HandshakeEndpoint);
                HandshakeClient.PerformHandshake(new HandshakeInfo { IpAddress = uri, Port = BoundPort });
                HandshakeChannel.ShutdownAsync();
            }

            Console.WriteLine("Sent port number to wrapper!");
            
            server.ShutdownTask.Wait();
        }
    }
}