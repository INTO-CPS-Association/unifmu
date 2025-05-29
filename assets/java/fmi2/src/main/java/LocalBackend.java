import org.zeromq.ZContext;

public class Backend extends AbstractBackend {
    public static void main(String[] args) throws Exception {
        System.out.println("starting FMU");

        try (ZContext context = new ZContext()) {
            connectToEndpoint(
                context,
                System.getenv("UNIFMU_DISPATCHER_ENDPOINT")
            );
            handshake();
            commandReplyLoop();
        }
    }
}