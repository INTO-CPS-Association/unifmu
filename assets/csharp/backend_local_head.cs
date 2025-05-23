using System;

namespace Launch
{
    partial class Program
    {
        public static void Main(string[] args)
        {
            string dispatcher_endpoint = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT");
            if (dispatcher_endpoint == null)
            {
                Console.Error.WriteLine("Environment variable 'UNIFMU_DISPATCHER_ENDPOINT' is not set in the current enviornment.");
                Environment.Exit(-1);
            }

            ConnectToEndpoint(dispatcher_endpoint);
            Handshake();
            CommandReplyLoop();
        }
    }
}