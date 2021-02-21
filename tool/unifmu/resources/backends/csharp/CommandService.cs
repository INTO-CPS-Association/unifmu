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
                    this.fmu.sw.WriteLine("Returned status ERROR");
                    return FmiStatus.Error;
                default:
                    return FmiStatus.Error;
            }
        }

        // Server side handler of the fmi function calls
        public override Task<StatusReturn> Fmi2SetReal(SetReal request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetReal called on slave with values: {0} and value references: {1}", request.Values, request.References);
            FmiStatus status = ConvertStatusType(this.fmu.SetReal(request.References, request.Values));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<GetRealReturn> Fmi2GetReal(GetXXX request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("GetReal called on slave with value references: {0}", request.References);
            (Fmi2Status status, IEnumerable<double> values) = this.fmu.GetReal(request.References);
            var getRealReturn = new GetRealReturn { Status = ConvertStatusType(status)};
            foreach (double v in values) {
                getRealReturn.Values.Add(v);
            }
            return Task.FromResult(getRealReturn);
        }

        public override Task<StatusReturn> Fmi2SetInteger(SetInteger request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetInteger called on slave with values: {0} and value references: {1}", request.Values, request.References);
            FmiStatus status = ConvertStatusType(this.fmu.SetInt(request.References, request.Values));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<GetIntegerReturn> Fmi2GetInteger(GetXXX request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("GetInteger called on slave with value references: {0}", request.References);
            (Fmi2Status status, IEnumerable<int> values) = this.fmu.GetInt(request.References);
            var getIntReturn = new GetIntegerReturn { Status = ConvertStatusType(status)};
            foreach (int v in values) {
                getIntReturn.Values.Add(v);
            }
            return Task.FromResult(getIntReturn);
        }

        public override Task<StatusReturn> Fmi2SetBoolean(SetBoolean request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetBoolean called on slave with values: {0} and value references: {1}", request.Values, request.References);
            FmiStatus status = ConvertStatusType(this.fmu.SetBool(request.References, request.Values));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<GetBooleanReturn> Fmi2GetBoolean(GetXXX request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("GetBool called on slave with value references: {0}", request.References);
            (Fmi2Status status, IEnumerable<bool> values) = this.fmu.GetBool(request.References);
            var getBooleanReturn = new GetBooleanReturn { Status = ConvertStatusType(status)};
            getBooleanReturn.Values.Add(values);
            return Task.FromResult(getBooleanReturn);
        }

        public override Task<StatusReturn> Fmi2SetString(SetString request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetString called on slave with values: {0} and value references: {1}", request.Values, request.References);
            FmiStatus status = ConvertStatusType(this.fmu.SetString(request.References, request.Values));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<GetStringReturn> Fmi2GetString(GetXXX request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("GetString called on slave with value references: {0}", request.References);
            (Fmi2Status status, IEnumerable<string> values) = this.fmu.GetString(request.References);
            var getStringReturn = new GetStringReturn { Status = ConvertStatusType(status)};
            getStringReturn.Values.Add(values);
            return Task.FromResult(getStringReturn);
        }


        public override Task<StatusReturn> Fmi2SetupExperiment(SetupExperiment request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetupExperiment called on slave with start_time: {0}, stop_time: {1}, and tolerance: {2}",
            request.StartTime, request.StopTime, request.Tolerance);

            double? StopTime = request.StopTime;
            double? Tolerance = request.Tolerance;

            if (request.HasStopTime == false)
                StopTime = null;
            if (request.HasTolerance == false)
                Tolerance = null;

            FmiStatus status = ConvertStatusType(this.fmu.SetupExperiment(request.StartTime, StopTime, Tolerance));
            return Task.FromResult(new StatusReturn { Status = status });
        }


        public override Task<StatusReturn> Fmi2EnterInitializationMode(EnterInitializationMode request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("EnterInitializationMode called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.EnterInitializationMode());
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<StatusReturn> Fmi2ExitInitializationMode(ExitInitializationMode request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("ExitInitializationMode called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.ExitInitializationMode());
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<StatusReturn> Fmi2DoStep(DoStep request, ServerCallContext context)
        {
            //this.fmu.sw.WriteLine("DoStep called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.DoStep(request.CurrentTime, request.StepSize, request.NoStepPrior));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<SerializeReturn> Serialize(SerializeMessage request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("Serialize called on slave");
            (byte[] state, Fmi2Status status) = this.fmu.Serialize();
            return Task.FromResult(new SerializeReturn { Status = ConvertStatusType(status), State = Google.Protobuf.ByteString.CopyFrom(state) });
        }

        public override Task<StatusReturn> Deserialize(DeserializeMessage request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("Deserialize called on slave");
            Fmi2Status status = this.fmu.Deserialize(request.State.ToByteArray());
            return Task.FromResult(new StatusReturn { Status = ConvertStatusType(status) });
        }


        public override Task<StatusReturn> Fmi2Terminate(Terminate request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("Terminate called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.Terminate());
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<StatusReturn> Fmi2Reset(Reset request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("Reset called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.Reset());
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<StatusReturn> Fmi2SetDebugLogging(SetDebugLogging request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("SetDebugLogging called on slave");
            FmiStatus status = ConvertStatusType(this.fmu.SetDebugLogging(request.Categories, request.LoggingOn));
            return Task.FromResult(new StatusReturn { Status = status });
        }

        public override Task<StatusReturn> Fmi2FreeInstance(FreeInstance request, ServerCallContext context)
        {
            this.fmu.sw.WriteLine("FreeInstance called on slave");
            GrpcEnvironment.KillServersAsync();
            return Task.FromResult(new StatusReturn { Status = FmiStatus.Ok });
        }
    }
}