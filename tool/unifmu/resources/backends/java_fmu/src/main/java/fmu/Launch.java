package fmu;

import java.nio.ByteBuffer;
import java.util.HashMap;

import com.google.flatbuffers.FlexBuffers;
import com.google.gson.Gson;

import org.zeromq.SocketType;
import org.zeromq.ZContext;
import org.zeromq.ZMQ;

import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.inf.ArgumentParser;
import net.sourceforge.argparse4j.inf.ArgumentParserException;
import net.sourceforge.argparse4j.inf.Namespace;

enum CommandIds {
    SetDebugLogging, SetupExperiment, EnterInitializationMode, ExitInitializationMode, Terminate, Reset, SetXXX, GetXXX,
    Serialize, Deserialize, GetDirectionalDerivative, SetInputDerivatives, GetOutputDerivatives, DoStep, CancelStep,
    GetXXXStatus,
}

public class Launch {

    private static FMI2FMU getFMU() {
        return new Adder();
    }

    public static void main(String[] args) {

        ArgumentParser parser = ArgumentParsers.newFor("UniFMU Java Backend").build().defaultHelp(true)
                .description("Application layer protocol for implementing FMUs in Java using ZMQ");

        parser.addArgument("--handshake-endpoint").help(
                "Endpoint string created by the wrapper which is used to perform handshake when the slave has started");

        Namespace ns = null;
        try {
            ns = parser.parseArgs(args);
            System.out.println(ns);
        } catch (ArgumentParserException e) {
            parser.handleError(e);
            System.exit(1);
        }

        FMI2FMU fmu = getFMU();

        try (ZContext context = new ZContext()) {

            ZMQ.Socket handshakeSocket = context.createSocket(SocketType.PUSH);
            ZMQ.Socket commandSocket = context.createSocket(SocketType.REP);
            handshakeSocket.connect(ns.getString("handshake_endpoint"));

            commandSocket.bindToRandomPort("tcp://127.0.0.1");

            // send handshake
            HashMap<String, String> handshakeInfo = new HashMap<>();
            handshakeInfo.put("serialization_format", "FlexBuffer");
            handshakeInfo.put("command_endpoint", commandSocket.getLastEndpoint());

            Gson gson = new Gson();
            String handshake_json = gson.toJson(handshakeInfo);
            handshakeSocket.send(handshake_json);

            ByteBuffer bb = ByteBuffer.allocate(100);
            while (!Thread.currentThread().isInterrupted()) {
                // Block until a message is received

                commandSocket.recvByteBuffer(bb, 0);

                FlexBuffers.Vector vec = FlexBuffers.getRoot(bb).asVector();

                int commandKind = vec.get(0).asInt();

                switch (commandKind) {
                    case 0:
                        FlexBuffers.Vector categories = vec.get(1).asVector();
                        boolean loggingOn = vec.get(2).asBoolean();
                        break;
                    default:
                        break;
                }

                // Send a response
                String response = "Hello, world!";

                System.out.println("Im dont cya later");

                // socket.send(response.getBytes(ZMQ.CHARSET), 0);
            }
        }

    }
}
