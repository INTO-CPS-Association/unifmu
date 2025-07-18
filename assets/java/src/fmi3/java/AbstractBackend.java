import com.google.protobuf.ByteString;
import com.google.protobuf.Message;
import com.google.protobuf.InvalidProtocolBufferException;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public abstract class AbstractBackend {
    static ZMQ.Socket socket;
    static Model model;

    static Fmi3Messages.Fmi3Command recvCommand() throws InvalidProtocolBufferException {
        return Fmi3Messages.Fmi3Command.parseFrom(socket.recv());
    }

    static void sendReply(Message reply) {
        socket.send(reply.toByteArray(), 0);
    }

    static Fmi3Messages.Fmi3Status toProtobufStatus(Model.Fmi3Status status) {
        return Fmi3Messages.Fmi3Status
            .forNumber(status.ordinal());
    }

    static void sendStatusReply(Model.Fmi3Status status) {
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
/*
    public static void loggingCallback(Model.Fmi3Status status, String category, String message) {
        sendReply(
            Fmi3Messages.Fmi3Return
                .newBuilder()
                .setLog(
                    Fmi3Messages.Fmi3LogReturn
                        .newBuilder()
                        .setStatus(
                            Fmi3Messages.Fmi3Status
                                .forNumber(status.ordinal())
                        )
                        .setCategory(category)
                        .setLogMessage(message)
                        .build()
                )
                .build()
        );

        try {
            Fmi3Messages.Fmi3Command command = recvCommand();

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
*/
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

    static Iterable<byte[]> convertToByteArrayList(List<ByteString> byteStrings) {
        List<byte[]> byteArrayList = new ArrayList<>();
        for (ByteString bs : byteStrings) {
            byteArrayList.add(bs.toByteArray());
        }
        return byteArrayList;
    }

    static List<ByteString> convertToByteStringList(List<byte[]> byteArrays) {
        List<ByteString> byteStringList = new ArrayList<>();
        for (byte[] arr : byteArrays) {
            byteStringList.add(ByteString.copyFrom(arr));
        }
        return byteStringList;
    }

    static void commandReplyLoop() throws Exception{
        while (true) {
            Fmi3Messages.Fmi3Command command = recvCommand();

            switch (command.getCommandCase()) {        
                    
                case FMI3INSTANTIATECOSIMULATION: {
                    var c = command.getFmi3InstantiateCoSimulation();
                    model = new Model(
                        c.getInstanceName(),
                        c.getInstantiationToken(),
                        c.getResourcePath(),
                        c.getVisible(),
                        c.getLoggingOn(),
                        c.getEventModeUsed(),
                        c.getEarlyReturnAllowed(),
                        c.getRequiredIntermediateVariablesList()
                    );
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
                            convertToByteArrayList(c.getValuesList())
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
                    System.exit(0);

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
                            c.getState().toByteArray()
                        )
                    );
                    break;
                }

                case FMI3SETDEBUGLOGGING:
                    break;

                case COMMAND_NOT_SET:
                    break;

                default:
                    break;
            }
        }
    }
}