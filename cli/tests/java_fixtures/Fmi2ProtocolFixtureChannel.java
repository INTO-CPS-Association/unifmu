package org.intocps.unifmu.testfixtures;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.protocol.Fmi2Messages;
import org.intocps.unifmu.protocol.UnifmuHandshake;

public final class Fmi2ProtocolFixtureChannel implements CommunicationChannel {
    private final List<byte[]> commands = new ArrayList<>();
    private int commandIndex;
    private int replyIndex;

    public Fmi2ProtocolFixtureChannel(String endpoint) {
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2Instantiate(Fmi2Messages.Fmi2Instantiate.newBuilder()
                .setInstanceName("in-memory")
                .setFmuType(Fmi2Messages.FMI2Type.FMI2_CO_SIMULATION)
                .setFmuGuid("fixture-guid")
                .build())));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2SetupExperiment(Fmi2Messages.Fmi2SetupExperiment.newBuilder()
                .setStartTime(0.0))));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2EnterInitializationMode(Fmi2Messages.Fmi2EnterInitializationMode.getDefaultInstance())));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2SetReal(Fmi2Messages.Fmi2SetReal.newBuilder()
                .addReferences(0)
                .addReferences(1)
                .addValues(2.0)
                .addValues(3.0))));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2ExitInitializationMode(Fmi2Messages.Fmi2ExitInitializationMode.getDefaultInstance())));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2DoStep(Fmi2Messages.Fmi2DoStep.newBuilder()
                .setCurrentTime(0.0)
                .setStepSize(1.0))));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2GetReal(Fmi2Messages.Fmi2GetReal.newBuilder().addReferences(2))));
        commands.add(command(Fmi2Messages.Fmi2Command.newBuilder()
            .setFmi2FreeInstance(Fmi2Messages.Fmi2FreeInstance.getDefaultInstance())));
    }

    private static byte[] command(Fmi2Messages.Fmi2Command.Builder builder) {
        return builder.build().toByteArray();
    }

    @Override
    public byte[] receive() throws IOException {
        if (commandIndex >= commands.size()) {
            throw new IOException("Fixture channel received more commands than expected.");
        }
        return commands.get(commandIndex++);
    }

    @Override
    public void send(byte[] bytes) throws IOException {
        try {
            if (replyIndex == 0) {
                UnifmuHandshake.HandshakeReply reply =
                    UnifmuHandshake.HandshakeReply.parseFrom(bytes);
                require(reply.getStatus() == UnifmuHandshake.HandshakeStatus.OK,
                    "Handshake reply was not OK.");
            } else {
                Fmi2Messages.Fmi2Return reply = Fmi2Messages.Fmi2Return.parseFrom(bytes);
                switch (replyIndex) {
                    case 1 -> require(
                        reply.getReturnMessageCase() == Fmi2Messages.Fmi2Return.ReturnMessageCase.EMPTY,
                        "Instantiation reply was not empty."
                    );
                    case 2, 3, 4, 5, 6 -> require(
                        reply.getReturnMessageCase() == Fmi2Messages.Fmi2Return.ReturnMessageCase.STATUS
                            && reply.getStatus().getStatus() == Fmi2Messages.Fmi2Status.FMI2_OK,
                        "FMI status reply was not OK."
                    );
                    case 7 -> require(
                        reply.getReturnMessageCase() == Fmi2Messages.Fmi2Return.ReturnMessageCase.GET_REAL
                            && reply.getGetReal().getStatus() == Fmi2Messages.Fmi2Status.FMI2_OK
                            && reply.getGetReal().getValuesCount() == 1
                            && reply.getGetReal().getValues(0) == 5.0,
                        "GetReal reply did not contain the model result."
                    );
                    default -> throw new IOException("Fixture channel received too many replies.");
                }
            }
            replyIndex++;
        } catch (com.google.protobuf.InvalidProtocolBufferException e) {
            throw new IOException("Fixture channel received an invalid protobuf reply.", e);
        }
    }

    private static void require(boolean condition, String message) throws IOException {
        if (!condition) {
            throw new IOException(message);
        }
    }

    @Override
    public void close() throws IOException {
        require(commandIndex == 8, "Backend did not process all fixture commands.");
        require(replyIndex == 8, "Backend did not send all expected protobuf replies.");
    }
}
