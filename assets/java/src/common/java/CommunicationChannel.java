package org.intocps.unifmu.communication;

import java.io.IOException;

public interface CommunicationChannel extends AutoCloseable {
    byte[] receive() throws IOException;

    void send(byte[] message) throws IOException;

    @Override
    void close() throws IOException;
}
