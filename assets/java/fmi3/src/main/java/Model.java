import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.util.Map;

public class Model {

    public Float float32_a = 0;
    public Float float32_b = 0;
    public Float float32_c = 0;
    public Double float64_a = 0.0;
    public Double float64_b = 0.0;
    public Double float64_c = 0.0;
    public Byte int8_a = 0;
    public Byte int8_b = 0;
    public Byte int8_c = 0;
    public Short uint8_a = 0; // No valid datatype in Java
    public Short uint8_b = 0; // No valid datatype in Java
    public Short uint8_c = 0; // No valid datatype in Java
    public Short int16_a = 0;
    public Short int16_b = 0;
    public Short int16_c = 0;
    public Integer uint16_a = 0; // No valid datatype in Java
    public Integer uint16_b = 0; // No valid datatype in Java
    public Integer uint16_c = 0; // No valid datatype in Java
    public Integer int32_a = 0;
    public Integer int32_b = 0;
    public Integer int32_c = 0;
    public Long uint32_a = 0; // No valid datatype in Java
    public Long uint32_b = 0; // No valid datatype in Java
    public Long uint32_c = 0; // No valid datatype in Java
    public Long int64_a = 0;
    public Long int64_b = 0;
    public Long int64_c = 0;
    public Long uint64_a = 0; // No valid datatype in Java
    public Long uint64_b = 0; // No valid datatype in Java
    public Long uint64_c = 0; // No valid datatype in Java
    public Boolean boolean_a = false;
    public Boolean boolean_b = false;
    public Boolean boolean_c = false;
    public String string_a = "";
    public String string_b = "";
    public String string_c = "";
    public byte[] binary_a = new byte[] {
        (byte) 0b00000000
    };
    public byte[] binary_b = new byte[] {
        (byte) 0b00000000
    };
    public byte[] binary_c = new byte[] {
        (byte) 0b00000000
    };

    public Float float32_tunable_parameter = 0;
    public Double  float64_tunable_parameter = 0.0;
    public Byte int8_tunable_parameter = 0;
    public Short uint8_tunable_parameter = 0;
    public Short int16_tunable_parameter = 0;
    public Integer uint16_tunable_parameter = 0;
    public Integer int32_tunable_parameter = 0;
    public Long uint32_tunable_parameter = 0;
    public Long int64_tunable_parameter = 0;
    public Long uint64_tunable_parameter = 0;
    public Boolean boolean_tunable_parameter = false;
    public String string_tunable_parameter = "";
    public byte[] binary_tunable_parameter = new byte[] {
        (byte) 0b00000000
    };
    public Long uint64_tunable_structural_parameter = 5;
    public Float float32_vector_using_tunable_structural_parameter = new Float[] {
        0.1,
        0.2,
        0.3,
        0.4,
        0.5
    };
    public Boolean clock_a = false;
    public Boolean clock_b = false;
    public Boolean clock_c = false;
    public Integer clocked_variable_a = 0;
    public Integer clocked_variable_b = 0;
    public Integer clocked_variable_c = 0;
    public Map<Integer,Double> clock_reference_to_interval = Map.of(1001, 1.0);
    public Map<Integer,Double> clock_reference_to_shift = Map.of(1001, 1.0);

    private ArrayList<Field> references_to_attributes;

    private ArrayList<Field> clocked_variables;

    private ArrayList<Field> parameters;

    private ArrayList<Field> tunable_parameters;

    private ArrayList<Field> tunable_structural_parameters;

    private ArrayList<Field> all_parameters;

    private ArrayList<Field> all_references;



