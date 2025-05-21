import org.zeromq.ZContext;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.File;
import java.util.Scanner;

import com.moandjiezana.toml.Toml;

public class Backend extends AbstractBackend {

    public static void main(String[] args) throws Exception {
        boolean inputOk = false;
        String port_str = "";
        int port_int = 0;

        if (args.length > 0 && args.length == 1) {
            try {
                port_str = args[0];
                port_int = Integer.parseInt(port_str);
                inputOk = true;
            } catch(NumberFormatException nfe) {
                System.err.println(RED + "Only one argument for the port in integer format is accepted." + RESET);
            }
        }

        while (!inputOk) {
            System.out.println(BACKGROUNDGREEN + "Input the port for remote proxy FMU:" + RESET);
            port_str = new BufferedReader(new InputStreamReader(System.in)).readLine();
            try {
                port_int = Integer.parseInt(port_str);
                inputOk = true;
            } catch(NumberFormatException nfe) {
                System.err.println(RED + "Only integers accepted." + RESET);
            }
        }

        Toml toml = new Toml().read(new File("endpoint.toml"));
        String proxy_ip_address = toml.getString("ip");

        String dispatcher_endpoint = "tcp://" + proxy_ip_address + ":" + port_str;
        System.out.println(YELLOW + "Dispatcher endpoint received:" + BOLD + BACKGROUNDGREEN + dispatcher_endpoint + RESET);

        try (ZContext context = new ZContext()) {
            connectToEndpoint(context, dispatcher_endpoint);
            System.out.println(YELLOW + "Socket connected successfully." + RESET);

            handshake();
            commandReplyLoop();
        }
    }

    private static final String RESET = "\u001B[0m";
    private static final String RED = "\u001B[31m";
    private static final String YELLOW = "\u001B[33m";
    private static final String BOLD = "\033[0;1m";
    private static final String BACKGROUNDGREEN = "\u001B[42m";
}