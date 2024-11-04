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

        String dispatcher_endpoint = System.getenv("UNIFMU_DISPATCHER_ENDPOINT");
        String dispatcher_endpoint_port = System.getenv("UNIFMU_DISPATCHER_ENDPOINT_PORT");

        System.out.println("Proxy dispatcher endpoint: " + dispatcher_endpoint + ".");
        System.out.println("Proxy dispatcher endpoint port: " + dispatcher_endpoint_port + ".");
        System.out.println(YELLOW + "Use this port to connect the remote (private) FMU model: " + BOLD + BACKGROUNDGREEN + "'" + dispatcher_endpoint_port + "'" + RESET);
    }
}
