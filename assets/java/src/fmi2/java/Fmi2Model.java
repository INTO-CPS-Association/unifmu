package org.intocps.unifmu.model.fmi2;

import java.util.List;
import java.util.function.Consumer;

public interface Fmi2Model {
    enum Fmi2Status {
        OK, Warning, Discard, Error, Fatal, Pending
    }

    record LogMessage(Fmi2Status status, String category, String message) {}

    class Fmi2GetValuePair<T> {
        public Fmi2Status status;
        public List<T> values;

        public Fmi2GetValuePair(Fmi2Status status, List<T> values) {
            this.status = status;
            this.values = values;
        }
    }

    class Fmi2SerializeFmuStatePair {
        public Fmi2Status status;
        public byte[] bytes;

        public Fmi2SerializeFmuStatePair(Fmi2Status status, byte[] bytes) {
            this.status = status;
            this.bytes = bytes;
        }
    }

    void setLogHandler(Consumer<LogMessage> logHandler);
    Fmi2Status fmi2DoStep(double currentTime, double stepSize, boolean noStepPrior) throws Exception;
    Fmi2Status fmi2SetReal(Iterable<Integer> references, Iterable<Double> values) throws Exception;
    Fmi2Status fmi2SetInteger(Iterable<Integer> references, Iterable<Integer> values) throws Exception;
    Fmi2Status fmi2SetBoolean(Iterable<Integer> references, Iterable<Boolean> values) throws Exception;
    Fmi2Status fmi2SetString(Iterable<Integer> references, Iterable<String> values) throws Exception;
    Fmi2GetValuePair<Double> fmi2GetReal(Iterable<Integer> references) throws Exception;
    Fmi2GetValuePair<Integer> fmi2GetInteger(Iterable<Integer> references) throws Exception;
    Fmi2GetValuePair<Boolean> fmi2GetBoolean(Iterable<Integer> references) throws Exception;
    Fmi2GetValuePair<String> fmi2GetString(Iterable<Integer> references) throws Exception;
    Fmi2Status fmi2EnterInitializationMode() throws Exception;
    Fmi2Status fmi2ExitInitializationMode() throws Exception;
    Fmi2Status fmi2SetupExperiment(double startTime, Double stopTime, Double tolerance) throws Exception;
    Fmi2Status fmi2Reset() throws Exception;
    Fmi2Status fmi2Terminate() throws Exception;
    Fmi2Status fmi2CancelStep() throws Exception;
    Fmi2SerializeFmuStatePair fmi2SerializeFmuState() throws Exception;
    Fmi2Status fmi2DeserializeFmuState(byte[] bytes) throws Exception;
    Fmi2Status fmi2SetDebugLogging(Iterable<String> categories, Boolean loggingOn) throws Exception;
}
