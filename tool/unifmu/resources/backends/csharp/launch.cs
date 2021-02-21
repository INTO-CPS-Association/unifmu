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
            public string handshake_endpoint { get; set; }
        }

        public static void Main(string[] args)
        {
            // Set up logging
            StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
            sw.AutoFlush = true;
            Console.SetOut(sw);

            // Obtain handshake arguments
            string handshake_endpoint = null;
            Options options = new Options();
            Parser.Default.ParseArguments<Options>(args).WithParsed<Options>(o =>
            {
                handshake_endpoint = o.handshake_endpoint;
            });
            if (handshake_endpoint == null)
                throw new Exception("The handshake endpoint is not defined.");

            // Get value references of attributes in modelDescription file of fmu
            Dictionary<uint, string> referenceToAttr = new Dictionary<uint, string>();
            string parentPath = Directory.GetCurrentDirectory();
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



            Channel handshake_channel = new Channel(handshake_endpoint, ChannelCredentials.Insecure);
            var handshake_client = new Handshaker.HandshakerClient(handshake_channel);
            Console.WriteLine("Connected handshake client to handshake endpoint: {0}", handshake_endpoint);
        }
    }
}