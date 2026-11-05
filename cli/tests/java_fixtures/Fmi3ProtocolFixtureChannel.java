package org.intocps.unifmu.testfixtures;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.protocol.Fmi3Messages;
import org.intocps.unifmu.protocol.UnifmuHandshake;

public final class Fmi3ProtocolFixtureChannel implements CommunicationChannel {
    private final List<byte[]> commands = new ArrayList<>();
    private int commandIndex;
    private int replyIndex;

    public Fmi3ProtocolFixtureChannel(String endpoint) {
        commands.add(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3InstantiateCoSimulation(
                Fmi3Messages.Fmi3InstantiateCoSimulation.newBuilder()
                    .setInstanceName("in-memory")
                    .setInstantiationToken("fixture-token")
                    .setResourcePath("")
                    .build()
            )
            .build()
            .toByteArray());
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3EnterInitializationMode(
                Fmi3Messages.Fmi3EnterInitializationMode.newBuilder().setStartTime(0.0).build()
            )));
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3ExitInitializationMode(Fmi3Messages.Fmi3ExitInitializationMode.getDefaultInstance())));
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3SetFloat64(Fmi3Messages.Fmi3SetFloat64.newBuilder()
                .addValueReferences(3)
                .addValueReferences(4)
                .addValues(2.5)
                .addValues(3.5))));
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3DoStep(Fmi3Messages.Fmi3DoStep.newBuilder()
                .setCurrentCommunicationPoint(0.0)
                .setCommunicationStepSize(1.0))));
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3GetFloat64(Fmi3Messages.Fmi3GetFloat64.newBuilder()
                .addValueReferences(5)
                .setNValues(1))));
        commands.add(command(Fmi3Messages.Fmi3Command.newBuilder()
            .setFmi3FreeInstance(Fmi3Messages.Fmi3FreeInstance.getDefaultInstance())));
    }

    private static byte[] command(Fmi3Messages.Fmi3Command.Builder builder) {
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
                Fmi3Messages.Fmi3Return reply = Fmi3Messages.Fmi3Return.parseFrom(bytes);
                switch (replyIndex) {
                    case 1 -> require(reply.getReturnMessageCase() == Fmi3Messages.Fmi3Return.ReturnMessageCase.EMPTY,
                        "Instantiation reply was not empty.");
                    case 2, 3, 4 -> require(
                        reply.getReturnMessageCase() == Fmi3Messages.Fmi3Return.ReturnMessageCase.STATUS
                            && reply.getStatus().getStatus() == Fmi3Messages.Fmi3Status.FMI3_OK,
                        "FMI status reply was not OK."
                    );
                    case 5 -> require(
                        reply.getReturnMessageCase() == Fmi3Messages.Fmi3Return.ReturnMessageCase.DO_STEP
                            && reply.getDoStep().getStatus() == Fmi3Messages.Fmi3Status.FMI3_OK
                            && reply.getDoStep().getLastSuccessfulTime() == 1.0,
                        "DoStep reply did not preserve the expected values."
                    );
                    case 6 -> require(
                        reply.getReturnMessageCase() == Fmi3Messages.Fmi3Return.ReturnMessageCase.GET_FLOAT_64
                            && reply.getGetFloat64().getStatus() == Fmi3Messages.Fmi3Status.FMI3_OK
                            && reply.getGetFloat64().getValuesCount() == 1
                            && reply.getGetFloat64().getValues(0) == 6.0,
                        "GetFloat64 reply did not contain the model result."
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
        require(commandIndex == 7, "Backend did not process all fixture commands.");
        require(replyIndex == 7, "Backend did not send all expected protobuf replies.");
    }
}
