import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;

public class Model {

    public Double real_a = 0.0;
    public Double real_b = 0.0;
    public Double real_c = 0.0;
    public Integer integer_a = 0;
    public Integer integer_b = 0;
    public Integer integer_c = 0;
    public Boolean boolean_a = false;
    public Boolean boolean_b = false;
    public Boolean boolean_c = false;
    public String string_a = "";
    public String string_b = "";
    public String string_c = "";

    private ArrayList<Field> references_to_attributes;

    public Model() throws Exception {

        super();
        this.references_to_attributes = new ArrayList<Field>();
        this.references_to_attributes.add(this.getClass().getField("real_a"));
        this.references_to_attributes.add(this.getClass().getField("real_b"));
        this.references_to_attributes.add(this.getClass().getField("real_c"));
        this.references_to_attributes.add(this.getClass().getField("integer_a"));
        this.references_to_attributes.add(this.getClass().getField("integer_b"));
        this.references_to_attributes.add(this.getClass().getField("integer_c"));
        this.references_to_attributes.add(this.getClass().getField("boolean_a"));
        this.references_to_attributes.add(this.getClass().getField("boolean_b"));
        this.references_to_attributes.add(this.getClass().getField("boolean_c"));
        this.references_to_attributes.add(this.getClass().getField("string_a"));
        this.references_to_attributes.add(this.getClass().getField("string_a"));
        this.references_to_attributes.add(this.getClass().getField("string_a"));

    }

    public Fmi2Status fmi2DoStep(double current_time, double step_size, boolean noStepPrior) {
        update_outputs();
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetReal(Iterable<Integer> references, Iterable<Double> values) throws Exception {

        SetValue(references, values);
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetInteger(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        SetValue(references, values);
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetBoolean(Iterable<Integer> references, Iterable<Boolean> values) throws Exception {
        SetValue(references, values);
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetString(Iterable<Integer> references, Iterable<String> values) throws Exception {
        SetValue(references, values);
        return Fmi2Status.OK;
    }

    public Fmi2GetValuePair<Double> fmi2GetReal(Iterable<Integer> references) throws Exception {

        ArrayList<Double> values = this.GetValue(references);
        return new Fmi2GetValuePair<Double>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<Integer> fmi2GetInteger(Iterable<Integer> references) throws Exception {
        ArrayList<Integer> values = this.GetValue(references);
        return new Fmi2GetValuePair<Integer>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<Boolean> fmi2GetBoolean(Iterable<Integer> references) throws Exception {
        ArrayList<Boolean> values = this.GetValue(references);
        return new Fmi2GetValuePair<Boolean>(Fmi2Status.OK, values);
    }

    public Fmi2GetValuePair<String> fmi2GetString(Iterable<Integer> references) throws Exception {
        ArrayList<String> values = this.GetValue(references);
        return new Fmi2GetValuePair<String>(Fmi2Status.OK, values);
    }

    public Fmi2Status fmi2EnterInitializationMode() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2ExitInitializationMode() {
        update_outputs();
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2SetupExperiment(double start_time, Double stop_time, Double tolerance) {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2Reset() {
        this.real_a = 0.0;
        this.real_c = 0.0;
        this.integer_a = 0;
        this.real_b = 0.0;
        this.integer_b = 0;
        this.integer_c = 0;
        this.boolean_a = false;
        this.boolean_b = false;
        this.boolean_c = false;
        this.string_a = "";
        this.string_b = "";
        this.string_c = "";
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2Terminate() {
        return Fmi2Status.OK;
    }

    public Fmi2Status fmi2CancelStep() {
        return Fmi2Status.OK;
    }

    public Fmi2SerializeFmuStatePair fmi2SerializeFmuState() throws Exception {

        var b = new ByteArrayOutputStream();
        var o = new ObjectOutputStream(b);

        o.writeObject(this);

        return new Fmi2SerializeFmuStatePair(Fmi2Status.OK, b.toByteArray());
    }

    public Fmi2Status fmi2DeserializeFmuState(byte[] bytes) throws Exception {

        try (ByteArrayInputStream b = new ByteArrayInputStream(bytes)) {
            try (ObjectInputStream o = new ObjectInputStream(b)) {
                var other = (Model) o.readObject();
                this.real_a = other.real_a;
                this.real_b = other.real_b;
                this.real_c = other.real_c;
                this.integer_a = other.integer_a;
                this.integer_b = other.integer_b;
                this.integer_c = other.integer_c;
                this.boolean_a = other.boolean_a;
                this.boolean_b = other.boolean_b;
                this.boolean_c = other.boolean_c;
                this.string_a = other.string_a;
                this.string_b = other.string_b;
                this.string_c = other.string_c;
            }
        }

        return Fmi2Status.OK;
    }

    private <T> ArrayList<T> GetValue(Iterable<Integer> references) throws Exception {
        var values = new ArrayList<T>();

        for (var ref : references) {

            @SuppressWarnings("unchecked")
            var val = (T) this.references_to_attributes.get(ref).get(this);
            values.add(val);
        }

        return values;

    }

    private <T> void SetValue(Iterable<Integer> references, Iterable<T> values) throws Exception {

        Iterator<Integer> i1 = references.iterator();
        Iterator<T> i2 = values.iterator();
        while (i1.hasNext() && i2.hasNext()) {
            this.references_to_attributes.get(i1.next()).set(this, i2.next());
        }
    }

    private void update_outputs() {
        this.real_c = this.real_a + this.real_b;
        this.integer_c = this.integer_a + this.integer_b;
        this.boolean_c = this.boolean_a || this.boolean_b;
        this.string_c = this.string_a + this.string_b;

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

    class Fmi2SerializeFmuStatePair {
        public Fmi2Status status;
        public byte[] bytes;

        Fmi2SerializeFmuStatePair(Fmi2Status status, byte[] bytes) {
            this.status = status;
            this.bytes = bytes;
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
