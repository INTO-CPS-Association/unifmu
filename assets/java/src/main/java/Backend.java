import java.util.function.Consumer;
import java.util.function.Supplier;

import com.google.protobuf.ByteString;
import com.google.protobuf.Message;
import com.google.protobuf.InvalidProtocolBufferException;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

interface ThrowableSupplier<R, E extends Throwable> {
    R get() throws E;
}

public class Backend {

    static ThrowableSupplier<Fmi2Messages.Fmi2Command, InvalidProtocolBufferException> commandReceiver(ZMQ.Socket socket) {
        return () -> Fmi2Messages.Fmi2Command.parseFrom(socket.recv());
    }

    static Consumer<Message> replySender(ZMQ.Socket socket) {
        return reply -> socket.send(reply.toByteArray(), 0);
    }

    static Consumer<Model.Fmi2Status> statusReplySender(Consumer<Message> sendReply) {
        return status -> {
            sendReply.accept(
                Fmi2Messages.Fmi2Return
                    .newBuilder()
                    .setStatus(
                        Fmi2Messages.Fmi2StatusReturn
                            .newBuilder()
                            .setStatus(Fmi2Messages.Fmi2Status.forNumber(status.ordinal()))
                            .build()
                    )
                    .build()
            );
        };
    }

    public static void main(String[] args) throws Exception {
        System.out.println("starting FMU");

        Model model = null;

        String dispacher_endpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");

        try (ZContext context = new ZContext()) {
            ZMQ.Socket socket = context.createSocket(SocketType.REQ);
            socket.connect(dispacher_endpoint);

            Consumer<Message> sendReply = replySender(socket);

            sendReply.accept(
                UnifmuHandshake.HandshakeReply
                    .newBuilder()
                    .setStatus(UnifmuHandshake.HandshakeStatus.OK)
                    .build()
            );

            ThrowableSupplier<Fmi2Messages.Fmi2Command, InvalidProtocolBufferException> recvCommand = commandReceiver(socket);

            Consumer<Model.Fmi2Status> sendStatusReply = statusReplySender(sendReply);

            while (true) {
                Fmi2Messages.Fmi2Command command = recvCommand.get();

                switch (command.getCommandCase()) {        
                    
                    case FMI2INSTANTIATE:
                        model = new Model();
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setEmpty(
                                    Fmi2Messages.Fmi2EmptyReturn
                                        .newBuilder()
                                        .build()
                                )
                                .build()
                        );
                        break;

                    case FMI2SETREAL: {
                        var c = command.getFmi2SetReal();
                        sendStatusReply.accept(
                            model.fmi2SetReal(
                                c.getReferencesList(),
                                c.getValuesList()
                            )
                        );
                        break;
                    }

                    case FMI2SETINTEGER: {
                        var c = command.getFmi2SetInteger();
                        sendStatusReply.accept(
                            model.fmi2SetInteger(
                                c.getReferencesList(),
                                c.getValuesList()
                            )
                        );
                        break;
                    }

                    case FMI2SETBOOLEAN: {
                        var c = command.getFmi2SetBoolean();
                        sendStatusReply.accept(
                            model.fmi2SetBoolean(
                                c.getReferencesList(),
                                c.getValuesList()
                            )
                        );
                        break;
                    }

                    case FMI2SETSTRING: {
                        var c = command.getFmi2SetString();
                        sendStatusReply.accept(
                            model.fmi2SetString(
                                c.getReferencesList(),
                                c.getValuesList()
                            )
                        );
                        break;
                    }

                    case FMI2GETREAL: {
                        var res = model.fmi2GetReal(
                            command.getFmi2GetReal()
                                .getReferencesList()
                        );
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setGetReal(
                                    Fmi2Messages.Fmi2GetRealReturn
                                        .newBuilder()
                                        .setStatus(
                                            Fmi2Messages.Fmi2Status
                                                .forNumber(res.status.ordinal())
                                        )
                                        .addAllValues(res.values)
                                        .build()
                                )
                                .build()
                        );
                        break;
                    }

                    case FMI2GETINTEGER: {
                        var res = model.fmi2GetInteger(
                            command.getFmi2GetInteger()
                                .getReferencesList()
                        );
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setGetInteger(
                                    Fmi2Messages.Fmi2GetIntegerReturn
                                        .newBuilder()
                                        .setStatus(
                                            Fmi2Messages.Fmi2Status
                                                .forNumber(res.status.ordinal())
                                        )
                                        .addAllValues(res.values)
                                        .build()
                                )
                                .build()
                        );
                        break;
                    }

                    case FMI2GETBOOLEAN: {
                        var res = model.fmi2GetBoolean(
                            command.getFmi2GetBoolean()
                                .getReferencesList()
                        );
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setGetBoolean(
                                    Fmi2Messages.Fmi2GetBooleanReturn
                                        .newBuilder()
                                        .setStatus(
                                            Fmi2Messages.Fmi2Status
                                                .forNumber(res.status.ordinal())
                                        )
                                        .addAllValues(res.values)
                                        .build()
                                )
                                .build()
                        );
                        break;
                    }

