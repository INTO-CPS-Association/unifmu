import com.google.protobuf.ByteString;
import com.google.protobuf.Message;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.File;
import java.util.Scanner;

import com.moandjiezana.toml.Toml;

public class Backend {

    public static void main(String[] args) throws Exception {
        String RESET = "\u001B[0m";
        String RED = "\u001B[31m";
        String GREEN = "\u001B[32m";
        String YELLOW = "\u001B[33m";
        String BOLD = "\033[0;1m";
        String BACKGROUNDGREEN = "\u001B[42m";
        String BACKGROUNDYELLOW = "\u001B[43m";
        Model model = new Model();

        boolean inputOk = false;
        String port_str = "";
        int port_int = 0;
        if (args.length > 0 && args.length == 1){
            try {
                port_str = args[0];
                port_int = Integer.parseInt(port_str);
                inputOk = true;
            } catch(NumberFormatException nfe) {
                System.err.println(RED + "Only one argument for the port in integer format is accepted." + RESET);
            }
        }

        while (!inputOk) {
            System.out.println(BACKGROUNDGREEN + "Input the port for remote proxy FMU:" + RESET);
            port_str = new BufferedReader(new InputStreamReader(System.in)).readLine();
            try {
                port_int = Integer.parseInt(port_str);
                inputOk = true;
            } catch(NumberFormatException nfe) {
                System.err.println(RED + "Only integers accepted." + RESET);
            }
        }
        Toml toml = new Toml().read(new File("endpoint.toml"));
        String proxy_ip_address = toml.getString("ip");


        String dispacher_endpoint = "tcp://" + proxy_ip_address + ":" + port_str;
        System.out.println(YELLOW + "Dispatcher endpoint received:" + BOLD + BACKGROUNDYELLOW + dispacher_endpoint + RESET);

        try (ZContext context = new ZContext()) {
            ZMQ.Socket socket = context.createSocket(SocketType.REQ);
            socket.connect(dispacher_endpoint);
            System.out.println(YELLOW + "Socket connected successfully." + RESET);

            socket.send(Fmi2Messages.Fmi2EmptyReturn.newBuilder().build().toByteArray(), 0);

            Message reply;
            // Java compiler does not know that reply is always initialized after switch
            // case, so we assign it a dummy value
            reply = Fmi2Messages.Fmi2StatusReturn.newBuilder().build();

            while (true) {
                byte[] message = socket.recv();

                var command = Fmi2Messages.Fmi2Command.parseFrom(message);

                switch (command.getCommandCase()) {

                    case FMI2SETREAL: {
                        var c = command.getFmi2SetReal();
                        var res = model.fmi2SetReal(c.getReferencesList(), c.getValuesList());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SETINTEGER: {
                        var c = command.getFmi2SetInteger();
                        var res = model.fmi2SetInteger(c.getReferencesList(), c.getValuesList());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SETBOOLEAN: {
                        var c = command.getFmi2SetBoolean();
                        var res = model.fmi2SetBoolean(c.getReferencesList(), c.getValuesList());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SETSTRING: {
                        var c = command.getFmi2SetString();
                        var res = model.fmi2SetString(c.getReferencesList(), c.getValuesList());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2GETREAL: {
                        var c = command.getFmi2GetReal();
                        var res = model.fmi2GetReal(c.getReferencesList());
                        reply = Fmi2Messages.Fmi2GetRealReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI2GETINTEGER: {
                        var c = command.getFmi2GetInteger();
                        var res = model.fmi2GetInteger(c.getReferencesList());
                        reply = Fmi2Messages.Fmi2GetIntegerReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI2GETBOOLEAN: {
                        var c = command.getFmi2GetBoolean();
                        var res = model.fmi2GetBoolean(c.getReferencesList());
                        reply = Fmi2Messages.Fmi2GetBooleanReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI2GETSTRING: {
                        var c = command.getFmi2GetString();
                        var res = model.fmi2GetString(c.getReferencesList());
                        reply = Fmi2Messages.Fmi2GetStringReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI2DOSTEP: {
                        var c = command.getFmi2DoStep();
                        var res = model.fmi2DoStep(c.getCurrentTime(), c.getStepSize(), c.getNoSetFmuStatePriorToCurrentPoint());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SETUPEXPERIMENT: {
                        var c = command.getFmi2SetupExperiment();
                        var res = model.fmi2SetupExperiment(c.getStartTime(), c.hasStopTime() ? c.getStopTime() : null,
                                c.hasTolerance() ? c.getTolerance() : null);
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2ENTERINITIALIZATIONMODE: {
                        var res = model.fmi2EnterInitializationMode();
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2EXITINITIALIZATIONMODE: {
                        var res = model.fmi2EnterInitializationMode();
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2FREEINSTANCE:
                        System.exit(0);

                    case FMI2RESET: {
                        var res = model.fmi2Reset();
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2TERMINATE: {
                        var res = model.fmi2Terminate();
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2CANCELSTEP: {
                        var res = model.fmi2CancelStep();
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SERIALIZEFMUSTATE: {
                        var res = model.fmi2SerializeFmuState();
                        reply = Fmi2Messages.Fmi2SerializeFmuStateReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.status.ordinal()))
                                .setState(ByteString.copyFrom(res.bytes))
                                .build();
                    }
                        break;

                    case FMI2DESERIALIZEFMUSTATE: {
                        var c = command.getFmi2DeserializeFmuState();
                        var res = model.fmi2DeserializeFmuState(c.getState().toByteArray());
                        reply = Fmi2Messages.Fmi2StatusReturn.newBuilder()
                                .setStatus(Fmi2Messages.Fmi2Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI2SETDEBUGLOGGING:
                        break;

                    case COMMAND_NOT_SET:
                        break;

                    default:
                        break;

                }

                socket.send(reply.toByteArray(), 0);

            }
        }

    }
}
