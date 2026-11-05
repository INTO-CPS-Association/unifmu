package org.intocps.unifmu;

import com.moandjiezana.toml.Toml;
import java.io.BufferedReader;
import java.io.File;
import java.io.InputStreamReader;
import org.intocps.unifmu.communication.ChannelSelection;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.model.fmi2.ModelSelection;
import org.intocps.unifmu.protocol.fmi2.Fmi2Protocol;
import org.intocps.unifmu.protocol.fmi3.Fmi3Protocol;

public final class Main {
    private Main() {}

    public static void main(String[] args) throws Exception {
        String launchMode = System.getProperty("unifmu.launch.mode", "auto");
        String endpoint = switch (launchMode) {
            case "local" -> localEndpoint();
            case "remote" -> remoteEndpoint(args);
            case "auto" -> {
                String dispatcherEndpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");
                yield dispatcherEndpoint == null || dispatcherEndpoint.isBlank()
                    ? remoteEndpoint(args)
                    : dispatcherEndpoint;
            }
            default -> throw new IllegalArgumentException(
                "Set the JVM property 'unifmu.launch.mode' to 'local', 'remote', or 'auto'."
            );
        };

        try (CommunicationChannel channel = ChannelSelection.connect(endpoint)) {
            switch (System.getProperty("unifmu.fmi.version", "").toLowerCase()) {
                case "2", "fmi2" ->
                    new Fmi2Protocol(channel, ModelSelection.fmi2Factory()).run();
                case "3", "fmi3" ->
                    new Fmi3Protocol(
                        channel,
                        org.intocps.unifmu.model.fmi3.ModelSelection.fmi3Factory()
                    ).run();
                default -> throw new IllegalArgumentException(
                    "Set the JVM property 'unifmu.fmi.version' to 'fmi2' or 'fmi3'."
                );
            }
        }
    }

    private static String localEndpoint() {
        String endpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");
        if (endpoint == null || endpoint.isBlank()) {
            throw new IllegalStateException(
                "UNIFMU_DISPATCHER_ENDPOINT is required for a local FMU backend."
            );
        }
        return endpoint;
    }

    private static String remoteEndpoint(String[] args) throws Exception {
        String port;
        if (args.length == 1) {
            port = args[0];
        } else {
            System.out.println("Input the port for remote proxy FMU:");
            port = new BufferedReader(new InputStreamReader(System.in)).readLine();
        }
        int parsedPort;
        try {
            parsedPort = Integer.parseInt(port);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException("Remote FMU port must be an integer.", e);
        }
        if (parsedPort < 1 || parsedPort > 65535) {
            throw new IllegalArgumentException("Remote FMU port must be between 1 and 65535.");
        }

        String ip = new Toml().read(new File("endpoint.toml")).getString("ip");
        if (ip == null || ip.isBlank()) {
            throw new IllegalArgumentException("endpoint.toml must define a non-empty 'ip'.");
        }
        return "tcp://" + ip + ":" + parsedPort;
    }
}
