using Grpc.Core;
using schemas.Fmi2Proto;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Linq;

namespace CommandService
{
    class CommandServicer : SendCommand.SendCommandBase
    {
        private Fmi2FMU fmu { get; set; }
        public CommandServicer(Fmi2FMU fmu) : base()
        {
            Console.WriteLine("Created C# GRPC slave");
            this.fmu = fmu;
        }

        private FmiStatus ConvertStatusType(Fmi2Status status)
        {
            switch (status)
            {
                case Fmi2Status.Ok:
                    return FmiStatus.Ok;
                case Fmi2Status.Discard:
                    return FmiStatus.Discard;
                case Fmi2Status.Fatal:
                    return FmiStatus.Fatal;
                case Fmi2Status.Warning:
                    return FmiStatus.Warning;
                case Fmi2Status.Pending:
                    return FmiStatus.Pending;
                case Fmi2Status.Error:
                    this.fmu.sw.WriteLine("Returned status ok");
                    return FmiStatus.Error;
                default:
                    return FmiStatus.Error;
            }
        }

        // Server side handler of the fmi function calls
        public override Task<StatusReturn> Fmi2SetReal(SetReal request, ServerCallContext context)
        {
            Console.WriteLine("SetReal called on slave with values: {0} and value references: {1}", request.Values, request.References);
            FmiStatus status = ConvertStatusType(this.fmu.SetReal(request.References, request.Values));
            return Task.FromResult(new StatusReturn { Status = status });
        }
    }
}