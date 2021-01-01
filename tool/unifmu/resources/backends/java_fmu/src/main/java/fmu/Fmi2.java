package fmu;

enum FMI2Status {
    OK, WARNING, DISCARD, ERROR, FATAL, PENDING,
}

class FMI2FMU {
    /*
     * Base class defining default implementations for FMI2 related methods
     */
    public FMI2Status doStep(double currentTime, double stepSize, boolean noStepPrior) {
        return FMI2Status.OK;
    }

    public FMI2Status setupExperiment(double startTime, Double stopTime, Double tolerance) {
        return FMI2Status.OK;
    }

    public FMI2Status setDebugLogging(String[] categories, boolean loggingOn) {
        return FMI2Status.OK;
    }

    public FMI2Status enterInitializationMode() {
        return FMI2Status.OK;
    }

    public FMI2Status exitInitializationMode() {
        return FMI2Status.OK;
    }

    public FMI2Status terminate() {
        return FMI2Status.OK;
    }

    public FMI2Status reset() {
        return FMI2Status.OK;
    }

    public FMI2Status cancelStep() {
        return FMI2Status.OK;
    }

    public byte[] serialize() throws RuntimeException {
        throw new RuntimeException(
                "the serialization method is not defined. Either define this method in the subclass, or set the 'canGetAndSetFMUstate' and 'canGetAndSetFMUstate' to false in the modelDescription.xml to indicate that the FMU does not support this functionality");
    }

    public FMI2Status deserialize(byte[] state) throws RuntimeException {
        throw new RuntimeException(
                "the deserialization method is not defined. Either define this method in the subclass, or set the 'canGetAndSetFMUstate' and 'canGetAndSetFMUstate' to false in the modelDescription.xml to indicate that the FMU does not support this functionality");
    }
}
