package org.intocps.unifmu.communication;

import java.io.IOException;
import org.zeromq.SocketType;
import org.zeromq.ZContext;
import org.zeromq.ZMQ;

public final class ZeroMqCommunicationChannel implements CommunicationChannel {
    private final ZContext context;
    private final ZMQ.Socket socket;

    public ZeroMqCommunicationChannel(String endpoint) throws IOException {
        if (endpoint == null || endpoint.isBlank()) {
            throw new IOException("The dispatcher endpoint is missing.");
        }

        context = new ZContext();
        socket = context.createSocket(SocketType.REQ);
        if (!socket.connect(endpoint)) {
            context.close();
            throw new IOException("Could not connect to dispatcher endpoint: " + endpoint);
        }
    }

    @Override
    public byte[] receive() throws IOException {
        byte[] message = socket.recv();
        if (message == null) {
            throw new IOException("ZeroMQ channel closed before a reply was received.");
        }
        return message;
    }

    @Override
    public void send(byte[] message) throws IOException {
        if (!socket.send(message, 0)) {
            throw new IOException("Could not send message on ZeroMQ channel.");
        }
    }

    @Override
    public void close() {
        context.close();
    }
}
