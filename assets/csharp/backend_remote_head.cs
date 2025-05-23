using System;
using System.IO;
using Tomlyn;

namespace Launch
{
    partial class Program
    {
        public static void Main(string[] args)
        {
            string RESET = "\x1b[0m";
            string RED = "\u001B[31m";
            string YELLOW = "\u001B[33m";
            string BOLD = "\x1b[0;1m";
            string BACKGROUNDGREEN = "\u001B[42m";

            bool inputOk = false;
            var port_str = "";
            int port_int = 0;

            if (args.Length > 1 && args.Length == 2){
                try {
                    port_str = args[1];
                    port_int = Int32.Parse(port_str);
                    inputOk = true;
                } catch(Exception e) {
                    Console.Error.WriteLine(RED + "Only one argument for the port in integer format is accepted." + RESET);
                }
            }

            while (!inputOk) {
                Console.WriteLine(BACKGROUNDGREEN + "Input the port for remote proxy FMU:" + RESET);
                port_str = Console.ReadLine();
                try {
                    port_int = Int32.Parse(port_str);
                    inputOk = true;
                } catch(Exception e2) {
                    Console.Error.WriteLine(RED + "Only integers accepted." + RESET);
                }
            }

            var toml_str = File.ReadAllText(@"endpoint.toml");
            var toml = Toml.ToModel(toml_str);
            string proxy_ip_address = (string)toml["ip"]!;

            string dispatcher_endpoint = "tcp://" + proxy_ip_address + ":" + port_str;
            Console.WriteLine(YELLOW + "Dispatcher endpoint received:" + BOLD + BACKGROUNDGREEN + dispatcher_endpoint + RESET);

            ConnectToEndpoint(dispatcher_endpoint);
            Console.WriteLine(YELLOW + "Socket connected successfully." + RESET);

            Handshake();
            CommandReplyLoop();
        }
    }
}