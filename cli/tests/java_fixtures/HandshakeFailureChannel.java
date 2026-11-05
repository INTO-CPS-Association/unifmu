package org.intocps.unifmu.testfixtures;

import java.io.IOException;
import org.intocps.unifmu.communication.CommunicationChannel;

public final class HandshakeFailureChannel implements CommunicationChannel {
    public HandshakeFailureChannel(String endpoint) {}

    @Override
    public byte[] receive() throws IOException {
        throw new IOException("Fixture channel does not provide handshake replies.");
    }

    @Override
    public void send(byte[] message) throws IOException {
        throw new IOException("Fixture channel failed before completing the handshake.");
    }

    @Override
    public void close() {}
}
