package fmu;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.HashMap;

import javax.management.RuntimeErrorException;

import com.google.flatbuffers.FlatBufferBuilder;
import com.google.flatbuffers.FlexBuffers;
import com.google.gson.Gson;

import org.zeromq.SocketType;
import org.zeromq.ZContext;
import org.zeromq.ZMQ;

import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.inf.ArgumentParser;
import net.sourceforge.argparse4j.inf.ArgumentParserException;
import net.sourceforge.argparse4j.inf.Namespace;
import flatbuffers.DeserializeArgs;
import flatbuffers.DoStepArgs;
import flatbuffers.FMI2Command;
import flatbuffers.Fmi2CommandArg;
import flatbuffers.GetBooleanArgs;
import flatbuffers.GetIntegerArgs;
import flatbuffers.GetRealArgs;
import flatbuffers.GetStringArgs;
import flatbuffers.SerializeReturn;
import flatbuffers.SetBooleanArgs;
import flatbuffers.SetDebugLoggingArgs;
import flatbuffers.SetIntegerArgs;
import flatbuffers.SetRealArgs;
import flatbuffers.SetStringArgs;
import flatbuffers.SetupExperimentArgs;
import flatbuffers.StatusReturn;

public class Launch {

    private static FMI2FMU getFMU() {
        return new Adder();
    }

