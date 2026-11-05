package org.intocps.unifmu.protocol.fmi2;

import com.google.protobuf.ByteString;
import com.google.protobuf.Message;
import com.google.protobuf.InvalidProtocolBufferException;

import org.intocps.unifmu.protocol.Fmi2Messages;
import org.intocps.unifmu.protocol.UnifmuHandshake;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.model.fmi2.Fmi2Model;
import org.intocps.unifmu.model.fmi2.Fmi2ModelFactory;
import java.io.IOException;
import java.io.UncheckedIOException;

public class Fmi2Protocol {
    private final CommunicationChannel channel;
    private final Fmi2ModelFactory modelFactory;
    private Fmi2Model model;

    public Fmi2Protocol(CommunicationChannel channel, Fmi2ModelFactory modelFactory) {
        this.channel = channel;
        this.modelFactory = modelFactory;
    }

    Fmi2Messages.Fmi2Command recvCommand() throws IOException, InvalidProtocolBufferException {
        return Fmi2Messages.Fmi2Command.parseFrom(channel.receive());
    }

    void sendReply(Message reply) throws IOException {
        channel.send(reply.toByteArray());
    }

    void sendStatusReply(Fmi2Model.Fmi2Status status) throws IOException {
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

    private void loggingCallback(Fmi2Model.LogMessage log) {
        try {
            sendReply(
                Fmi2Messages.Fmi2Return.newBuilder()
                    .setLog(Fmi2Messages.Fmi2LogReturn.newBuilder()
                        .setStatus(Fmi2Messages.Fmi2Status.forNumber(log.status().ordinal()))
                        .setCategory(log.category())
                        .setLogMessage(log.message())
                        .build())
                    .build()
            );

            Fmi2Messages.Fmi2Command command = recvCommand();
            switch (command.getCommandCase()) {
                case FMI2CALLBACKCONTINUE:
                    break;
                default:
                    throw new IllegalStateException(
                        "Unexpected command received after replying with a logging message."
                    );
            }
        } catch (IOException e) {
            throw new UncheckedIOException(
                "Could not receive the expected callback-continue command.", e
            );
        }
    }

    void handshake() throws IOException {
        sendReply(
            UnifmuHandshake.HandshakeReply
                .newBuilder()
                .setStatus(UnifmuHandshake.HandshakeStatus.OK)
                .build()
        );
    }

    public void run() throws Exception {
        handshake();
        commandReplyLoop();
    }

    void commandReplyLoop() throws Exception {
        while (true) {
            Fmi2Messages.Fmi2Command command = recvCommand();

            switch (command.getCommandCase()) {        
                    
                case FMI2INSTANTIATE:
                    model = modelFactory.create(
                        command.getFmi2Instantiate(), this::loggingCallback
                    );
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
                    return;

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