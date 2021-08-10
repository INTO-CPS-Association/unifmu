package fmu;

import net.sourceforge.argparse4j.ArgumentParsers;
import net.sourceforge.argparse4j.inf.ArgumentParser;
import net.sourceforge.argparse4j.inf.ArgumentParserException;
import net.sourceforge.argparse4j.inf.Namespace;
import fmu.Fmi2Proto.HandshakeInfo;
import fmu.Fmi2Proto.StatusReturn;
import io.grpc.ManagedChannelBuilder;
import io.grpc.Server;
import io.grpc.ServerBuilder;

import java.io.IOException;

import fmu.Fmi2Proto.FmiStatus;
import io.grpc.stub.StreamObserver;

public class Launch {

    private static class CommandService extends SendCommandGrpc.SendCommandImplBase {

        private FMI2FMU fmu;

        CommandService(FMI2FMU fmu) {
            this.fmu = fmu;
        }

        public void fmi2SetupExperiment(Fmi2Proto.SetupExperiment request,
                StreamObserver<StatusReturn> responseObserver) {

            var status = StatusReturn.newBuilder().setStatus(FmiStatus.Ok).build();

            fmu.enterInitializationMode();

            responseObserver.onNext(status);
            responseObserver.onCompleted();
        }

    }

    private static class CommandServer {

        private final Server server;

        CommandServer(int port, FMI2FMU slave) {

            this.server = ServerBuilder.forPort(port).addService(new CommandService(slave)).build();
        }

        void awaitTermination() throws InterruptedException {
            this.server.awaitTermination();
        }

        void start() throws IOException {
            this.server.start();
        }

        void stop() throws InterruptedException {
            this.server.awaitTermination();
        }
    }

    public static void main(String[] args) throws Exception {

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

        int command_port = 50020;
        FMI2FMU fmu = new Adder();
        CommandServer server = new CommandServer(command_port, fmu);
        server.start();

        var channel = ManagedChannelBuilder.forAddress("TODO", 50000).usePlaintext().build();
        var client = HandshakerGrpc.newBlockingStub(channel);
        var msg = HandshakeInfo.newBuilder().setIpAddress("TODO").setPort("10").build();
        client.performHandshake(msg);

        server.awaitTermination();

    }
}
