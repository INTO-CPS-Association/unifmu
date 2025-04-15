import com.google.protobuf.ByteString;
import com.google.protobuf.Message;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

import java.util.logging.Logger;
import java.util.logging.ConsoleHandler;
import java.util.logging.Level;


public class Backend {
    private static final Logger logger = Logger.getLogger(Backend.class.getName());

    public static void main(String[] args) throws Exception {
        ConsoleHandler consoleHandler = new ConsoleHandler();
        logger.addHandler(consoleHandler);
        logger.setLevel(Level.ALL);

        Model model = null;

        String dispacher_endpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");

        try (ZContext context = new ZContext()) {
            ZMQ.Socket socket = context.createSocket(SocketType.REQ);
            socket.connect(dispacher_endpoint);

            socket.send(
                UnifmuHandshake.HandshakeReply
                    .newBuilder()
                    .setStatus(UnifmuHandshake.HandshakeStatus.OK)
                    .build()
                    .toByteArray(),
                0
            );

            Message reply;
            // Java compiler does not know that reply is always initialized after switch
            // case, so we assign it a dummy value
            reply = Fmi3Messages.Fmi3StatusReturn.newBuilder().build();

            while (true) {
                byte[] message = socket.recv();

                var command = Fmi3Messages.Fmi3Command.parseFrom(message);
                //logger.info("Command: " + command.toString());
                switch (command.getCommandCase()) {        
                    
                    case FMI3INSTANTIATE: {
                        model = new Model();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(0))
                                .build();                        
                    }
                        break;

                    case FMI3SETREAL: {
                        var c = command.getFmi3SetReal();
                        var res = model.fmi3SetReal(c.getReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINTEGER: {
                        var c = command.getFmi3SetInteger();
                        var res = model.fmi3SetInteger(c.getReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETBOOLEAN: {
                        var c = command.getFmi3SetBoolean();
                        var res = model.fmi3SetBoolean(c.getReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETSTRING: {
                        var c = command.getFmi3SetString();
                        var res = model.fmi3SetString(c.getReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3GETREAL: {
                        var c = command.getFmi3GetReal();
                        var res = model.fmi3GetReal(c.getReferencesList());
                        reply = Fmi3Messages.Fmi3GetRealReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINTEGER: {
                        var c = command.getFmi3GetInteger();
                        var res = model.fmi3GetInteger(c.getReferencesList());
                        reply = Fmi3Messages.Fmi3GetIntegerReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETBOOLEAN: {
                        var c = command.getFmi3GetBoolean();
                        var res = model.fmi3GetBoolean(c.getReferencesList());
                        reply = Fmi3Messages.Fmi3GetBooleanReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETSTRING: {
                        var c = command.getFmi3GetString();
                        var res = model.fmi3GetString(c.getReferencesList());
                        reply = Fmi3Messages.Fmi3GetStringReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3DOSTEP: {
                        var c = command.getFmi3DoStep();
                        var res = model.fmi3DoStep(c.getCurrentTime(), c.getStepSize(), c.getNoSetFmuStatePriorToCurrentPoint());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETUPEXPERIMENT: {
                        var c = command.getFmi3SetupExperiment();
                        var res = model.fmi3SetupExperiment(c.getStartTime(), c.hasStopTime() ? c.getStopTime() : null,
                                c.hasTolerance() ? c.getTolerance() : null);
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3ENTERINITIALIZATIONMODE: {
                        var res = model.fmi3EnterInitializationMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3EXITINITIALIZATIONMODE: {
                        var res = model.fmi3EnterInitializationMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3FREEINSTANCE:
                        System.exit(0);

                    case FMI3RESET: {
                        var res = model.fmi3Reset();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3TERMINATE: {
                        var res = model.fmi3Terminate();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3CANCELSTEP: {
                        var res = model.fmi3CancelStep();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SERIALIZEFMUSTATE: {
                        var res = model.fmi3SerializeFmuState();
                        reply = Fmi3Messages.Fmi3SerializeFmuStateReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .setState(ByteString.copyFrom(res.bytes))
                                .build();
                    }
                        break;

                    case FMI3DESERIALIZEFMUSTATE: {
                        var c = command.getFmi3DeserializeFmuState();
                        var res = model.fmi3DeserializeFmuState(c.getState().toByteArray());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETDEBUGLOGGING:
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