    public Model() throws Exception {

        super();
        this.references_to_attributes = new ArrayList<Field>();
        this.references_to_attributes.add(this.getClass().getField("float32_a"));
        this.references_to_attributes.add(this.getClass().getField("float32_b"));
        this.references_to_attributes.add(this.getClass().getField("float32_c"));
        this.references_to_attributes.add(this.getClass().getField("float64_a"));
        this.references_to_attributes.add(this.getClass().getField("float64_b"));
        this.references_to_attributes.add(this.getClass().getField("float64_c"));
        this.references_to_attributes.add(this.getClass().getField("int8_a"));
        this.references_to_attributes.add(this.getClass().getField("int8_b"));
        this.references_to_attributes.add(this.getClass().getField("int8_c"));
        this.references_to_attributes.add(this.getClass().getField("uint8_a"));
        this.references_to_attributes.add(this.getClass().getField("uint8_b"));
        this.references_to_attributes.add(this.getClass().getField("uint8_c"));

        this.references_to_attributes.add(this.getClass().getField("int16_a"));
        this.references_to_attributes.add(this.getClass().getField("int16_b"));
        this.references_to_attributes.add(this.getClass().getField("int16_c"));
        this.references_to_attributes.add(this.getClass().getField("uint16_a"));
        this.references_to_attributes.add(this.getClass().getField("uint16_b"));
        this.references_to_attributes.add(this.getClass().getField("uint16_c"));

        this.references_to_attributes.add(this.getClass().getField("int32_a"));
        this.references_to_attributes.add(this.getClass().getField("int32_b"));
        this.references_to_attributes.add(this.getClass().getField("int32_c"));
        this.references_to_attributes.add(this.getClass().getField("uint32_a"));
        this.references_to_attributes.add(this.getClass().getField("uint32_b"));
        this.references_to_attributes.add(this.getClass().getField("uint32_c"));

        this.references_to_attributes.add(this.getClass().getField("int64_a"));
        this.references_to_attributes.add(this.getClass().getField("int64_b"));
        this.references_to_attributes.add(this.getClass().getField("int64_c"));
        this.references_to_attributes.add(this.getClass().getField("uint64_a"));
        this.references_to_attributes.add(this.getClass().getField("uint64_b"));
        this.references_to_attributes.add(this.getClass().getField("uint64_c"));

        this.references_to_attributes.add(this.getClass().getField("boolean_a"));
        this.references_to_attributes.add(this.getClass().getField("boolean_b"));
        this.references_to_attributes.add(this.getClass().getField("boolean_c"));
        this.references_to_attributes.add(this.getClass().getField("string_a"));
        this.references_to_attributes.add(this.getClass().getField("string_b"));
        this.references_to_attributes.add(this.getClass().getField("string_c"));
        this.references_to_attributes.add(this.getClass().getField("binary_a"));
        this.references_to_attributes.add(this.getClass().getField("binary_b"));
        this.references_to_attributes.add(this.getClass().getField("binary_c"));

        this.clocked_variables = new ArrayList<Field>();
        this.clocked_variables.add(this.getClass().getField("clock_a"));
        this.clocked_variables.add(this.getClass().getField("clock_b"));
        this.clocked_variables.add(this.getClass().getField("clock_c"));
        this.clocked_variables.add(this.getClass().getField("clocked_variable_a"));
        this.clocked_variables.add(this.getClass().getField("clocked_variable_b"));
        this.clocked_variables.add(this.getClass().getField("clocked_variable_c"));

        this.parameters = new ArrayList<Field>();

        this.tunable_parameters = new ArrayList<Field>();
        this.tunable_parameters.add(this.getClass().getField("float32_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("float64_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("int8_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("uint8_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("int16_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("uint16_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("int32_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("uint32_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("int64_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("uint64_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("boolean_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("string_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("binary_tunable_parameter"));
        this.tunable_parameters.add(this.getClass().getField("float32_vector_using_tunable_structural_parameter"));

        this.tunable_structural_parameters = new ArrayList<Field>();
        this.tunable_parameters.add(this.getClass().getField("uint64_tunable_structural_parameter"));

        this.all_references = new ArrayList<Field>();
        this.all_references.addAll(self.tunable_structural_parameters);
        this.all_references.addAll(self.parameters);
        this.all_references.addAll(self.tunable_parameters);
        this.all_references.addAll(self.clocked_variables);
        this.all_references.addAll(self.references_to_attributes);

        this.all_parameters = new ArrayList<Field>();
        this.all_parameters.addAll(self.tunable_structural_parameters);
        this.all_parameters.addAll(self.parameters);
        this.all_parameters.addAll(self.tunable_parameters);        

    }

    public Fmi3Status fmi3DoStep(double current_time, double step_size, boolean noStepPrior) {
        update_outputs();
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetReal(Iterable<Integer> references, Iterable<Double> values) throws Exception {

        SetValue(references, values);
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetInteger(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        SetValue(references, values);
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetBoolean(Iterable<Integer> references, Iterable<Boolean> values) throws Exception {
        SetValue(references, values);
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetString(Iterable<Integer> references, Iterable<String> values) throws Exception {
        SetValue(references, values);
        return Fmi3Status.OK;
    }

    public Fmi3GetValuePair<Double> fmi3GetReal(Iterable<Integer> references) throws Exception {

        ArrayList<Double> values = this.GetValue(references);
        return new Fmi3GetValuePair<Double>(Fmi3Status.OK, values);
    }

    public Fmi3GetValuePair<Integer> fmi3GetInteger(Iterable<Integer> references) throws Exception {
        ArrayList<Integer> values = this.GetValue(references);
        return new Fmi3GetValuePair<Integer>(Fmi3Status.OK, values);
    }

    public Fmi3GetValuePair<Boolean> fmi3GetBoolean(Iterable<Integer> references) throws Exception {
        ArrayList<Boolean> values = this.GetValue(references);
        return new Fmi3GetValuePair<Boolean>(Fmi3Status.OK, values);
    }

    public Fmi3GetValuePair<String> fmi3GetString(Iterable<Integer> references) throws Exception {
        ArrayList<String> values = this.GetValue(references);
        return new Fmi3GetValuePair<String>(Fmi3Status.OK, values);
    }

    public Fmi3Status fmi3EnterInitializationMode() {
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3ExitInitializationMode() {
        update_outputs();
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetupExperiment(double start_time, Double stop_time, Double tolerance) {
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3Reset() {
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
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3Terminate() {
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3CancelStep() {
        return Fmi3Status.OK;
    }

    public Fmi3SerializeFmuStatePair fmi3SerializeFmuState() throws Exception {

        var b = new ByteArrayOutputStream();
        var o = new ObjectOutputStream(b);

        o.writeObject(this);

        return new Fmi3SerializeFmuStatePair(Fmi3Status.OK, b.toByteArray());
    }

    public Fmi3Status fmi3DeserializeFmuState(byte[] bytes) throws Exception {

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

        return Fmi3Status.OK;
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

    class Fmi3GetValuePair<T> {
        Fmi3Status status;
        List<T> values;

        Fmi3GetValuePair(Fmi3Status status, List<T> values)

        {
            this.status = status;
            this.values = values;
        }
    }

    class Fmi3SerializeFmuStatePair {
        public Fmi3Status status;
        public byte[] bytes;

        Fmi3SerializeFmuStatePair(Fmi3Status status, byte[] bytes) {
            this.status = status;
            this.bytes = bytes;
        }
    }

    enum Fmi3Status {
        OK,
        Warning,
        Discard,
        Error,
        Fatal,
        Pending
    }

}
