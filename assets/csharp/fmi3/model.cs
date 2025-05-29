using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Numerics;

using System.Linq;
using Fmi3Messages;

public class Model
{
    private string instance_name = "";
    private string instantiation_token = "";
    private string resource_path = "";
    private bool visible = false;
    private bool logging_on = false;
    private bool event_mode_used = false;
    private bool early_return_allowed = false;
    private List<uint> required_intermediate_variables;

    private FMIState state = FMIState.FMIInstantiatedState;
    public float float32_a { get; set; } = 0.0f;
    public float float32_b { get; set; } = 0.0f;
    public float float32_c { get; set; } = 0.0f;
    public double float64_a { get; set; } = 0.0;
    public double float64_b { get; set; } = 0.0;
    public double float64_c { get; set; } = 0.0;
    public int int8_a { get; set; } = 0;
    public int int8_b { get; set; } = 0;
    public int int8_c { get; set; } = 0;
    public uint uint8_a { get; set; } = 0; 
    public uint uint8_b { get; set; } = 0; 
    public uint uint8_c { get; set; } = 0; 
    public int int16_a { get; set; } = 0;
    public int int16_b { get; set; } = 0;
    public int int16_c { get; set; } = 0;
    public uint uint16_a { get; set; } = 0; 
    public uint uint16_b { get; set; } = 0; 
    public uint uint16_c { get; set; } = 0; 
    public int int32_a { get; set; } = 0;
    public int int32_b { get; set; } = 0;
    public int int32_c { get; set; } = 0;
    public uint uint32_a { get; set; } = 0; 
    public uint uint32_b { get; set; } = 0; 
    public uint uint32_c { get; set; } = 0; 
    public long int64_a { get; set; } = 0L;
    public long int64_b { get; set; } = 0L;
    public long int64_c { get; set; } = 0L;
    public ulong uint64_a { get; set; } = 0L; 
    public ulong uint64_b { get; set; } = 0L; 
    public ulong uint64_c { get; set; } = 0L; 
    public bool boolean_a { get; set; } = false;
    public bool boolean_b { get; set; } = false;
    public bool boolean_c { get; set; } = false;
    public string string_a { get; set; } = "";
    public string string_b { get; set; } = "";
    public string string_c { get; set; } = "";
    public byte[] binary_a { get; set; } = new byte[] {
        (byte) 0b00000000
    };
    public byte[] binary_b { get; set; } = new byte[] {
        (byte) 0b00000000
    };
    public byte[] binary_c { get; set; } = new byte[] {
        (byte) 0b00000000
    };

    public float float32_tunable_parameter { get; set; } = 0.0f;
    public double  float64_tunable_parameter { get; set; } = 0.0;
    public int int8_tunable_parameter { get; set; } = 0;
    public uint uint8_tunable_parameter { get; set; } = 0;
    public int int16_tunable_parameter { get; set; } = 0;
    public uint uint16_tunable_parameter { get; set; } = 0;
    public int int32_tunable_parameter { get; set; } = 0;
    public uint uint32_tunable_parameter { get; set; } = 0;
    public long int64_tunable_parameter { get; set; } = 0L;
    public ulong uint64_tunable_parameter { get; set; } = 0L;
    public bool boolean_tunable_parameter { get; set; } = false;
    public string string_tunable_parameter { get; set; } = "";
    public byte[] binary_tunable_parameter { get; set; } = new byte[] {
        (byte) 0b00000000
    };
    public ulong uint64_tunable_structural_parameter { get; set; } = 5L;
    public float[] float32_vector_using_tunable_structural_parameter { get; set; } = new float[] {
        0.1f,
        0.2f,
        0.3f,
        0.4f,
        0.5f
    };
    public bool clock_a { get; set; } = false;
    public bool clock_b { get; set; } = false;
    public bool clock_c { get; set; } = false;
    public int clocked_variable_a { get; set; } = 0;
    public int clocked_variable_b { get; set; } = 0;
    public int clocked_variable_c { get; set; } = 0;
    public Dictionary<uint,double> clock_reference_to_interval = new Dictionary<uint,double>{{1001, 1.0}};
    public Dictionary<uint,double> clock_reference_to_shift = new Dictionary<uint,double>{{1001, 1.0}};


