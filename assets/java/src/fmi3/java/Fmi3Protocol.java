package org.intocps.unifmu.protocol.fmi3;

import com.google.protobuf.ByteString;
import com.google.protobuf.Message;
import com.google.protobuf.InvalidProtocolBufferException;

import org.intocps.unifmu.protocol.Fmi3Messages;
import org.intocps.unifmu.protocol.UnifmuHandshake;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.model.fmi3.Fmi3Model;
import org.intocps.unifmu.model.fmi3.Fmi3ModelFactory;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.ByteBuffer;

import java.util.ArrayList;
import java.util.List;

public class Fmi3Protocol {
    private final CommunicationChannel channel;
    private final Fmi3ModelFactory modelFactory;
    private Fmi3Model model;

    public Fmi3Protocol(CommunicationChannel channel, Fmi3ModelFactory modelFactory) {
        this.channel = channel;
        this.modelFactory = modelFactory;
    }

    Fmi3Messages.Fmi3Command recvCommand() throws IOException, InvalidProtocolBufferException {
        return Fmi3Messages.Fmi3Command.parseFrom(channel.receive());
    }

    void sendReply(Message reply) throws IOException {
        channel.send(reply.toByteArray());
    }

    static Fmi3Messages.Fmi3Status toProtobufStatus(Fmi3Model.Fmi3Status status) {
        return Fmi3Messages.Fmi3Status
            .forNumber(status.ordinal());
    }

    void sendStatusReply(Fmi3Model.Fmi3Status status) throws IOException {
        sendReply(
            Fmi3Messages.Fmi3Return
                .newBuilder()
                .setStatus(
                    Fmi3Messages.Fmi3StatusReturn
                        .newBuilder()
                        .setStatus(toProtobufStatus(status))
                        .build()
                )
                .build()
        );
    }

