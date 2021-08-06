
using System.IO;
using System;
using Fmi2Proto;
using System.Collections.Generic;
using NetMQ.Sockets;
using Google.Protobuf;
using NetMQ;


namespace Launch
{
  class Program
  {


    public static void Main(string[] args)
    {
      // Set up logging
      StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
      sw.AutoFlush = true;
      Console.SetOut(sw);

      var references_to_attr = new Dictionary<uint, string>();
      var model = new Model(references_to_attr);

      // read environment variables
      string dispatcher_endpoint = System.Environment.GetEnvironmentVariable("UNIFMU_DISPATCHER_ENDPOINT");


      var socket = new RequestSocket();
      socket.Connect(dispatcher_endpoint);

      var result = new Fmi2Return();
      result.Fmi2StatusReturn = new Fmi2StatusReturn();
      result.Fmi2GetRealReturn = new Fmi2GetRealReturn();
      result.Fmi2GetIntegerReturn = new Fmi2GetIntegerReturn();
      result.Fmi2GetBooleanReturn = new Fmi2GetBooleanReturn();
      result.Fmi2GetStringReturn = new Fmi2GetStringReturn();
      result.Fmi2GetStringReturn = new Fmi2GetStringReturn();
      result.Fmi2ExtSerializeSlaveReturn = new Fmi2ExtSerializeSlaveReturn();
      // this has to be last to set the variant!
      result.Fmi2ExtHandshakeReturn = new Fmi2ExtHandshakeReturn();


      socket.SendFrame(result.ToByteArray(), false);
      Fmi2Command command;



      while (true)
      {

        command = Fmi2Command.Parser.ParseFrom(socket.ReceiveFrameBytes());
        Console.WriteLine("received command {0}", command);

        // result.ClearResult();
        switch (command.CommandCase)
        {
          case Fmi2Command.CommandOneofCase.Fmi2SetupExperiment:
            {
              result.Fmi2StatusReturn = new Fmi2StatusReturn();

              var status = model.SetupExperiment(
                   command.Fmi2SetupExperiment.StartTime,
                   command.Fmi2SetupExperiment.HasStopTime ? command.Fmi2SetupExperiment.StopTime : null,
                   command.Fmi2SetupExperiment.HasTolerance ? command.Fmi2SetupExperiment.Tolerance : null
               );

              result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)status;
            }
            break;

          case Fmi2Command.CommandOneofCase.Fmi2EnterInitializationMode:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.EnterInitializationMode();
            break;

          case Fmi2Command.CommandOneofCase.Fmi2ExitInitializationMode:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.ExitInitializationMode();
            break;

          case Fmi2Command.CommandOneofCase.Fmi2DoStep:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Fmi2DoStep(command.Fmi2DoStep.CurrentTime, command.Fmi2DoStep.StepSize, command.Fmi2DoStep.NoStepPrior);
            break;

          case Fmi2Command.CommandOneofCase.Fmi2SetReal:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.FmiSetReal(command.Fmi2SetReal.References, command.Fmi2SetReal.Values);
            break;


          case Fmi2Command.CommandOneofCase.Fmi2SetInteger:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Fmi2SetInteger(
                command.Fmi2SetInteger.References,
                command.Fmi2SetInteger.Values);
            break;

          case Fmi2Command.CommandOneofCase.Fmi2SetBoolean:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Fmi2SetBoolean(
                command.Fmi2SetBoolean.References,
                command.Fmi2SetBoolean.Values
            );
            break;


          case Fmi2Command.CommandOneofCase.Fmi2SetString:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Fmi2SetString(command.Fmi2SetInteger.References, command.Fmi2SetString.Values);
            break;

          case Fmi2Command.CommandOneofCase.Fmi2GetReal:
            {
              result.Fmi2GetRealReturn = new Fmi2GetRealReturn();
              (var status, var values) = model.Fmi2GetReal(command.Fmi2GetReal.References);
              result.Fmi2GetRealReturn.Values.AddRange(values);
              result.Fmi2GetRealReturn.Status = (Fmi2Proto.Fmi2Status)status;
            }
            break;

          case Fmi2Command.CommandOneofCase.Fmi2GetInteger:
            {
              result.Fmi2GetIntegerReturn = new Fmi2GetIntegerReturn();
              (var status, var values) = model.Fmi2GetInteger(command.Fmi2GetInteger.References);
              result.Fmi2GetIntegerReturn.Values.AddRange(values);
              result.Fmi2GetIntegerReturn.Status = (Fmi2Proto.Fmi2Status)status;
            }
            break;

          case Fmi2Command.CommandOneofCase.Fmi2GetBoolean:
            {
              result.Fmi2GetBooleanReturn = new Fmi2GetBooleanReturn();
              (var status, var values) = model.Fmi2GetBoolean(command.Fmi2GetBoolean.References);
              result.Fmi2GetBooleanReturn.Values.AddRange(values);
              result.Fmi2GetBooleanReturn.Status = (Fmi2Proto.Fmi2Status)status;
            }
            break;

          case Fmi2Command.CommandOneofCase.Fmi2GetString:
            {
              result.Fmi2GetStringReturn = new Fmi2GetStringReturn();
              (var status, var values) = model.Fmi2GetString(command.Fmi2GetString.References);
              result.Fmi2GetStringReturn.Values.AddRange(values);
              result.Fmi2GetStringReturn.Status = (Fmi2Proto.Fmi2Status)status;
            }
            break;

          case Fmi2Command.CommandOneofCase.Fmi2CancelStep:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.CancelStep();
            break;

          case Fmi2Command.CommandOneofCase.Fmi2Reset:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Reset();
            break;

          case Fmi2Command.CommandOneofCase.Fmi2Terminate:
            result.Fmi2StatusReturn = new Fmi2StatusReturn();
            result.Fmi2StatusReturn.Status = (Fmi2Proto.Fmi2Status)model.Terminate();
            break;

          case Fmi2Command.CommandOneofCase.Fmi2ExtSerializeSlave:
            {

              result.Fmi2ExtSerializeSlaveReturn = new Fmi2ExtSerializeSlaveReturn();
              (var state, var status) = model.Serialize();
              result.Fmi2ExtSerializeSlaveReturn.Status = (Fmi2Proto.Fmi2Status)status;
              result.Fmi2ExtSerializeSlaveReturn.State = ByteString.CopyFrom(state);

            }
            break;

          default:
            Console.WriteLine("unrecognized command {0}, shutting down", command.CommandCase);
            Environment.Exit(-1);
            break;
        }


        socket.SendFrame(result.ToByteArray(), false);
        Console.WriteLine("returning result {0}", result);

      }

    }
  }
}