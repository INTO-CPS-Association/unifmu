import java.lang.reflect.Field;
import java.math.BigInteger;
import java.math.BigDecimal;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.util.Map;
import java.io.Serializable;


public class Model implements Serializable {
    // No valid datatypes between Protobuf and Java for int8, uint8, int16, uint16 -> all handled as Integer
    // No valid datatypes in Java for uint64 -> handled as Long

    private String instance_name = "";
    private String instantiation_token = "";
    private String resource_path = "";
    private Boolean visible = false;
    private Boolean logging_on = false;
    private Boolean event_mode_used = false;
    private Boolean early_return_allowed = false;
    private transient List<Integer> required_intermediate_variables;

    private int state = FMIState.FMIInstantiatedState;
    public Float float32_a = 0.0f;
    public Float float32_b = 0.0f;
    public Float float32_c = 0.0f;
    public Double float64_a = 0.0;
    public Double float64_b = 0.0;
    public Double float64_c = 0.0;
    public Integer int8_a = 0;
    public Integer int8_b = 0;
    public Integer int8_c = 0;
    public Integer uint8_a = 0; 
    public Integer uint8_b = 0; 
    public Integer uint8_c = 0; 
    public Integer int16_a = 0;
    public Integer int16_b = 0;
    public Integer int16_c = 0;
    public Integer uint16_a = 0; 
    public Integer uint16_b = 0; 
    public Integer uint16_c = 0; 
    public Integer int32_a = 0;
    public Integer int32_b = 0;
    public Integer int32_c = 0;
    public Integer uint32_a = 0; 
    public Integer uint32_b = 0; 
    public Integer uint32_c = 0; 
    public Long int64_a = 0L;
    public Long int64_b = 0L;
    public Long int64_c = 0L;
    public Long uint64_a = 0L; 
    public Long uint64_b = 0L; 
    public Long uint64_c = 0L; 
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

    public Float float32_tunable_parameter = 0.0f;
    public Double  float64_tunable_parameter = 0.0;
    public Integer int8_tunable_parameter = 0;
    public Integer uint8_tunable_parameter = 0;
    public Integer int16_tunable_parameter = 0;
    public Integer uint16_tunable_parameter = 0;
    public Integer int32_tunable_parameter = 0;
    public Integer uint32_tunable_parameter = 0;
    public Long int64_tunable_parameter = 0L;
    public Long uint64_tunable_parameter = 0L;
    public Boolean boolean_tunable_parameter = false;
    public String string_tunable_parameter = "";
    public byte[] binary_tunable_parameter = new byte[] {
        (byte) 0b00000000
    };
    public Long uint64_tunable_structural_parameter = 5L;
    public Float[] float32_vector_using_tunable_structural_parameter = new Float[] {
        0.1f,
        0.2f,
        0.3f,
        0.4f,
        0.5f
    };
    public Boolean clock_a = false;
    public Boolean clock_b = false;
    public Boolean clock_c = false;
    public Integer clocked_variable_a = 0;
    public Integer clocked_variable_b = 0;
    public Integer clocked_variable_c = 0;
    public Map<Integer,Double> clock_reference_to_interval;
    public Map<Integer,Double> clock_reference_to_shift;


    public transient Map<Integer,Field> map_to_attributes;

    private transient ArrayList<Field> references_to_attributes;

    private transient ArrayList<Field> clocked_variables;

    private transient ArrayList<Field> parameters;

    private transient ArrayList<Field> tunable_parameters;

    private transient ArrayList<Field> tunable_structural_parameters;

    private transient ArrayList<Field> all_parameters;