    public static void main(String[] args) throws Exception {

        ArgumentParser parser = ArgumentParsers.newFor("UniFMU Java Backend").build().defaultHelp(true)
                .description("Application layer protocol for implementing FMUs in Java using ZMQ");

        parser.addArgument("--handshake-endpoint").help(
                "Endpoint string created by the wrapper which is used to perform handshake when the slave has started");

        Namespace ns = null;
        try {
            ns = parser.parseArgs(args);
            System.out.println(ns);
        } catch (ArgumentParserException e) {
            parser.handleError(e);
            System.exit(1);
        }

        FMI2FMU fmu = getFMU();

        try (ZContext context = new ZContext()) {

            ZMQ.Socket handshakeSocket = context.createSocket(SocketType.PUSH);
            ZMQ.Socket commandSocket = context.createSocket(SocketType.REP);
            handshakeSocket.connect(ns.getString("handshake_endpoint"));

            commandSocket.bindToRandomPort("tcp://127.0.0.1");

            // send handshake
            HashMap<String, String> handshakeInfo = new HashMap<>();
            handshakeInfo.put("serialization_format", "FlexBuffer");
            handshakeInfo.put("command_endpoint", commandSocket.getLastEndpoint());

            Gson gson = new Gson();
            String handshake_json = gson.toJson(handshakeInfo);
            handshakeSocket.send(handshake_json);

            ByteBuffer bb = ByteBuffer.allocate(100);
            FlatBufferBuilder fbb = new FlatBufferBuilder();

            while (!Thread.currentThread().isInterrupted()) {
                // Block until a message is received

                commandSocket.recvByteBuffer(bb, 0);

                FMI2Command command = FMI2Command.getRootAsFMI2Command(bb);

                if (command.argsType() == Fmi2CommandArg.SetDebugLoggingArgs) {
                    SetDebugLoggingArgs a = (SetDebugLoggingArgs) command.args(new SetDebugLoggingArgs());
                    String[] categories = new String[a.categoriesLength()];
                    for (int i = 0; i < a.categoriesLength(); i++) {
                        categories[i] = a.categories(i);
                    }

                    fbb.finish(StatusReturn.createStatusReturn(fbb,
                            fmu.setDebugLogging(categories, a.loggingOn()).ordinal()));

                } else if (command.argsType() == Fmi2CommandArg.SetupExperimentArgs) {
                    SetupExperimentArgs arg = (SetupExperimentArgs) command.args(new SetupExperimentArgs());
                    Double stopTime = arg.hasStopTime() ? arg.stopTime() : null;
                    Double tolerance = arg.hasTolerance() ? arg.tolerance() : null;

                    fbb.finish(StatusReturn.createStatusReturn(fbb,
                            fmu.setupExperiment(arg.startTime(), stopTime, tolerance).ordinal()));
                } else if (command.argsType() == Fmi2CommandArg.FreeInstanceArgs) {
                    System.exit(0);
                }

                else if (command.argsType() == Fmi2CommandArg.EnterInitializationModeArgs) {
                    fbb.finish(StatusReturn.createStatusReturn(fbb, fmu.enterInitializationMode().ordinal()));
                }

                else if (command.argsType() == Fmi2CommandArg.ExitInitializationModeArgs) {
                    fbb.finish(StatusReturn.createStatusReturn(fbb, fmu.exitInitializationMode().ordinal()));
                } else if (command.argsType() == Fmi2CommandArg.EnterInitializationModeArgs) {
                    fbb.finish(StatusReturn.createStatusReturn(fbb, fmu.terminate().ordinal()));
                } else if (command.argsType() == Fmi2CommandArg.ResetArgs) {
                    fbb.finish(StatusReturn.createStatusReturn(fbb, fmu.reset().ordinal()));
                } else if (command.argsType() == Fmi2CommandArg.SetRealArgs) {
                    SetRealArgs a = (SetRealArgs) command.args(new SetRealArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());
                    ArrayList<Double> values = new ArrayList<Double>(a.valuesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                        values.set(i, a.values(i));
                    }

                    throw new Exception("Not implemented");

                } else if (command.argsType() == Fmi2CommandArg.SetIntegerArgs) {
                    SetIntegerArgs a = (SetIntegerArgs) command.args(new SetIntegerArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());
                    ArrayList<Integer> values = new ArrayList<Integer>(a.valuesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                        values.set(i, a.values(i));
                    }

                    throw new Exception("Not implemented");

                } else if (command.argsType() == Fmi2CommandArg.SetBooleanArgs) {
                    SetBooleanArgs a = (SetBooleanArgs) command.args(new SetBooleanArgs());

                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());
                    ArrayList<Boolean> values = new ArrayList<Boolean>(a.valuesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                        values.set(i, a.values(i));
                    }

                    throw new Exception("Not implemented");

                } else if (command.argsType() == Fmi2CommandArg.SetStringArgs) {
                    SetStringArgs a = (SetStringArgs) command.args(new SetStringArgs());

                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());
                    ArrayList<String> values = new ArrayList<String>(a.valuesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                        values.set(i, a.values(i));
                    }

                    throw new Exception("Not implemented");

                }

                else if (command.argsType() == Fmi2CommandArg.GetRealArgs) {
                    GetRealArgs a = (GetRealArgs) command.args(new GetRealArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                    }

                    throw new Exception("Not implemented");
                }

                else if (command.argsType() == Fmi2CommandArg.GetIntegerArgs) {
                    GetIntegerArgs a = (GetIntegerArgs) command.args(new GetIntegerArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                    }

                    throw new Exception("Not implemented");
                }

                else if (command.argsType() == Fmi2CommandArg.GetBooleanArgs) {
                    GetBooleanArgs a = (GetBooleanArgs) command.args(new GetBooleanArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                    }

                    throw new Exception("Not implemented");
                }

                else if (command.argsType() == Fmi2CommandArg.GetStringArgs) {
                    GetStringArgs a = (GetStringArgs) command.args(new GetStringArgs());
                    ArrayList<Integer> references = new ArrayList<Integer>(a.referencesLength());

                    for (int i = 0; i < a.referencesLength(); i++) {
                        references.set(i, a.references(i));
                    }

                    throw new Exception("Not implemented");
                }

                else if (command.argsType() == Fmi2CommandArg.SerializeArgs) {

                    var state = fmu.serialize();

                    SerializeReturn.startSerializeReturn(fbb);
                    SerializeReturn.startStateVector(fbb, state.length);

                    for (int i = 0; i < state.length; i++) {
                        fbb.putByte(state[i]);
                    }

                    SerializeReturn.endSerializeReturn(fbb);
                    fbb.finish(SerializeReturn.endSerializeReturn(fbb));
                }

                else if (command.argsType() == Fmi2CommandArg.DeserializeArgs) {

                    DeserializeArgs a = (DeserializeArgs) command.args(new DeserializeArgs());
                    byte[] state = new byte[a.stateLength()];

                    for (int i = 0; i < a.stateLength(); i++) {
                        state[i] = a.state(i);
                    }

                    SerializeReturn.endSerializeReturn(fbb);
                    fbb.finish(StatusReturn.createStatusReturn(fbb, fmu.deserialize(state).ordinal()));
                }

                else if (command.argsType() == Fmi2CommandArg.DoStepArgs) {

                    DoStepArgs a = (DoStepArgs) command.args(new DoStepArgs());

                    fbb.finish(StatusReturn.createStatusReturn(fbb,
                            fmu.doStep(a.currentTime(), a.stepSize(), a.noStepPrior()).ordinal()));
                }

                else {
                    System.out.println(
                            "Slave received unrecognized command, this is likely a bug in mismatch in the protocol used by the wrapper and the backend");
                    System.exit(1);
                }

            }
        }

    }
}
