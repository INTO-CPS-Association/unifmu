public class Backend {

    public static void main(String[] args) throws Exception {
        String RESET = "\u001B[0m";
        String RED = "\u001B[31m";
        String GREEN = "\u001B[32m";
        String YELLOW = "\u001B[33m";
        String BOLD = "\033[0;1m";
        String BACKGROUNDGREEN = "\u001B[42m";
        String BACKGROUNDYELLOW = "\u001B[43m";
        System.out.println("Setting up proxy FMU backend.");

        String dispacher_endpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");
        String dispacher_endpoint_port = System.getenv("UNIFMU_DISPATCHER_ENDPOINT_PORT");

        System.out.println("Proxy dispatcher endpoint: " + dispacher_endpoint + ".");
        System.out.println("Proxy dispatcher endpoint port: " + dispacher_endpoint_port + ".");
        System.out.println(YELLOW + "Use this port to connect the remote (private) FMU model: " + BOLD + BACKGROUNDYELLOW + "'" + dispacher_endpoint_port + "'" + RESET);
    }
}