                    case FMI2GETSTRING: {
                        var res = model.fmi2GetString(
                            command.getFmi2GetString()
                                .getReferencesList()
                        );
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setGetString(
                                    Fmi2Messages.Fmi2GetStringReturn
                                        .newBuilder()
                                        .setStatus(
                                            Fmi2Messages.Fmi2Status
                                                .forNumber(res.status.ordinal())
                                        )
                                        .addAllValues(res.values)
                                        .build()
                                )
                                .build()
                        );
                        break;
                    }

                    case FMI2DOSTEP: {
                        var c = command.getFmi2DoStep();
                        sendStatusReply.accept(
                            model.fmi2DoStep(
                                c.getCurrentTime(),
                                c.getStepSize(),
                                c.getNoSetFmuStatePriorToCurrentPoint()
                            )
                        );
                        break;
                    }

                    case FMI2SETUPEXPERIMENT: {
                        var c = command.getFmi2SetupExperiment();
                        sendStatusReply.accept(
                            model.fmi2SetupExperiment(
                                c.getStartTime(),
                                c.hasStopTime() ? c.getStopTime() : null,
                                c.hasTolerance() ? c.getTolerance() : null
                            )
                        );
                        break;
                    }

                    case FMI2ENTERINITIALIZATIONMODE:
                        sendStatusReply.accept(model.fmi2EnterInitializationMode());
                        break;

                    case FMI2EXITINITIALIZATIONMODE:
                        sendStatusReply.accept(model.fmi2ExitInitializationMode());
                        break;

                    case FMI2FREEINSTANCE:
                        System.exit(0);

                    case FMI2RESET:
                        sendStatusReply.accept(model.fmi2Reset());
                        break;

                    case FMI2TERMINATE:
                        sendStatusReply.accept(model.fmi2Terminate());
                        break;

                    case FMI2CANCELSTEP:
                        sendStatusReply.accept(model.fmi2CancelStep());
                        break;

                    case FMI2SERIALIZEFMUSTATE: {
                        var res = model.fmi2SerializeFmuState();
                        sendReply.accept(
                            Fmi2Messages.Fmi2Return
                                .newBuilder()
                                .setSerializeFmuState(
                                    Fmi2Messages.Fmi2SerializeFmuStateReturn
                                        .newBuilder()
                                        .setStatus(
                                            Fmi2Messages.Fmi2Status
                                                .forNumber(res.status.ordinal())
                                        )
                                        .setState(ByteString.copyFrom(res.bytes))
                                        .build()
                                )
                                .build()
                        );
                        break;
                    }

                    case FMI2DESERIALIZEFMUSTATE:
                        sendStatusReply.accept(
                            model.fmi2DeserializeFmuState(
                                command.getFmi2DeserializeFmuState()
                                    .getState()
                                    .toByteArray()
                            )
                        );
                        break;

                    case FMI2SETDEBUGLOGGING:
                        break;

                    case COMMAND_NOT_SET:
                        break;

                    default:
                        break;
                }
            }
        }
    }
}