    private Dictionary<uint, PropertyInfo> reference_to_attributes = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> all_references = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> clocked_variables = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> parameters = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> tunable_parameters = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> tunable_structural_parameters = new Dictionary<uint, PropertyInfo>();

    private Dictionary<uint, PropertyInfo> all_parameters = new Dictionary<uint, PropertyInfo>();

    public Model(string instance_name, string instantiation_token, string resource_path, bool visible, bool logging_on, bool event_mode_used, bool early_return_allowed, List<uint> required_intermediate_variables)
    {
        this.instance_name = instance_name;
        this.instantiation_token = instantiation_token;
        this.resource_path = resource_path;
        this.visible = visible;
        this.logging_on = logging_on;
        this.event_mode_used = false;
        this.early_return_allowed = early_return_allowed;
        this.required_intermediate_variables = required_intermediate_variables;
        var type = this.GetType();

        this.reference_to_attributes = new Dictionary<uint, PropertyInfo>
        {
            { 0, type.GetProperty("float32_a") },
            { 1, type.GetProperty("float32_b") },
            { 2, type.GetProperty("float32_c") },
            { 3, type.GetProperty("float64_a") },
            { 4, type.GetProperty("float64_b") },
            { 5, type.GetProperty("float64_c") },
            { 6, type.GetProperty("int8_a") },
            { 7, type.GetProperty("int8_b") },
            { 8, type.GetProperty("int8_c") },
            { 9, type.GetProperty("uint8_a") },
            { 10, type.GetProperty("uint8_b") },
            { 11, type.GetProperty("uint8_c") },
            { 12, type.GetProperty("int16_a") },
            { 13, type.GetProperty("int16_b") },
            { 14, type.GetProperty("int16_c") },
            { 15, type.GetProperty("uint16_a") },
            { 16, type.GetProperty("uint16_b") },
            { 17, type.GetProperty("uint16_c") },
            { 18, type.GetProperty("int32_a") },
            { 19, type.GetProperty("int32_b") },
            { 20, type.GetProperty("int32_c") },
            { 21, type.GetProperty("uint32_a") },
            { 22, type.GetProperty("uint32_b") },
            { 23, type.GetProperty("uint32_c") },
            { 24, type.GetProperty("int64_a") },
            { 25, type.GetProperty("int64_b") },
            { 26, type.GetProperty("int64_c") },
            { 27, type.GetProperty("uint64_a") },
            { 28, type.GetProperty("uint64_b") },
            { 29, type.GetProperty("uint64_c") },
            { 30, type.GetProperty("boolean_a") },
            { 31, type.GetProperty("boolean_b") },
            { 32, type.GetProperty("boolean_c") },
            { 33, type.GetProperty("string_a") },
            { 34, type.GetProperty("string_b") },
            { 35, type.GetProperty("string_c") },
            { 36, type.GetProperty("binary_a") },
            { 37, type.GetProperty("binary_b") },
            { 38, type.GetProperty("binary_c") },
        };

        this.clocked_variables = new Dictionary<uint, PropertyInfo>
        {
            { 1001, type.GetProperty("clock_a") },
            { 1002, type.GetProperty("clock_b") },
            { 1003, type.GetProperty("clock_c") },
            { 1100, type.GetProperty("clocked_variable_a") },
            { 1101, type.GetProperty("clocked_variable_b") },
            { 1102, type.GetProperty("clocked_variable_c") },
        };

        this.parameters = new Dictionary<uint, PropertyInfo>();

        this.tunable_parameters = new Dictionary<uint, PropertyInfo>
        {
            { 100, type.GetProperty("float32_tunable_parameter") },
            { 101, type.GetProperty("float64_tunable_parameter") },
            { 102, type.GetProperty("int8_tunable_parameter") },
            { 103, type.GetProperty("uint8_tunable_parameter") },
            { 104, type.GetProperty("int16_tunable_parameter") },
            { 105, type.GetProperty("uint16_tunable_parameter") },
            { 106, type.GetProperty("int32_tunable_parameter") },
            { 107, type.GetProperty("uint32_tunable_parameter") },
            { 108, type.GetProperty("int64_tunable_parameter") },
            { 109, type.GetProperty("uint64_tunable_parameter") },
            { 110, type.GetProperty("boolean_tunable_parameter") },
            { 111, type.GetProperty("string_tunable_parameter") },
            { 112, type.GetProperty("binary_tunable_parameter") },
            { 113, type.GetProperty("uint64_tunable_structural_parameter") },
        };

        this.tunable_structural_parameters = new Dictionary<uint, PropertyInfo>
        {
            { 114, type.GetProperty("float32_vector_using_tunable_structural_parameter") },
        };

        this.all_references = new Dictionary<uint, PropertyInfo>();
        foreach (var kv in this.tunable_structural_parameters) all_references[kv.Key] = kv.Value;
        foreach (var kv in this.parameters) all_references[kv.Key] = kv.Value;
        foreach (var kv in this.tunable_parameters) all_references[kv.Key] = kv.Value;
        foreach (var kv in this.clocked_variables) all_references[kv.Key] = kv.Value;
        foreach (var kv in this.reference_to_attributes) all_references[kv.Key] = kv.Value;

        this.all_parameters = new Dictionary<uint, PropertyInfo>();
        foreach (var kv in this.tunable_structural_parameters) all_parameters[kv.Key] = kv.Value;
        foreach (var kv in this.parameters) all_parameters[kv.Key] = kv.Value;
        foreach (var kv in this.tunable_parameters) all_parameters[kv.Key] = kv.Value;

        UpdateOutputs();
        UpdateClocks();
        UpdateClockedOutputs();

    }