    public Model(String instance_name, String instantiation_token, String resource_path, Boolean visible, Boolean logging_on, Boolean event_mode_used, Boolean early_return_allowed, List<Integer> required_intermediate_variables) throws Exception {

        super();

        
        this.instance_name = instance_name;
        this.instantiation_token = instantiation_token;
        this.resource_path = resource_path;
        this.visible = visible;
        this.logging_on = logging_on;
        this.event_mode_used = false;
        this.early_return_allowed = early_return_allowed;
        this.required_intermediate_variables = required_intermediate_variables;

        this.clock_reference_to_interval = new HashMap<>();
        this.clock_reference_to_interval.put(1001, 1.0);

        this.clock_reference_to_shift = new HashMap<>();
        this.clock_reference_to_shift.put(1001, 1.0);
        
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
        this.tunable_structural_parameters.add(this.getClass().getField("uint64_tunable_structural_parameter"));
       
        // Map of indexes to attributes (based on ModelDescription.xml)
        this.map_to_attributes = new HashMap<>();
        List<Integer> references_to_attributes_indexes = new ArrayList<>();
        List<Integer> clocked_variables_indexes = new ArrayList<>();
        List<Integer> parameters_indexes = new ArrayList<>();
        List<Integer> tunable_parameters_indexes = new ArrayList<>();
        List<Integer> tunable_structural_parameters_indexes = new ArrayList<>();
        for (int i=0;i<=38;i++){
            references_to_attributes_indexes.add(i);
        }
        for (int i=1001;i<=1003;i++){
            clocked_variables_indexes.add(i);
        }
        for (int i=1100;i<=1102;i++){
            clocked_variables_indexes.add(i);
        }
        for (int i=100;i<=112;i++){
            tunable_parameters_indexes.add(i);
        }
        tunable_parameters_indexes.add(114);
        tunable_structural_parameters_indexes.add(113);
        for (int j = 0; j < references_to_attributes_indexes.size(); j++) {
            this.map_to_attributes.put(references_to_attributes_indexes.get(j), references_to_attributes.get(j));
        }
        for (int j = 0; j < clocked_variables_indexes.size(); j++) {
            this.map_to_attributes.put(clocked_variables_indexes.get(j), clocked_variables.get(j));
        }
        for (int j = 0; j < tunable_parameters_indexes.size(); j++) {
            this.map_to_attributes.put(tunable_parameters_indexes.get(j), tunable_parameters.get(j));
        }
        for (int j = 0; j < parameters_indexes.size(); j++) {
            this.map_to_attributes.put(parameters_indexes.get(j), parameters.get(j));
        }
        for (int j = 0; j < tunable_structural_parameters_indexes.size(); j++) {
            this.map_to_attributes.put(tunable_structural_parameters_indexes.get(j), tunable_structural_parameters.get(j));
        }
     
        this.all_parameters = new ArrayList<Field>(); 
        this.all_parameters.addAll(this.parameters);
        this.all_parameters.addAll(this.tunable_parameters);
        this.all_parameters.addAll(this.tunable_structural_parameters);        

        update_outputs();
        update_clocks();
        update_clocked_outputs();

    }

    public Fmi3Status fmi3SetDebugLogging(Iterable<String> categories, Boolean logging_on) {
        return Fmi3Status.OK;
    }

    /**
     * Sends a logging message to the importer of the FMU.
     * 
     * In its basic form, when this function is called it sends a message to
     * the UniFMU API layer which then decides whether or not to forward that
     * message to the program importing this FMU. This decision is based on the
     * value of the category parameter and information given to the API layer
     * by the FMU importer. 
     * 
     * The importer can turn all logging on or off, or signal that it is only
     * interested in a subset of logging categories. This filtering is handled
     * by the UniFMU API layer already and cannot be disabled. Blanket
     * enabling/disabling is communicated at instantiation, and full and fine
     * control is done through calls to fmi2SetDebugLogging(). 
     * 
     * Expand on this function to increase functionality or leave it as is to
     * simply send log events to the FMU importer through the UniFMU API layer.
     * 
     * @param message The message to be logged.
     * @param status The status of the FMU at the moment of logging. This is
     *   used to determine the severity of the message.
     * @param category The logging category that this message falls under.
     *   This is used by the UniFMU API layer and the FMU importer to determine
     *   whether or not it is interested in the message. The category can have
     *   any value, but only values set in the modelDescription.xml are valid
     *   and recognized by the FMU importer. The following categories are 
     *   predefined by the FMI3 standard and are included in the
     *   modelDescription.xml by default:
     *     - logStatusWarning
     *     - logStatusDiscard
     *     - logStatusError
     *     - logStatusFatal
     *     - logEvents
     *   If custom categories are defined, make sure to include them in the
     *   modelDescription.xml AND ensure that the importer doesn't disable
     *   them.
     */
    public void log(String message, Fmi3Status status, String category) {
        Backend.loggingCallback(status, category, message);
    }

