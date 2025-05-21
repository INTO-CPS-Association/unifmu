import com.google.protobuf.ByteString;
import com.google.protobuf.Message;
import com.google.protobuf.InvalidProtocolBufferException;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

public abstract class AbstractBackend {
    static ZMQ.Socket socket;
    static Model model;

    static Fmi2Messages.Fmi2Command recvCommand() throws InvalidProtocolBufferException {
        return Fmi2Messages.Fmi2Command.parseFrom(socket.recv());
    }

    static void sendReply(Message reply) {
        socket.send(reply.toByteArray(), 0);
    }

    static void sendStatusReply(Model.Fmi2Status status) {
        sendReply(
            Fmi2Messages.Fmi2Return
                .newBuilder()
                .setStatus(
                    Fmi2Messages.Fmi2StatusReturn
                        .newBuilder()
                        .setStatus(
                            Fmi2Messages.Fmi2Status
                                .forNumber(status.ordinal())
                        )
                        .build()
                )
                .build()
        );
    }

    public static void loggingCallback(Model.Fmi2Status status, String category, String message) {
        sendReply(
            Fmi2Messages.Fmi2Return
                .newBuilder()
                .setLog(
                    Fmi2Messages.Fmi2LogReturn
                        .newBuilder()
                        .setStatus(
                            Fmi2Messages.Fmi2Status
                                .forNumber(status.ordinal())
                        )
                        .setCategory(category)
                        .setLogMessage(message)
                        .build()
                )
                .build()
        );

        try {
            Fmi2Messages.Fmi2Command command = recvCommand();

            switch (command.getCommandCase()) {
                case FMI2CALLBACKCONTINUE:
                    break;
                default:
                    System.out.println("Unexpected command received after replying with a logging message.");
                    System.exit(1);
            }
        }
        catch(Exception e) {
            System.out.println("A fatal error occured while parsing expected continue command from UniFMU API layer.");
            System.exit(1);
        }
    }

    static void handshake() {
        sendReply(
            UnifmuHandshake.HandshakeReply
                .newBuilder()
                .setStatus(UnifmuHandshake.HandshakeStatus.OK)
                .build()
        );
    }

    static void connectToEndpoint(ZContext context, String endpoint) {
        socket = context.createSocket(SocketType.REQ);
        socket.connect(endpoint);
    }

    static void commandReplyLoop() throws Exception{
        while (true) {
            Fmi2Messages.Fmi2Command command = recvCommand();

            switch (command.getCommandCase()) {        
                    
                case FMI2INSTANTIATE:
                    model = new Model();
                    sendReply(
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
                    sendStatusReply(
                        model.fmi2SetReal(
                            c.getReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI2SETINTEGER: {
                    var c = command.getFmi2SetInteger();
                    sendStatusReply(
                        model.fmi2SetInteger(
                            c.getReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI2SETBOOLEAN: {
                    var c = command.getFmi2SetBoolean();
                    sendStatusReply(
                        model.fmi2SetBoolean(
                            c.getReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI2SETSTRING: {
                    var c = command.getFmi2SetString();
                    sendStatusReply(
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
                    sendReply(
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
                    sendReply(
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
                    sendReply(
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
                    sendReply(
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
                    sendStatusReply(
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
                    sendStatusReply(
                        model.fmi2SetupExperiment(
                            c.getStartTime(),
                            c.hasStopTime() ? c.getStopTime() : null,
                            c.hasTolerance() ? c.getTolerance() : null
                        )
                    );
                    break;
                }

                case FMI2ENTERINITIALIZATIONMODE:
                    sendStatusReply(model.fmi2EnterInitializationMode());
                    break;

                case FMI2EXITINITIALIZATIONMODE:
                    sendStatusReply(model.fmi2ExitInitializationMode());
                    break;

                case FMI2FREEINSTANCE:
                    System.exit(0);

                case FMI2RESET:
                    sendStatusReply(model.fmi2Reset());
                    break;

                case FMI2TERMINATE:
                    sendStatusReply(model.fmi2Terminate());
                    break;

                case FMI2CANCELSTEP:
                    sendStatusReply(model.fmi2CancelStep());
                    break;

                case FMI2SERIALIZEFMUSTATE: {
                    var res = model.fmi2SerializeFmuState();
                    sendReply(
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
                    sendStatusReply(
                        model.fmi2DeserializeFmuState(
                            command.getFmi2DeserializeFmuState()
                                .getState()
                                .toByteArray()
                        )
                    );
                    break;

                case FMI2SETDEBUGLOGGING: {
                    var c = command.getFmi2SetDebugLogging();
                    sendStatusReply(
                        model.fmi2SetDebugLogging(
                            c.getCategoriesList(),
                            c.getLoggingOn()
                        )
                    );
                    break;
                }

                case COMMAND_NOT_SET:
                    break;

                default:
                    break;
            }
        }
    }
}