    /* doStep and updateDiscreteStates */

    public (Fmi3Status, bool, bool, bool, double) Fmi3DoStep(double currentCommunicationPoint, double communicationStepSize, bool noStepPrior)
    {
        UpdateOutputs();
        bool event_handling_needed = false;
        bool terminate_simulation = false;
        bool early_return = false;
        double last_successful_time = currentCommunicationPoint + communicationStepSize;
        return (Fmi3Status.Fmi3Ok,event_handling_needed,terminate_simulation,early_return,last_successful_time);
    }

    public (Fmi3Status, bool, bool, bool, bool, bool, double) Fmi3UpdateDiscreteStates(){
        bool discrete_states_need_update = false;
        bool terminate_simulation = false;
        bool nominals_continuous_states_changed = false;
        bool values_continuous_states_changed = false;
        bool next_event_time_defined = true;
        double next_event_time = 1.0;

        UpdateClockedOutputs();

        return (Fmi3Status.Fmi3Ok, discrete_states_need_update, terminate_simulation, nominals_continuous_states_changed, values_continuous_states_changed, next_event_time_defined, next_event_time);
    }

    /* Initialization, Enter, Termination, and Reset */

    public Fmi3Status Fmi3EnterInitializationMode()
    {
        this.state = FMIState.FMIEventModeState;
        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3ExitInitializationMode()
    {
        if (this.event_mode_used) {
            this.state = FMIState.FMIEventModeState;
        } else{
            this.state = FMIState.FMIStepModeState;
        }
        this.UpdateOutputs();
        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3EnterEventMode(){
        this.state = FMIState.FMIEventModeState;
        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3EnterStepMode(){
        this.state = FMIState.FMIStepModeState;
        return Fmi3Status.Fmi3Ok;
    }
    
    public Fmi3Status Fmi3EnterConfigurationMode(){
        if (this.state == FMIState.FMIInstantiatedState){
            this.state = FMIState.FMIConfigurationModeState;
        } else{
            this.state = FMIState.FMIReconfigurationModeState;
        }
        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3ExitConfigurationMode(){
        if (this.state == FMIState.FMIConfigurationModeState){
            this.state = FMIState.FMIInstantiatedState;
        }            
        else if (this.state == FMIState.FMIReconfigurationModeState){
            this.state = FMIState.FMIStepModeState;
        }            
        else{
            return Fmi3Status.Fmi3Error;
        }
        return Fmi3Status.Fmi3Ok;   
    }

    public Fmi3Status Fmi3Reset()
    {
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
        this.float32_vector_using_tunable_structural_parameter = new float[] {
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
        this.clock_reference_to_interval = new Dictionary<uint,double>{{1001, 1.0}};
        this.clock_reference_to_shift = new Dictionary<uint,double>{{1001, 1.0}};
        UpdateOutputs();
        UpdateClocks();
        UpdateClockedOutputs();
        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3Terminate()
    {
        this.state = FMIState.FMITerminatedState;
        return Fmi3Status.Fmi3Ok;
    }

    /* Setters */

    public Fmi3Status Fmi3SetFloat32(IEnumerable<uint> references, IEnumerable<float> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetFloat64(IEnumerable<uint> references, IEnumerable<double> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetInt8(IEnumerable<uint> references, IEnumerable<int> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetUInt8(IEnumerable<uint> references, IEnumerable<uint> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetInt16(IEnumerable<uint> references, IEnumerable<int> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetUInt16(IEnumerable<uint> references, IEnumerable<uint> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetInt32(IEnumerable<uint> references, IEnumerable<int> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetUInt32(IEnumerable<uint> references, IEnumerable<uint> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetInt64(IEnumerable<uint> references, IEnumerable<long> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetUInt64(IEnumerable<uint> references, IEnumerable<ulong> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetBinary(IEnumerable<uint> references, IEnumerable<ulong> valueSizes, IEnumerable<byte[]> values)
    {
        // You can store valueSizes if needed
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetClock(IEnumerable<uint> references, IEnumerable<bool> values)
    {
        var status = this.SetValueReflection(references, values);
        this.UpdateClocks();
        return status;
    }

    public Fmi3Status Fmi3SetBoolean(IEnumerable<uint> references, IEnumerable<bool> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi3Status Fmi3SetString(IEnumerable<uint> references, IEnumerable<string> values)
    {
        return this.SetValueReflection(references, values);

    }

    public Fmi3Status Fmi3SetIntervalDecimal(IEnumerable<uint> references, IEnumerable<double> intervals)
    {
        using (var referenceEnumerator = references.GetEnumerator())
        using (var intervalEnumerator = intervals.GetEnumerator())
        {
            while (referenceEnumerator.MoveNext() && intervalEnumerator.MoveNext())
            {
                clock_reference_to_interval[referenceEnumerator.Current] = intervalEnumerator.Current;
            }
        }

        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3SetIntervalFraction(IEnumerable<uint> references, IEnumerable<ulong> counters, IEnumerable<ulong> resolutions)
    {
        using (var referenceEnumerator = references.GetEnumerator())
        using (var counterEnumerator = counters.GetEnumerator())
        using (var resolutionEnumerator = resolutions.GetEnumerator())
        {
            while (referenceEnumerator.MoveNext() && counterEnumerator.MoveNext() && resolutionEnumerator.MoveNext())
            {
                var interval = (double)counterEnumerator.Current / resolutionEnumerator.Current;
                clock_reference_to_interval[referenceEnumerator.Current] = interval;
            }
        }

        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3SetShiftDecimal(IEnumerable<uint> references, IEnumerable<double> shifts)
    {
        using (var referenceEnumerator = references.GetEnumerator())
        using (var shiftEnumerator = shifts.GetEnumerator())
        {
            while (referenceEnumerator.MoveNext() && shiftEnumerator.MoveNext())
            {
                clock_reference_to_shift[referenceEnumerator.Current] = shiftEnumerator.Current;
            }
        }

        return Fmi3Status.Fmi3Ok;
    }

    public Fmi3Status Fmi3SetShiftFraction(IEnumerable<uint> references, IEnumerable<ulong> counters, IEnumerable<ulong> resolutions)
    {
        using (var referenceEnumerator = references.GetEnumerator())
        using (var counterEnumerator = counters.GetEnumerator())
        using (var resolutionEnumerator = resolutions.GetEnumerator())
        {
            while (referenceEnumerator.MoveNext() && counterEnumerator.MoveNext() && resolutionEnumerator.MoveNext())
            {
                var shift = (double)counterEnumerator.Current / resolutionEnumerator.Current;
                clock_reference_to_shift[referenceEnumerator.Current] = shift;
            }
        }

        return Fmi3Status.Fmi3Ok;
    }

    /* Getters */

    public (Fmi3Status, IEnumerable<float>) Fmi3GetFloat32(IEnumerable<uint> references)
    {
        return this.GetValueReflection<float>(references);
    }

    public (Fmi3Status, IEnumerable<double>) Fmi3GetFloat64(IEnumerable<uint> references)
    {
        return this.GetValueReflection<double>(references);
    }

    public (Fmi3Status, IEnumerable<int>) Fmi3GetInt8(IEnumerable<uint> references)
    {
        return this.GetValueReflection<int>(references);
    }

    public (Fmi3Status, IEnumerable<uint>) Fmi3GetUInt8(IEnumerable<uint> references)
    {
        return this.GetValueReflection<uint>(references);
    }

    public (Fmi3Status, IEnumerable<int>) Fmi3GetInt16(IEnumerable<uint> references)
    {
        return this.GetValueReflection<int>(references);
    }

    public (Fmi3Status, IEnumerable<uint>) Fmi3GetUInt16(IEnumerable<uint> references)
    {
        return this.GetValueReflection<uint>(references);
    }

    public (Fmi3Status, IEnumerable<int>) Fmi3GetInt32(IEnumerable<uint> references)
    {
        return this.GetValueReflection<int>(references);
    }

    public (Fmi3Status, IEnumerable<uint>) Fmi3GetUInt32(IEnumerable<uint> references)
    {
        return this.GetValueReflection<uint>(references);
    }

    public (Fmi3Status, IEnumerable<long>) Fmi3GetInt64(IEnumerable<uint> references)
    {
        return this.GetValueReflection<long>(references);
    }

    public (Fmi3Status, IEnumerable<ulong>) Fmi3GetUInt64(IEnumerable<uint> references)
    {
        return this.GetValueReflection<ulong>(references);
    }

    public (Fmi3Status, IEnumerable<byte[]>) Fmi3GetBinary(IEnumerable<uint> references)
    {
        return this.GetValueReflection<byte[]>(references);
    }

    public (Fmi3Status, IEnumerable<bool>) Fmi3GetClock(IEnumerable<uint> references)
    {
        return this.GetValueReflection<bool>(references);
    }

    public (Fmi3Status, IEnumerable<bool>) Fmi3GetBoolean(IEnumerable<uint> references)
    {
        return this.GetValueReflection<bool>(references);
    }

    public (Fmi3Status, IEnumerable<string>) Fmi3GetString(IEnumerable<uint> references)
    {
        return this.GetValueReflection<string>(references);
    }

    public (Fmi3Status, IEnumerable<double>, IEnumerable<int>) Fmi3GetIntervalDecimal(IEnumerable<uint> references)
    {
        var intervals = new List<double>();
        var qualifiers = new List<int>();

        foreach (var reference in references)
        {
            intervals.Add(clock_reference_to_interval[reference]);
            qualifiers.Add(2);
        }

        return (Fmi3Status.Fmi3Ok, intervals, qualifiers);
    }

    public (Fmi3Status, IEnumerable<ulong>, IEnumerable<ulong>, IEnumerable<int>) Fmi3GetIntervalFraction(IEnumerable<uint> references)
    {
        var counters = new List<ulong>();
        var resolutions = new List<ulong>();
        var qualifiers = new List<int>();

        foreach (var reference in references)
        {
            var decimalValue = clock_reference_to_interval[reference].ToString();
            var fraction = new Fraction(decimalValue);
            counters.Add(fraction.Numerator);
            resolutions.Add(fraction.Denominator);
            qualifiers.Add(2);
        }

        return (Fmi3Status.Fmi3Ok, counters, resolutions, qualifiers);
    }

    public (Fmi3Status, IEnumerable<double>) Fmi3GetShiftDecimal(IEnumerable<uint> references)
    {
        var shifts = new List<double>();

        foreach (var reference in references)
        {
            shifts.Add(clock_reference_to_shift[reference]);
        }

        return (Fmi3Status.Fmi3Ok, shifts);
    }

    public (Fmi3Status, IEnumerable<ulong>, IEnumerable<ulong>) Fmi3GetShiftFraction(IEnumerable<uint> references)
    {
        var counters = new List<ulong>();
        var resolutions = new List<ulong>();

        foreach (var reference in references)
        {
            var decimalValue = clock_reference_to_shift[reference].ToString();
            var fraction = new Fraction(decimalValue);
            counters.Add(fraction.Numerator);
            resolutions.Add(fraction.Denominator);
        }

        return (Fmi3Status.Fmi3Ok, counters, resolutions);
    }

    /* Serialization */

    public (Fmi3Status, byte[]) Fmi3SerializeFmuState()
    {
        using (MemoryStream m = new MemoryStream())
        {
            using (BinaryWriter writer = new BinaryWriter(m))
            {
                foreach (var entry in this.all_references.OrderBy(e => e.Key))
                {
                    var prop = entry.Value;
                    var value = prop.GetValue(this);

                    switch (value)
                    {
                        case float f: writer.Write(f); break;
                        case double d: writer.Write(d); break;
                        case sbyte sb: writer.Write(sb); break;
                        case byte b: writer.Write(b); break;
                        case short s: writer.Write(s); break;
                        case ushort us: writer.Write(us); break;
                        case int i: writer.Write(i); break;
                        case uint ui: writer.Write(ui); break;
                        case long l: writer.Write(l); break;
                        case ulong ul: writer.Write(ul); break;
                        case bool bo: writer.Write(bo); break;
                        case string str: writer.Write(str ?? ""); break;
                        case byte[] arr:
                            writer.Write(arr.Length);
                            writer.Write(arr);
                            break;
                        case float[] arr:
                            writer.Write(arr.Length);
                            foreach (float f in arr)
                                writer.Write(f);
                            break;
                        default:
                            throw new InvalidOperationException($"Unsupported type for property {prop.Name}");
                    }
                }
            }
            return (Fmi3Status.Fmi3Ok, m.ToArray());
        }
    }

    public Fmi3Status Fmi3DeserializeFmuState(byte[] state)
    {
        using (MemoryStream m = new MemoryStream(state))
        {
            using (BinaryReader reader = new BinaryReader(m))
            {
                foreach (var entry in this.all_references.OrderBy(e => e.Key))
                {
                    var prop = entry.Value;
                    var type = prop.PropertyType;

                    if (type == typeof(float)) prop.SetValue(this, reader.ReadSingle());
                    else if (type == typeof(double)) prop.SetValue(this, reader.ReadDouble());
                    else if (type == typeof(sbyte)) prop.SetValue(this, reader.ReadSByte());
                    else if (type == typeof(byte)) prop.SetValue(this, reader.ReadByte());
                    else if (type == typeof(short)) prop.SetValue(this, reader.ReadInt16());
                    else if (type == typeof(ushort)) prop.SetValue(this, reader.ReadUInt16());
                    else if (type == typeof(int)) prop.SetValue(this, reader.ReadInt32());
                    else if (type == typeof(uint)) prop.SetValue(this, reader.ReadUInt32());
                    else if (type == typeof(long)) prop.SetValue(this, reader.ReadInt64());
                    else if (type == typeof(ulong)) prop.SetValue(this, reader.ReadUInt64());
                    else if (type == typeof(bool)) prop.SetValue(this, reader.ReadBoolean());
                    else if (type == typeof(string)) prop.SetValue(this, reader.ReadString());
                    else if (type == typeof(byte[]))
                    {
                        int len = reader.ReadInt32();
                        byte[] data = reader.ReadBytes(len);
                        prop.SetValue(this, data);
                    }
                    else if (type == typeof(float[]))
                    {
                        int len = reader.ReadInt32();
                        float[] data = new float[len];
                        for (int i = 0; i < len; i++)
                            data[i] = reader.ReadSingle();
                        prop.SetValue(this, data);
                    }
                    else
                    {
                        throw new InvalidOperationException($"Unsupported type for property {prop.Name}");
                    }
                }
            }
        }
        return Fmi3Status.Fmi3Ok;
    }

    private void UpdateOutputs()
    {
        this.float32_c = this.float32_a + this.float32_b;
        this.float64_c = this.float64_a + this.float64_b;
        this.int8_c = (this.int8_a + this.int8_b);
        this.uint8_c = (this.uint8_a + this.uint8_b);
        this.int16_c = (this.int16_a + this.int16_b);
        this.uint16_c = (this.uint16_a + this.uint16_b);
        this.int32_c = this.int32_a + this.int32_b;
        this.uint32_c = this.uint32_a + this.uint32_b;
        this.int64_c = this.int64_a + this.int64_b;
        this.uint64_c = this.uint64_a + this.uint64_b;
        this.boolean_c = this.boolean_a || this.boolean_b;
        this.string_c = this.string_a + this.string_b;

        int length = Math.Min(this.binary_a.Length, this.binary_b.Length);
        byte[] result = new byte[length];
        for (int i = 0; i < length; i++)
        {
            result[i] = (byte)(this.binary_a[i] ^ this.binary_b[i]);
        }
        this.binary_c = result;
    }

    private void UpdateClocks()
    {
        this.clock_c = this.clock_a && this.clock_b;
    }

    private void UpdateClockedOutputs()
    {
        this.clocked_variable_c += this.clocked_variable_a + this.clocked_variable_b;
    }

    public Fmi3Status SetValueReflection<T>(IEnumerable<uint> references, IEnumerable<T> values)
    {
        if ((state == FMIState.FMIConfigurationModeState) || (state == FMIState.FMIReconfigurationModeState))
        {
            foreach (var r in references)
            {
                if (clocked_variables.ContainsKey(r) || reference_to_attributes.ContainsKey(r))
                    return Fmi3Status.Fmi3Error;
            }
        }
        else if (state == FMIState.FMIEventModeState)
        {
            foreach (var r in references)
            {
                if (reference_to_attributes.ContainsKey(r) || tunable_structural_parameters.ContainsKey(r))
                    return Fmi3Status.Fmi3Error;
            }
        }
        else if (state == FMIState.FMIInitializationModeState)
        {
            
        }
        else{
            foreach (var r in references)
            {
                if ((event_mode_used && all_parameters.ContainsKey(r)) || clocked_variables.ContainsKey(r))
                    return Fmi3Status.Fmi3Error;
            }
        }

        foreach (var (r, v) in references.Zip(values))
        {
            this.all_references[r].SetValue(this, (object)v);
        }

        return Fmi3Status.Fmi3Ok;
    }    

    public (Fmi3Status, IEnumerable<T>) GetValueReflection<T>(IEnumerable<uint> references)
    {
        foreach (var r in references)
        {
            if (clocked_variables.ContainsKey(r))
            {
                if (!(state.HasFlag(FMIState.FMIEventModeState) || state.HasFlag(FMIState.FMIInitializationModeState)))
                    return (Fmi3Status.Fmi3Error, null);
            }
        }

        var values = new List<T>(references.Count());

        foreach (var r in references)
        {
            values.Add((T)this.all_references[r].GetValue(this));
        }

        return (Fmi3Status.Fmi3Ok, values);
    }

    [Flags]
    public enum FMIState
    {
        FMIStartAndEndState         = 1 << 0,
        FMIInstantiatedState        = 1 << 1,
        FMIInitializationModeState  = 1 << 2,
        FMITerminatedState          = 1 << 3,
        FMIConfigurationModeState   = 1 << 4,
        FMIReconfigurationModeState = 1 << 5,
        FMIEventModeState           = 1 << 6,
        FMIContinuousTimeModeState  = 1 << 7,
        FMIStepModeState            = 1 << 8,
        FMIClockActivationMode      = 1 << 9
    }
    
    public class Fraction
    {
        public ulong Numerator { get; }
        public ulong Denominator { get; }

        public Fraction(string decimalValue)
        {
            decimal decimalNumber = Decimal.Parse(decimalValue);
            int scale = GetScale(decimalValue);
            
            BigInteger den = BigInteger.Pow(10, scale);
            BigInteger num = (BigInteger)(decimalNumber * (decimal)den);

            BigInteger gcd = BigInteger.GreatestCommonDivisor(num, den);
            this.Numerator = (ulong)(num / gcd);
            this.Denominator = (ulong)(den / gcd);
        }

        private int GetScale(string decimalValue)
        {
            int scale = 0;
            int index = decimalValue.IndexOf('.');
            if (index >= 0)
            {
                scale = decimalValue.Length - index - 1;
            }
            return scale;
        }
        
        public override string ToString()
        {
            return $"{Numerator}/{Denominator}";
        }
    }

}