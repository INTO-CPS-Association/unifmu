import java.util.ArrayList;
import java.util.List;

public class Model {

    public Model() {
        super();
    }

    public Fmi2Status fmi2DoStep(double current_time, double step_size, boolean noStepPrior) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetReal(Iterable<Integer> references, Iterable<Double> values) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetInteger(Iterable<Integer> references, Iterable<Integer> values) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetBoolean(Iterable<Integer> references, Iterable<Boolean> values) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetString(Iterable<Integer> references, Iterable<String> values) {
        return Fmi2Status.OK;
    }

    public Fmi2GetValuePair<Double> fmi2GetReal(Iterable<Integer> references) {
        var values = new ArrayList<Double>();
        return new Fmi2GetValuePair<Double>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<Integer> fmi2GetInteger(Iterable<Integer> references) {
        var values = new ArrayList<Integer>();
        return new Fmi2GetValuePair<Integer>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<Boolean> fmi2GetBoolean(Iterable<Integer> references) {
        var values = new ArrayList<Boolean>();
        return new Fmi2GetValuePair<Boolean>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<String> fmi2GetString(Iterable<Integer> references) {
        var values = new ArrayList<String>();
        return new Fmi2GetValuePair<String>(Fmi2Status.OK, values);
    }

    public Fmi2Status fmi2EnterInitializationMode() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2ExitInitializationMode() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetupExperiment(double start_time, Double stop_time, Double tolerance) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2Reset() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2Terminate() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2CancelStep() {
        return Fmi2Status.OK;
    }

    class Fmi2GetValuePair<T> {
        Fmi2Status status;
        List<T> values;

        Fmi2GetValuePair(Fmi2Status status, List<T> values)

        {
            this.status = status;
            this.values = values;
        }
    }

    enum Fmi2Status {
        OK,
        Warning,
        Discard,
        Error,
        Fatal,
        Pending
    }

}