    /* doStep and updateDiscreteStates */

    public Fmi3DoStepResult fmi3DoStep(double currentCommunicationPoint, double communicationStepSize, boolean noStepPrior) {

        update_outputs();
        Boolean event_handling_needed = false;
        Boolean terminate_simulation = false;
        Boolean early_return = false;
        Double last_successful_time = currentCommunicationPoint + communicationStepSize;

        return new Fmi3DoStepResult<>(Fmi3Status.OK,event_handling_needed,terminate_simulation,early_return,last_successful_time);
    }

    public Fmi3UpdateDiscreteStatesResult fmi3UpdateDiscreteStates(){
        Boolean discrete_states_need_update = false;
        Boolean terminate_simulation = false;
        Boolean nominals_continuous_states_changed = false;
        Boolean values_continuous_states_changed = false;
        Boolean next_event_time_defined = true;
        Double next_event_time = 1.0;


        update_clocked_outputs();

        return new Fmi3UpdateDiscreteStatesResult<>(Fmi3Status.OK, discrete_states_need_update, terminate_simulation, nominals_continuous_states_changed, values_continuous_states_changed, next_event_time_defined, next_event_time);
    }
        

    /* Setters */

    public Fmi3Status fmi3SetFloat32(Iterable<Integer> references, Iterable<Float> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetFloat64(Iterable<Integer> references, Iterable<Double> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetInt8(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetUInt8(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetInt16(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetUInt16(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetInt32(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetUInt32(Iterable<Integer> references, Iterable<Integer> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetInt64(Iterable<Integer> references, Iterable<Long> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetUInt64(Iterable<Integer> references, Iterable<Long> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetBoolean(Iterable<Integer> references, Iterable<Boolean> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetString(Iterable<Integer> references, Iterable<String> values) throws Exception {
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetBinary(Iterable<Integer> references, Iterable<Long> valueSizes, Iterable<ByteBuffer> values) throws Exception {
        // Store 'valueSizes' somewhere if needed
        Fmi3Status status = SetValue(references, values);
        return status;
    }

    public Fmi3Status fmi3SetClock(Iterable<Integer> references, Iterable<Boolean> values) throws Exception {
        
        Fmi3Status status = SetValue(references, values);
        update_clocks();
        return status;
    }

    public Fmi3Status fmi3SetIntervalDecimal(Iterable<Integer> references, Iterable<Double> intervals) {
        Iterator<Integer> i1 = references.iterator();
        Iterator<Double> i2 = intervals.iterator();
        while (i1.hasNext() && i2.hasNext()) {
            Integer ref = i1.next();
            Double interval = i2.next();
            clock_reference_to_interval.put(ref, interval);
        }
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetIntervalFraction(Iterable<Integer> references, Iterable<Long> counters, Iterable<Long> resolutions) {
        Iterator<Integer> i1 = references.iterator();
        Iterator<Long> i2 = counters.iterator();
        Iterator<Long> i3 = resolutions.iterator();
        while (i1.hasNext() && i2.hasNext() && i3.hasNext()) {
            Integer ref = i1.next();
            Double interval = i2.next().doubleValue() / i3.next().doubleValue();
            clock_reference_to_interval.put(ref, interval);
        }
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetShiftDecimal(Iterable<Integer> references, Iterable<Double> shifts) {
        Iterator<Integer> i1 = references.iterator();
        Iterator<Double> i2 = shifts.iterator();
        while (i1.hasNext() && i2.hasNext()) {
            Integer ref = i1.next();
            Double shift = i2.next();
            clock_reference_to_shift.put(ref, shift);
        }
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3SetShiftFraction(Iterable<Integer> references, Iterable<Long> counters, Iterable<Long> resolutions) {
        Iterator<Integer> i1 = references.iterator();
        Iterator<Long> i2 = counters.iterator();
        Iterator<Long> i3 = resolutions.iterator();
        while (i1.hasNext() && i2.hasNext() && i3.hasNext()) {
            Integer ref = i1.next();
            Double shift = i2.next().doubleValue() / i3.next().doubleValue();
            clock_reference_to_shift.put(ref, shift);
        }
        return Fmi3Status.OK;
    }

    /* Getters */

    public Fmi3GetValuePair<Float> fmi3GetFloat32(Iterable<Integer> references) throws Exception {

        Fmi3GetValuePair<Float> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Double> fmi3GetFloat64(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Double> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetInt8(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetUInt8(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetInt16(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetUInt16(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetInt32(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Integer> fmi3GetUInt32(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Integer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Long> fmi3GetInt64(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Long> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Long> fmi3GetUInt64(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Long> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Boolean> fmi3GetBoolean(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Boolean> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<String> fmi3GetString(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<String> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<ByteBuffer> fmi3GetBinary(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<ByteBuffer> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetValuePair<Boolean> fmi3GetClock(Iterable<Integer> references) throws Exception {
        Fmi3GetValuePair<Boolean> pair = this.GetValue(references);
        return pair;
    }

    public Fmi3GetIntervalDecimalPair<Double,Integer> fmi3GetIntervalDecimal(Iterable<Integer> references) {
        List<Double> intervals = new ArrayList<>();
        List<Integer> qualifiers = new ArrayList<>();
        Iterator<Integer> i1 = references.iterator();
        while (i1.hasNext()) {
            intervals.add(clock_reference_to_interval.get(i1.next()));
            qualifiers.add(2);
        }
    
        return new Fmi3GetIntervalDecimalPair<Double,Integer>(Fmi3Status.OK, intervals, qualifiers);
    }

    public Fmi3GetIntervalFractionPair<Long,Integer> fmi3GetIntervalFraction(Iterable<Integer> references) {
        List<Long> counters = new ArrayList<>();
        List<Long> resolutions = new ArrayList<>();
        List<Integer> qualifiers = new ArrayList<>();
    
        Iterator<Integer> i1 = references.iterator();
        while (i1.hasNext()) {
            String decimal = String.valueOf(clock_reference_to_interval.get(i1.next()));
            Fraction fraction = new Fraction(decimal);
            counters.add(fraction.getNumerator());
            resolutions.add(fraction.getDenominator());
            qualifiers.add(2);
        }
    
        return new Fmi3GetIntervalFractionPair<Long,Integer>(Fmi3Status.OK, counters, resolutions, qualifiers);
    }

    public Fmi3GetShiftDecimalPair<Double> fmi3GetShiftDecimal(Iterable<Integer> references) {
        List<Double> shifts = new ArrayList<>();
    
        Iterator<Integer> i1 = references.iterator();
        while (i1.hasNext()) {
            shifts.add(clock_reference_to_shift.get(i1.next()));
        }
    
        return new Fmi3GetShiftDecimalPair<Double>(Fmi3Status.OK, shifts);
    }

    public Fmi3GetShiftFractionPair<Long> fmi3GetShiftFraction(Iterable<Integer> references) {
        List<Long> counters = new ArrayList<>();
        List<Long> resolutions = new ArrayList<>();
    
        Iterator<Integer> i1 = references.iterator();
        while (i1.hasNext()) {
            String decimal = String.valueOf(clock_reference_to_shift.get(i1.next()));
            Fraction fraction = new Fraction(decimal);
            counters.add(fraction.getNumerator());
            resolutions.add(fraction.getDenominator());
        }
    
        return new Fmi3GetShiftFractionPair<Long>(Fmi3Status.OK, counters, resolutions);
    }

    /* Initialization, Enter, Termination, and Reset */

    public Fmi3Status fmi3EnterInitializationMode() {
        this.state = FMIState.FMIInitializationModeState;
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3ExitInitializationMode() {
        if (this.event_mode_used) {
            this.state = FMIState.FMIEventModeState;
        } else{
            this.state = FMIState.FMIStepModeState;
        }
        update_outputs();
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3EnterEventMode(){
        this.state = FMIState.FMIEventModeState;
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3EnterStepMode(){
        this.state = FMIState.FMIStepModeState;
        return Fmi3Status.OK;
    }
    
    public Fmi3Status fmi3EnterConfigurationMode(){
        if (this.tunable_structural_parameters.size() > 0) {
            if (this.state == FMIState.FMIInstantiatedState){
                this.state = FMIState.FMIConfigurationModeState;
            } else{
                this.state = FMIState.FMIReconfigurationModeState;
            }
        } else {
            return Fmi3Status.Error;
        }        
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3ExitConfigurationMode(){
        if (this.state == FMIState.FMIConfigurationModeState){
            this.state = FMIState.FMIInstantiatedState;
        }            
        else if (this.state == FMIState.FMIReconfigurationModeState){
            this.state = FMIState.FMIStepModeState;
        }            
        else{
            return Fmi3Status.Error;
        }
        return Fmi3Status.OK;   
    }

    public Fmi3Status fmi3Reset() {
        this.state = FMIState.FMIInstantiatedState;
        this.float32_a = 0.0f;
        this.float32_b = 0.0f;
        this.float32_c = 0.0f;
        this.float64_a = 0.0;
        this.float64_b = 0.0;
        this.float64_c = 0.0;
        this.int8_a = 0;
        this.int8_b = 0;
        this.int8_c = 0;
        this.uint8_a = 0; 
        this.uint8_b = 0; 
        this.uint8_c = 0; 
        this.int16_a = 0;
        this.int16_b = 0;
        this.int16_c = 0;
        this.uint16_a = 0; 
        this.uint16_b = 0; 
        this.uint16_c = 0; 
        this.int32_a = 0;
        this.int32_b = 0;
        this.int32_c = 0;
        this.uint32_a = 0; 
        this.uint32_b = 0; 
        this.uint32_c = 0; 
        this.int64_a = 0L;
        this.int64_b = 0L;
        this.int64_c = 0L;
        this.uint64_a = 0L; 
        this.uint64_b = 0L; 
        this.uint64_c = 0L; 
        this.boolean_a = false;
        this.boolean_b = false;
        this.boolean_c = false;
        this.string_a = "";
        this.string_b = "";
        this.string_c = "";
        this.binary_a = new byte[] {
            (byte) 0b00000000
        };
        this.binary_b = new byte[] {
            (byte) 0b00000000
        };
        this.binary_c = new byte[] {
            (byte) 0b00000000
        };

        this.float32_tunable_parameter = 0.0f;
        this.float64_tunable_parameter = 0.0;
        this.int8_tunable_parameter = 0;
        this.uint8_tunable_parameter = 0;
        this.int16_tunable_parameter = 0;
        this.uint16_tunable_parameter = 0;
        this.int32_tunable_parameter = 0;
        this.uint32_tunable_parameter = 0;
        this.int64_tunable_parameter = 0L;
        this.uint64_tunable_parameter = 0L;
        this.boolean_tunable_parameter = false;
        this.string_tunable_parameter = "";
        this.binary_tunable_parameter = new byte[] {
            (byte) 0b00000000
        };
        this.uint64_tunable_structural_parameter = 5L;
        this.float32_vector_using_tunable_structural_parameter = new Float[] {
            0.1f,
            0.2f,
            0.3f,
            0.4f,
            0.5f
        };
        this.clock_a = false;
        this.clock_b = false;
        this.clock_c = false;
        this.clocked_variable_a = 0;
        this.clocked_variable_b = 0;
        this.clocked_variable_c = 0;
        this.clock_reference_to_interval = new HashMap<>();
        this.clock_reference_to_interval.put(1001, 1.0);
        this.clock_reference_to_shift = new HashMap<>();
        this.clock_reference_to_shift.put(1001, 1.0);
        update_outputs();
        update_clocks();
        update_clocked_outputs();
        return Fmi3Status.OK;
    }

    public Fmi3Status fmi3Terminate() {
        this.state = FMIState.FMITerminatedState;
        return Fmi3Status.OK;
    }

    /* Serialization */

    public Fmi3SerializeFmuStatePair fmi3SerializeFmuState() throws Exception {

        var b = new ByteArrayOutputStream();
        var o = new ObjectOutputStream(b);

        o.writeObject(this);

        return new Fmi3SerializeFmuStatePair(Fmi3Status.OK, ByteBuffer.wrap(b.toByteArray()));
    }

    public Fmi3Status fmi3DeserializeFmuState(ByteBuffer bytes) throws Exception {
        byte[] state_byte_array = new byte[bytes.remaining()];
        bytes.get(state_byte_array);
        try (ByteArrayInputStream b = new ByteArrayInputStream(state_byte_array)) {
            try (ObjectInputStream o = new ObjectInputStream(b)) {
                var other = (Model) o.readObject();
                this.state = other.state;
                this.float32_a = other.float32_a;
                this.float32_b = other.float32_b;
                this.float32_c = other.float32_c;
                this.float64_a = other.float64_a;
                this.float64_b = other.float64_b;
                this.float64_c = other.float64_c;
                this.int8_a = other.int8_a;
                this.int8_b = other.int8_b;
                this.int8_c = other.int8_c;
                this.uint8_a = other.uint8_a; 
                this.uint8_b = other.uint8_b; 
                this.uint8_c = other.uint8_c; 
                this.int16_a = other.int16_a;
                this.int16_b = other.int16_b;
                this.int16_c = other.int16_c;
                this.uint16_a = other.uint16_a; 
                this.uint16_b = other.uint16_b; 
                this.uint16_c = other.uint16_c; 
                this.int32_a = other.int32_a;
                this.int32_b = other.int32_b;
                this.int32_c = other.int32_c;
                this.uint32_a = other.uint32_a; 
                this.uint32_b = other.uint32_b; 
                this.uint32_c = other.uint32_c; 
                this.int64_a = other.int64_a;
                this.int64_b = other.int64_b;
                this.int64_c = other.int64_c;
                this.uint64_a = other.uint64_a; 
                this.uint64_b = other.uint64_b; 
                this.uint64_c = other.uint64_c; 
                this.boolean_a = other.boolean_a;
                this.boolean_b = other.boolean_b;
                this.boolean_c = other.boolean_c;
                this.string_a = other.string_a;
                this.string_b = other.string_b;
                this.string_c = other.string_c;
                this.binary_a = other.binary_a;
                this.binary_b = other.binary_b;
                this.binary_c = other.binary_c;
                this.float32_tunable_parameter = other.float32_tunable_parameter;
                this.float64_tunable_parameter = other.float64_tunable_parameter;
                this.int8_tunable_parameter = other.int8_tunable_parameter;
                this.uint8_tunable_parameter = other.uint8_tunable_parameter;
                this.int16_tunable_parameter = other.int16_tunable_parameter;
                this.uint16_tunable_parameter = other.uint16_tunable_parameter;
                this.int32_tunable_parameter = other.int32_tunable_parameter;
                this.uint32_tunable_parameter = other.uint32_tunable_parameter;
                this.int64_tunable_parameter = other.int64_tunable_parameter;
                this.uint64_tunable_parameter = other.uint64_tunable_parameter;
                this.boolean_tunable_parameter = other.boolean_tunable_parameter;
                this.string_tunable_parameter = other.string_tunable_parameter;
                this.binary_tunable_parameter = other.binary_tunable_parameter;
                this.uint64_tunable_structural_parameter = other.uint64_tunable_structural_parameter;
                this.float32_vector_using_tunable_structural_parameter = other.float32_vector_using_tunable_structural_parameter;
                this.clock_a = other.clock_a;
                this.clock_b = other.clock_b;
                this.clock_c = other.clock_c;
                this.clocked_variable_a = other.clocked_variable_a;
                this.clocked_variable_b = other.clocked_variable_b;
                this.clocked_variable_c = other.clocked_variable_c;
                this.clock_reference_to_interval = other.clock_reference_to_interval;
                this.clock_reference_to_shift = other.clock_reference_to_shift;
                update_outputs();
                update_clocks();
                update_clocked_outputs();
            }
        }

        return Fmi3Status.OK;
    }

    /* Helpers */

    private <T> Fmi3GetValuePair<T> GetValue(Iterable<Integer> references) throws Exception {
        Fmi3Status status = Fmi3Status.OK;
        var values = new ArrayList<T>();

        for (var ref : references) {
            var field = this.map_to_attributes.get(ref);

            if (this.clocked_variables.contains(field)){
                if (!((this.state == FMIState.FMIEventModeState) || (this.state == FMIState.FMIInitializationModeState)))
                {
                    this.log(
                        String.format(
                            "Accessed clocked variable #%s# when neither in event mode nor in initialization mode.",
                            ref
                        ),
                        Fmi3Status.Warning,
                        "logStatusWarning"
                    );
                    status = Fmi3Status.Warning;
                }
            }

            var val = field.get(this);

            if (val instanceof byte[]) {
                val = ByteBuffer.wrap((byte[]) val);
            }

            if (val.getClass().isArray()) {
                values.addAll(Arrays.asList((T[]) val));
            } else {
                values.add((T) val);
            }
        }

        return new Fmi3GetValuePair<T>(status, values);
    }

    private <T> Fmi3Status SetValue(Iterable<Integer> references, Iterable<T> values) throws Exception {        
        Fmi3Status status = Fmi3Status.OK;

        Iterator<Integer> i1 = references.iterator();
        Iterator<T> i2 = values.iterator();

        while (i1.hasNext() && i2.hasNext()) {
            Integer r = i1.next();
            var field = (T) this.map_to_attributes.get(r);
            var v = i2.next();

            if (
                this.clocked_variables.contains(field)
                || this.tunable_parameters.contains(field)
            ) {
                if (
                    this.state != FMIState.FMIEventModeState
                    && this.state != FMIState.FMIInitializationModeState
                ) {
                    this.log(
                        String.format(
                            "Set clocked variable or tunable parameter #%s# when neither in event mode nor in initialization mode.",
                            r
                        ),
                        Fmi3Status.Warning,
                        "logStatusWarning"
                    );
                    status = Fmi3Status.Warning;
                }
            } else if (this.tunable_structural_parameters.contains(field)) {
                if (
                    this.state != FMIState.FMIConfigurationModeState
                    && this.state != FMIState.FMIReconfigurationModeState
                ) {
                    this.log(
                        String.format(
                            "Set tunable structural parameter #%s# when neither in configuration mode nor in reconfiguration mode.",
                            r
                        ),
                        Fmi3Status.Warning,
                        "logStatusWarning"
                    );
                    status = Fmi3Status.Warning;
                }
            } else if (this.parameters.contains(field)) {
                if (this.state != FMIState.FMIInitializationModeState) {
                    this.log(
                        String.format(
                            "Set parameter #%s# when not in initialization mode.",
                            r
                        ),
                        Fmi3Status.Warning,
                        "logStatusWarning"
                    );
                    status = Fmi3Status.Warning;
                }
            }
            
            if (v instanceof ByteBuffer) {
                ByteBuffer byte_buffer = (ByteBuffer) v;
                byte[] byte_array = new byte[byte_buffer.remaining()];
                byte_buffer.get(byte_array);
                this.map_to_attributes.get(r).set(this, byte_array);
            } else {
                this.map_to_attributes.get(r).set(this, v);
            }
        }
        
        return status;
    }

    private void update_outputs() {
        this.float32_c = this.float32_a + this.float32_b;
        this.float64_c = this.float64_a + this.float64_b;
        this.int8_c = (this.int8_a + this.int8_b);
        this.uint8_c = (this.uint8_a + this.uint8_b);
        this.int16_c = (this.int16_a + this.int16_b);
        this.uint16_c = this.uint16_a + this.uint16_b;
        this.int32_c = this.int32_a + this.int32_b;
        this.uint32_c = this.uint32_a + this.uint32_b;
        this.int64_c = this.int64_a + this.int64_b;
        this.uint64_c = this.uint64_a + this.uint64_b;
        this.boolean_c = this.boolean_a || this.boolean_b;
        this.string_c = this.string_a + this.string_b;
        int length = Math.min(
            this.binary_a.length, this.binary_b.length
        );
        byte[] binary_result = new byte[length];
        for (int i = 0; i < length; i++) {
            binary_result[i] = (byte) (this.binary_a[i] ^ this.binary_b[i]);
        }
        this.binary_c = binary_result;
    }

    private void update_clocks(){
        this.clock_c = this.clock_a && this.clock_b;
    }

    private void update_clocked_outputs(){
        this.clocked_variable_c += this.clocked_variable_a + this.clocked_variable_b;
    }

    class Fmi3DoStepResult<T> {
        Fmi3Status status;
        Boolean event_handling_needed;
        Boolean terminate_simulation;
        Boolean early_return;
        Double last_successful_time;

        Fmi3DoStepResult(Fmi3Status status, Boolean event_handling_needed, Boolean terminate_simulation, Boolean early_return, Double last_successful_time)

        {
            this.status = status;
            this.event_handling_needed = event_handling_needed;
            this.terminate_simulation = terminate_simulation;
            this.early_return = early_return;
            this.last_successful_time = last_successful_time;
        }
    }

    class Fmi3UpdateDiscreteStatesResult<T> {
        Fmi3Status status;
        Boolean discrete_states_need_update;
        Boolean terminate_simulation;
        Boolean nominals_continuous_states_changed;
        Boolean values_continuous_states_changed;
        Boolean next_event_time_defined;
        Double next_event_time;

        Fmi3UpdateDiscreteStatesResult(Fmi3Status status, Boolean discrete_states_need_update, Boolean terminate_simulation, Boolean nominals_continuous_states_changed, Boolean values_continuous_states_changed, Boolean next_event_time_defined, Double next_event_time)

        {
            this.status = status;
            this.discrete_states_need_update = discrete_states_need_update;
            this.terminate_simulation = terminate_simulation;
            this.nominals_continuous_states_changed = nominals_continuous_states_changed;
            this.values_continuous_states_changed = values_continuous_states_changed;
            this.next_event_time_defined = next_event_time_defined;
            this.next_event_time = next_event_time;
        }
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

    class Fmi3GetIntervalDecimalPair<T, U> {
        Fmi3Status status;
        List<T> intervals;
        List<U> qualifiers;

        Fmi3GetIntervalDecimalPair(Fmi3Status status, List<T> intervals, List<U> qualifiers)

        {
            this.status = status;
            this.intervals = intervals;
            this.qualifiers = qualifiers;
        }
    }

    class Fmi3GetIntervalFractionPair<T, U> {
        Fmi3Status status;
        List<T> counters;
        List<T> resolutions;
        List<U> qualifiers;

        Fmi3GetIntervalFractionPair(Fmi3Status status, List<T> counters, List<T> resolutions, List<U> qualifiers)

        {
            this.status = status;
            this.counters = counters;
            this.resolutions = resolutions;
            this.qualifiers = qualifiers;
        }
    }

    class Fmi3GetShiftDecimalPair<T> {
        Fmi3Status status;
        List<T> shifts;

        Fmi3GetShiftDecimalPair(Fmi3Status status, List<T> shifts)

        {
            this.status = status;
            this.shifts = shifts;
        }
    }

    class Fmi3GetShiftFractionPair<T> {
        Fmi3Status status;
        List<T> counters;
        List<T> resolutions;

        Fmi3GetShiftFractionPair(Fmi3Status status, List<T> counters, List<T> resolutions)

        {
            this.status = status;
            this.counters = counters;
            this.resolutions = resolutions;
        }
    }    

    class Fmi3SerializeFmuStatePair {
        public Fmi3Status status;
        public ByteBuffer bytes;

        Fmi3SerializeFmuStatePair(Fmi3Status status, ByteBuffer bytes) {
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

    class FMIState {
        public static final int FMIStartAndEndState         = 1 << 0;
        public static final int FMIInstantiatedState        = 1 << 1;
        public static final int FMIInitializationModeState  = 1 << 2;
        public static final int FMITerminatedState          = 1 << 3;
        public static final int FMIConfigurationModeState   = 1 << 4;
        public static final int FMIReconfigurationModeState = 1 << 5;
        public static final int FMIEventModeState           = 1 << 6;
        public static final int FMIContinuousTimeModeState  = 1 << 7;
        public static final int FMIStepModeState            = 1 << 8;
        public static final int FMIClockActivationMode      = 1 << 9;
    }

    class Fraction {
        private final Long numerator;
        private final Long denominator;
    
        public Fraction(String decimal) {
            BigDecimal bd = new BigDecimal(decimal);
            int scale = bd.scale();
            BigInteger den = BigInteger.TEN.pow(scale);
            BigInteger num = bd.multiply(new BigDecimal(den)).toBigIntegerExact();
    
            BigInteger gcd = num.gcd(den);
            this.numerator = num.divide(gcd).longValue();
            this.denominator = den.divide(gcd).longValue();
        }
    
        public Long getNumerator() {
            return numerator;
        }
    
        public Long getDenominator() {
            return denominator;
        }
    }

}
