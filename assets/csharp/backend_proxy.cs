
using System.IO;
using System;

namespace Launch
{
    class Program
    {


        public static void Main(string[] args)
        {
            string RESET = "\x1b[0m";
            string RED = "\u001B[31m";
            string GREEN = "\u001B[32m";
            string YELLOW = "\u001B[33m";
            string BOLD = "\x1b[0;1m";
            string BACKGROUNDGREEN = "\u001B[42m";
            string BACKGROUNDYELLOW = "\u001B[43m";
            Console.WriteLine("Setting up proxy FMU backend.");

            var dispatcher_endpoint = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT",EnvironmentVariableTarget.Process);
            var dispatcher_endpoint_port = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT_PORT",EnvironmentVariableTarget.Process);

            Console.WriteLine("Proxy dispatcher endpoint: {0}.", dispatcher_endpoint);
            Console.WriteLine("Proxy dispatcher endpoint port: {0}", dispatcher_endpoint_port);
            Console.WriteLine(YELLOW + "Use this port to connect the remote (private) FMU model: " + BOLD + BACKGROUNDGREEN + "'{0}'" + RESET, dispatcher_endpoint_port);

        }
    }
}