    private void loggingCallback(Fmi3Model.LogMessage log) {
        try {
            sendReply(
                Fmi3Messages.Fmi3Return.newBuilder()
                    .setLog(Fmi3Messages.Fmi3LogReturn.newBuilder()
                        .setStatus(Fmi3Messages.Fmi3Status.forNumber(log.status().ordinal()))
                        .setCategory(log.category())
                        .setLogMessage(log.message())
                        .build())
                    .build()
            );

            Fmi3Messages.Fmi3Command command = recvCommand();
            switch (command.getCommandCase()) {
                case FMI3CALLBACKCONTINUE:
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

    static Iterable<ByteBuffer> convertToByteBufferArrayList(List<ByteString> byteStrings) {
        List<ByteBuffer> byteBufferArrayList = new ArrayList<>();
        for (ByteString bs : byteStrings) {
            byteBufferArrayList.add(bs.asReadOnlyByteBuffer());
        }
        return byteBufferArrayList;
    }

    static List<ByteString> convertToByteStringList(List<ByteBuffer> byteBuffers) {
        List<ByteString> byteStringList = new ArrayList<>();
        for (ByteBuffer buffer : byteBuffers) {
            byteStringList.add(ByteString.copyFrom(buffer.duplicate()));
        }
        return byteStringList;
    }

    public void run() throws Exception {
        handshake();
        commandReplyLoop();
    }

    void commandReplyLoop() throws Exception {
        while (true) {
            Fmi3Messages.Fmi3Command command = recvCommand();

            switch (command.getCommandCase()) {        
                    
                case FMI3INSTANTIATECOSIMULATION: {
                    var c = command.getFmi3InstantiateCoSimulation();
                    model = modelFactory.create(c, this::loggingCallback);
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setEmpty(
                                Fmi3Messages.Fmi3EmptyReturn
                                    .newBuilder()
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3SETFLOAT32: {
                    var c = command.getFmi3SetFloat32();
                    sendStatusReply(
                        model.fmi3SetFloat32(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }
                    
                case FMI3SETFLOAT64: {
                    var c = command.getFmi3SetFloat64();
                    sendStatusReply(
                        model.fmi3SetFloat64(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETINT8: {
                    var c = command.getFmi3SetInt8();
                    sendStatusReply(
                        model.fmi3SetInt8(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETUINT8: {
                    var c = command.getFmi3SetUInt8();
                    sendStatusReply(
                        model.fmi3SetUInt8(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETINT16: {
                    var c = command.getFmi3SetInt16();
                    sendStatusReply(
                        model.fmi3SetInt16(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETUINT16: {
                    var c = command.getFmi3SetUInt16();
                    sendStatusReply(
                        model.fmi3SetUInt16(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETINT32: {
                    var c = command.getFmi3SetInt32();
                    sendStatusReply(
                        model.fmi3SetInt32(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETUINT32: {
                    var c = command.getFmi3SetUInt32();
                    sendStatusReply(
                        model.fmi3SetUInt32(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETINT64: {
                    var c = command.getFmi3SetInt64();
                    sendStatusReply(
                        model.fmi3SetInt64(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETUINT64: {
                    var c = command.getFmi3SetUInt64();
                    sendStatusReply(
                        model.fmi3SetUInt64(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETBOOLEAN: {
                    var c = command.getFmi3SetBoolean();
                    sendStatusReply(
                        model.fmi3SetBoolean(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETSTRING: {
                    var c = command.getFmi3SetString();
                    sendStatusReply(
                        model.fmi3SetString(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETBINARY: {
                    var c = command.getFmi3SetBinary();
                    sendStatusReply(
                        model.fmi3SetBinary(
                            c.getValueReferencesList(),
                            c.getValueSizesList(),
                            convertToByteBufferArrayList(c.getValuesList())
                        )
                    );
                    break;
                }

                case FMI3SETCLOCK: {
                    var c = command.getFmi3SetClock();
                    sendStatusReply(
                        model.fmi3SetClock(
                            c.getValueReferencesList(),
                            c.getValuesList()
                        )
                    );
                    break;
                }

                case FMI3SETINTERVALDECIMAL: {
                    var c = command.getFmi3SetIntervalDecimal();
                    sendStatusReply(
                        model.fmi3SetIntervalDecimal(
                            c.getValueReferencesList(),
                            c.getIntervalsList()
                        )
                    );
                    break;
                }

                case FMI3SETINTERVALFRACTION: {
                    var c = command.getFmi3SetIntervalFraction();
                    sendStatusReply(
                        model.fmi3SetIntervalFraction(
                            c.getValueReferencesList(),
                            c.getCountersList(),
                            c.getResolutionsList()
                        )
                    );
                    break;
                }

                case FMI3SETSHIFTDECIMAL: {
                    var c = command.getFmi3SetShiftDecimal();
                    sendStatusReply(
                        model.fmi3SetShiftDecimal(
                            c.getValueReferencesList(),
                            c.getShiftsList()
                        )
                    );
                    break;
                }

                case FMI3SETSHIFTFRACTION: {
                    var c = command.getFmi3SetShiftFraction();
                    sendStatusReply(
                        model.fmi3SetShiftFraction(
                            c.getValueReferencesList(),
                            c.getCountersList(),
                            c.getResolutionsList()
                        )
                    );
                    break;
                }

                case FMI3GETFLOAT32: {
                    var c = command.getFmi3GetFloat32();
                    var res = model.fmi3GetFloat32(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetFloat32(
                                Fmi3Messages.Fmi3GetFloat32Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }
                    
                case FMI3GETFLOAT64: {
                    var c = command.getFmi3GetFloat64();
                    var res = model.fmi3GetFloat64(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetFloat64(
                                Fmi3Messages.Fmi3GetFloat64Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINT8: {
                    var c = command.getFmi3GetInt8();
                    var res = model.fmi3GetInt8(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetInt8(
                                Fmi3Messages.Fmi3GetInt8Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETUINT8: {
                    var c = command.getFmi3GetUInt8();
                    var res = model.fmi3GetUInt8(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetUInt8(
                                Fmi3Messages.Fmi3GetUInt8Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINT16: {
                    var c = command.getFmi3GetInt16();
                    var res = model.fmi3GetInt16(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetInt16(
                                Fmi3Messages.Fmi3GetInt16Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETUINT16: {
                    var c = command.getFmi3GetUInt16();
                    var res = model.fmi3GetUInt16(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetUInt16(
                                Fmi3Messages.Fmi3GetUInt16Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINT32: {
                    var c = command.getFmi3GetInt32();
                    var res = model.fmi3GetInt32(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetInt32(
                                Fmi3Messages.Fmi3GetInt32Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETUINT32: {
                    var c = command.getFmi3GetUInt32();
                    var res = model.fmi3GetUInt32(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetUInt32(
                                Fmi3Messages.Fmi3GetUInt32Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINT64: {
                    var c = command.getFmi3GetInt64();
                    var res = model.fmi3GetInt64(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetInt64(
                                Fmi3Messages.Fmi3GetInt64Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETUINT64: {
                    var c = command.getFmi3GetUInt64();
                    var res = model.fmi3GetUInt64(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetUInt64(
                                Fmi3Messages.Fmi3GetUInt64Return
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETBOOLEAN: {
                    var c = command.getFmi3GetBoolean();
                    var res = model.fmi3GetBoolean(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetBoolean(
                                Fmi3Messages.Fmi3GetBooleanReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETSTRING: {
                    var c = command.getFmi3GetString();
                    var res = model.fmi3GetString(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetString(
                                Fmi3Messages.Fmi3GetStringReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETBINARY: {
                    var c = command.getFmi3GetBinary();
                    var res = model.fmi3GetBinary(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetBinary(
                                Fmi3Messages.Fmi3GetBinaryReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(
                                        convertToByteStringList(res.values)
                                    )
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETCLOCK: {
                    var c = command.getFmi3GetClock();
                    var res = model.fmi3GetClock(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetClock(
                                Fmi3Messages.Fmi3GetClockReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllValues(res.values)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINTERVALDECIMAL: {
                    var c = command.getFmi3GetIntervalDecimal();
                    var res = model.fmi3GetIntervalDecimal(
                        c.getValueReferencesList()
                    );
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetIntervalDecimal(
                                Fmi3Messages.Fmi3GetIntervalDecimalReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllIntervals(res.intervals)
                                    .addAllQualifiers(res.qualifiers)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETINTERVALFRACTION: {
                    var c = command.getFmi3GetIntervalFraction();
                    var res = model.fmi3GetIntervalFraction(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetIntervalFraction(
                                Fmi3Messages.Fmi3GetIntervalFractionReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllCounters(res.counters)
                                    .addAllResolutions(res.resolutions)
                                    .addAllQualifiers(res.qualifiers)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETSHIFTDECIMAL: {
                    var c = command.getFmi3GetShiftDecimal();
                    var res = model.fmi3GetShiftDecimal(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetShiftDecimal(
                                Fmi3Messages.Fmi3GetShiftDecimalReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllShifts(res.shifts)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3GETSHIFTFRACTION: {
                    var c = command.getFmi3GetShiftFraction();
                    var res = model.fmi3GetShiftFraction(c.getValueReferencesList());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setGetShiftFraction(
                                Fmi3Messages.Fmi3GetShiftFractionReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .addAllCounters(res.counters)
                                    .addAllResolutions(res.resolutions)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }
    

                case FMI3DOSTEP: {
                    var c = command.getFmi3DoStep();
                    var res = model.fmi3DoStep(c.getCurrentCommunicationPoint(), c.getCommunicationStepSize(), c.getNoSetFmuStatePriorToCurrentPoint());
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setDoStep(
                                Fmi3Messages.Fmi3DoStepReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .setEventHandlingNeeded(res.event_handling_needed)
                                    .setTerminateSimulation(res.terminate_simulation)
                                    .setEarlyReturn(res.early_return)
                                    .setLastSuccessfulTime(res.last_successful_time)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3UPDATEDISCRETESTATES: {
                    var c = command.getFmi3UpdateDiscreteStates();
                    var res = model.fmi3UpdateDiscreteStates();
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setUpdateDiscreteStates(
                                Fmi3Messages.Fmi3UpdateDiscreteStatesReturn
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .setDiscreteStatesNeedUpdate(res.discrete_states_need_update)
                                    .setTerminateSimulation(res.terminate_simulation)
                                    .setNominalsContinuousStatesChanged(res.nominals_continuous_states_changed)
                                    .setValuesContinuousStatesChanged(res.values_continuous_states_changed)
                                    .setNextEventTimeDefined(res.next_event_time_defined)
                                    .setNextEventTime(res.next_event_time)
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3ENTERINITIALIZATIONMODE:
                    sendStatusReply(model.fmi3EnterInitializationMode());
                    break;

                case FMI3EXITINITIALIZATIONMODE:
                    sendStatusReply(model.fmi3ExitInitializationMode());
                    break;

                case FMI3ENTERCONFIGURATIONMODE:
                    sendStatusReply(model.fmi3EnterConfigurationMode());
                    break;

                case FMI3EXITCONFIGURATIONMODE:
                    sendStatusReply(model.fmi3ExitConfigurationMode());
                    break;

                case FMI3ENTEREVENTMODE:
                    sendStatusReply(model.fmi3EnterEventMode());
                    break;

                case FMI3ENTERSTEPMODE:
                    sendStatusReply(model.fmi3EnterStepMode());
                    break;

                case FMI3FREEINSTANCE:
                    return;

                case FMI3RESET:
                    sendStatusReply(model.fmi3Reset());
                    break;

                case FMI3TERMINATE:
                    sendStatusReply(model.fmi3Terminate());
                    break;


                case FMI3SERIALIZEFMUSTATE: {
                    var res = model.fmi3SerializeFmuState();
                    sendReply(
                        Fmi3Messages.Fmi3Return
                            .newBuilder()
                            .setSerializeFmuState(
                                Fmi3Messages.Fmi3SerializeFmuStateReturn    
                                    .newBuilder()
                                    .setStatus(toProtobufStatus(res.status))
                                    .setState(ByteString.copyFrom(res.bytes))
                                    .build()
                            )
                            .build()
                    );
                    break;
                }

                case FMI3DESERIALIZEFMUSTATE: {
                    var c = command.getFmi3DeserializeFmuState();
                    sendStatusReply(
                        model.fmi3DeserializeFmuState(
                            c.getState().asReadOnlyByteBuffer()
                        )
                    );
                    break;
                }

                case FMI3SETDEBUGLOGGING:
                    var c = command.getFmi3SetDebugLogging();
                    sendStatusReply(
                        model.fmi3SetDebugLogging(
                            c.getCategoriesList(),
                            c.getLoggingOn()
                        )
                    );
                    break;

                case COMMAND_NOT_SET:
                    break;

                default:
                    break;
            }
        }
    }
}