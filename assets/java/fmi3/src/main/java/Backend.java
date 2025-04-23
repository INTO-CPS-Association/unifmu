import com.google.protobuf.ByteString;
import com.google.protobuf.Message;

import org.zeromq.SocketType;
import org.zeromq.ZMQ;
import org.zeromq.ZContext;

import java.util.logging.Logger;
import java.util.logging.ConsoleHandler;
import java.util.logging.Level;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;


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
                logger.info("Command: " + command.toString());
                switch (command.getCommandCase()) {        
                    
                    case FMI3INSTANTIATECOSIMULATION: {
                        var c = command.getFmi3InstantiateCoSimulation();
                        model = new Model(c.getInstanceName(),c.getInstantiationToken(),c.getResourcePath(), c.getVisible(), c.getLoggingOn(),c.getEventModeUsed(), c.getEarlyReturnAllowed(),c.getRequiredIntermediateVariablesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(0))
                                .build();                        
                    }
                        break;

                    case FMI3SETFLOAT32: {
                        var c = command.getFmi3SetFloat32();
                        var res = model.fmi3SetFloat32(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;
                    
                    case FMI3SETFLOAT64: {
                        var c = command.getFmi3SetFloat64();
                        var res = model.fmi3SetFloat64(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINT8: {
                        var c = command.getFmi3SetInt8();
                        var res = model.fmi3SetInt8(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETUINT8: {
                        var c = command.getFmi3SetUInt8();
                        var res = model.fmi3SetUInt8(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINT16: {
                        var c = command.getFmi3SetInt16();
                        var res = model.fmi3SetInt16(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETUINT16: {
                        var c = command.getFmi3SetUInt16();
                        var res = model.fmi3SetUInt16(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINT32: {
                        var c = command.getFmi3SetInt32();
                        var res = model.fmi3SetInt32(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETUINT32: {
                        var c = command.getFmi3SetUInt32();
                        var res = model.fmi3SetUInt32(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINT64: {
                        var c = command.getFmi3SetInt64();
                        var res = model.fmi3SetInt64(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETUINT64: {
                        var c = command.getFmi3SetUInt64();
                        var res = model.fmi3SetUInt64(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETBOOLEAN: {
                        var c = command.getFmi3SetBoolean();
                        var res = model.fmi3SetBoolean(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETSTRING: {
                        var c = command.getFmi3SetString();
                        var res = model.fmi3SetString(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETBINARY: {
                        var c = command.getFmi3SetBinary();
                        var res = model.fmi3SetBinary(c.getValueReferencesList(), c.getValueSizesList(), convertToByteArrayList(c.getValuesList()));
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETCLOCK: {
                        var c = command.getFmi3SetClock();
                        var res = model.fmi3SetClock(c.getValueReferencesList(), c.getValuesList());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINTERVALDECIMAL: {
                        var c = command.getFmi3SetIntervalDecimal();
                        var res = model.fmi3SetIntervalDecimal(c.getValueReferencesList(), c.getIntervals());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETINTERVALFRACTION: {
                        var c = command.getFmi3SetIntervalFraction();
                        var res = model.fmi3SetIntervalFraction(c.getValueReferencesList(), c.getCounters(), c.getResolutions());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETSHIFTDECIMAL: {
                        var c = command.getFmi3SetShiftDecimal();
                        var res = model.fmi3SetShiftDecimal(c.getValueReferencesList(), c.getShifts());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3SETSHIFTFRACTION: {
                        var c = command.getFmi3SetShiftFraction();
                        var res = model.fmi3SetShiftFraction(c.getValueReferencesList(), c.getCounters(), c.getResolutions());
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3GETFLOAT32: {
                        var c = command.getFmi3GetFloat32();
                        var res = model.fmi3GetFloat32(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetFloat32Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;
                    
                    case FMI3GETFLOAT64: {
                        var c = command.getFmi3GetFloat64();
                        var res = model.fmi3GetFloat64(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetFloat64Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINT8: {
                        var c = command.getFmi3GetInt8();
                        var res = model.fmi3GetInt8(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetInt8Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETUINT8: {
                        var c = command.getFmi3GetUInt8();
                        var res = model.fmi3GetUInt8(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetUInt8Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINT16: {
                        var c = command.getFmi3GetInt16();
                        var res = model.fmi3GetInt16(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetInt16Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETUINT16: {
                        var c = command.getFmi3GetUInt16();
                        var res = model.fmi3GetUInt16(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetUInt16Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINT32: {
                        var c = command.getFmi3GetInt32();
                        var res = model.fmi3GetInt32(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetInt32Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETUINT32: {
                        var c = command.getFmi3GetUInt32();
                        var res = model.fmi3GetUInt32(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetUInt32Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINT64: {
                        var c = command.getFmi3GetInt64();
                        var res = model.fmi3GetInt64(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetInt64Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETUINT64: {
                        var c = command.getFmi3GetUInt64();
                        var res = model.fmi3GetUInt64(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetUInt64Return.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETBOOLEAN: {
                        var c = command.getFmi3GetBoolean();
                        var res = model.fmi3GetBoolean(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetBooleanReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETSTRING: {
                        var c = command.getFmi3GetString();
                        var res = model.fmi3GetString(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetStringReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETBINARY: {
                        var c = command.getFmi3GetBinary();
                        var res = model.fmi3GetBinary(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetBinaryReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETCLOCK: {
                        var c = command.getFmi3GetClock();
                        var res = model.fmi3GetClock(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetClockReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllValues(res.values)
                                .build();
                    }
                        break;

                    case FMI3GETINTERVALDECIMAL: {
                        var c = command.getFmi3GetIntervalDecimal();
                        var res = model.fmi3GetIntervalDecimal(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetIntervalDecimalReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllIntervals(res.intervals)
                                .addAllQualifiers(res.qualifiers)
                                .build();
                    }
                        break;

                    case FMI3GETINTERVALFRACTION: {
                        var c = command.getFmi3GetIntervalFraction();
                        var res = model.fmi3GetIntervalFraction(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetIntervalFractionReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllCounters(res.counters)
                                .addAllResolutions(res.resolutions)
                                .addAllQualifiers(res.qualifiers)
                                .build();
                    }
                        break;

                    case FMI3GETSHIFTDECIMAL: {
                        var c = command.getFmi3GetShiftDecimal();
                        var res = model.fmi3GetShiftDecimal(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetShiftDecimalReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllShifts(res.shifts)
                                .build();
                    }
                        break;

                    case FMI3GETSHIFTFRACTION: {
                        var c = command.getFmi3GetShiftFraction();
                        var res = model.fmi3GetShiftFraction(c.getValueReferencesList());
                        reply = Fmi3Messages.Fmi3GetShiftFractionReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addAllCounters(res.counters)
                                .addAllResolutions(res.resolutions)
                                .build();
                    }
                        break;
    

                    case FMI3DOSTEP: {
                        var c = command.getFmi3DoStep();
                        var res = model.fmi3DoStep(c.getCurrentCommunicationPoint(), c.getCommunicationStepSize(), c.getNoSetFmuStatePriorToCurrentPoint());
                        reply = Fmi3Messages.Fmi3DoStepReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addBoolean(res.event_handling_needed)
                                .addBoolean(res.terminate_simulation)
                                .addBoolean(res.early_return)
                                .addDouble(res.last_successful_time)
                                .build();
                    }
                        break;

                    case FMI3UPDATEDISCRETESTATES: {
                        var c = command.getFmi3UpdateDiscreteStates();
                        var res = model.fmi3UpdateDiscreteStates();
                        reply = Fmi3Messages.Fmi3UpdateDiscreteStatesReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.status.ordinal()))
                                .addBoolean(res.discrete_states_need_update)
                                .addBoolean(res.terminate_simulation)
                                .addBoolean(res.nominals_continuous_states_changed)
                                .addBoolean(res.values_continuous_states_changed)
                                .addBoolean(res.next_event_time_defined)
                                .addDouble(res.next_event_time)
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
                        var res = model.fmi3ExitInitializationMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3ENTERCONFIGURATIONMODE: {
                        var res = model.fmi3EnterConfigurationMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3EXITCONFIGURATIONMODE: {
                        var res = model.fmi3ExitConfigurationMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3ENTEREVENTMODE: {
                        var res = model.fmi3EnterEventMode();
                        reply = Fmi3Messages.Fmi3StatusReturn.newBuilder()
                                .setStatus(Fmi3Messages.Fmi3Status.forNumber(res.ordinal()))
                                .build();
                    }
                        break;

                    case FMI3ENTERSTEPMODE: {
                        var res = model.fmi3EnterStepMode();
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

    public static Iterable<byte[]> convertToByteArrayList(List<ByteString> byteStrings) {
        List<byte[]> byteArrayList = new ArrayList<>();
        for (ByteString bs : byteStrings) {
            byteArrayList.add(bs.toByteArray());
        }
        return byteArrayList;
    }

}
