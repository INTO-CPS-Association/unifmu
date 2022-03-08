using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;

using System.Linq;
using Fmi2Messages;

public class Model
{
    public double real_a { get; set; } = 0.0f;
    public double real_b { get; set; } = 0.0f;
    public double real_c { get; set; } = 0.0f;
    public int integer_a { get; set; } = 0;
    public int integer_b { get; set; } = 0;
    public int integer_c { get; set; } = 0;
    public bool boolean_a { get; set; } = false;
    public bool boolean_b { get; set; } = false;
    public bool boolean_c { get; set; } = false;
    public string string_a { get; set; } = "";
    public string string_b { get; set; } = "";
    public string string_c { get; set; } = "";

    private Dictionary<uint, PropertyInfo> reference_to_attributes = new Dictionary<uint, PropertyInfo>();


    public Model()
    {
        this.reference_to_attributes = new Dictionary<uint, PropertyInfo>
    {
      { 0, this.GetType().GetProperty("real_a") },
      { 1, this.GetType().GetProperty("real_b") },
      { 2, this.GetType().GetProperty("real_c") },
      { 3, this.GetType().GetProperty("integer_a") },
      { 4, this.GetType().GetProperty("integer_b") },
      { 5, this.GetType().GetProperty("integer_c") },
      { 6, this.GetType().GetProperty("boolean_a") },
      { 7, this.GetType().GetProperty("boolean_b") },
      { 8, this.GetType().GetProperty("boolean_c") },
      { 9, this.GetType().GetProperty("string_a") },
      { 10, this.GetType().GetProperty("string_b") },
      { 11, this.GetType().GetProperty("string_c") },
    };

    }
    public Fmi2Status Fmi2DoStep(double currentTime, double stepSize, bool noStepPrior)
    {
        UpdateOutputs();
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2SetupExperiment(double startTime, double? stopTime, double? tolerance)
    {
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2SetDebugLogging(IEnumerable<String> categories, bool loggingOn)
    {
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2EnterInitializationMode()
    {
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2ExitInitializationMode()
    {
        this.UpdateOutputs();
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status FmiSetReal(IEnumerable<uint> references, IEnumerable<double> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi2Status Fmi2SetInteger(IEnumerable<uint> references, IEnumerable<int> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi2Status Fmi2SetBoolean(IEnumerable<uint> references, IEnumerable<bool> values)
    {
        return this.SetValueReflection(references, values);
    }

    public Fmi2Status Fmi2SetString(IEnumerable<uint> references, IEnumerable<string> values)
    {
        return this.SetValueReflection(references, values);

    }

    public (Fmi2Status, IEnumerable<double>) Fmi2GetReal(IEnumerable<uint> references)
    {
        return this.GetValueReflection<double>(references);
    }

    public (Fmi2Status, IEnumerable<int>) Fmi2GetInteger(IEnumerable<uint> references)
    {
        return this.GetValueReflection<int>(references);
    }

    public (Fmi2Status, IEnumerable<bool>) Fmi2GetBoolean(IEnumerable<uint> references)
    {
        return this.GetValueReflection<bool>(references);
    }

    public (Fmi2Status, IEnumerable<String>) Fmi2GetString(IEnumerable<uint> references)
    {
        return this.GetValueReflection<String>(references);
    }

    public Fmi2Status Fmi2CancelStep()
    {
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2Reset()
    {
        return Fmi2Status.Fmi2Ok;
    }

    public Fmi2Status Fmi2Terminate()
    {
        return Fmi2Status.Fmi2Ok;
    }

    public (Fmi2Status, byte[]) Fmi2SerializeFmuState()
    {
        using (MemoryStream m = new MemoryStream())
        {
            using (BinaryWriter writer = new BinaryWriter(m))
            {
                writer.Write(real_a);
                writer.Write(real_b);
                writer.Write(real_c);
                writer.Write(integer_a);
                writer.Write(integer_b);
                writer.Write(integer_c);
                writer.Write(boolean_a);
                writer.Write(boolean_b);
                writer.Write(boolean_c);
                writer.Write(string_a);
                writer.Write(string_b);
                writer.Write(string_c);
            }
            return (Fmi2Status.Fmi2Ok, m.ToArray());
        }
    }

    public Fmi2Status Fmi2DeserializeFmuState(byte[] state)
    {
        using (MemoryStream m = new MemoryStream(state))
        {
            using (BinaryReader reader = new BinaryReader(m))
            {
                this.real_a = reader.ReadDouble();
                this.real_b = reader.ReadDouble();
                this.real_c = reader.ReadDouble();
                this.integer_a = reader.ReadInt32();
                this.integer_b = reader.ReadInt32();
                this.integer_c = reader.ReadInt32();
                this.boolean_a = reader.ReadBoolean();
                this.boolean_b = reader.ReadBoolean();
                this.boolean_c = reader.ReadBoolean();
                this.string_a = reader.ReadString();
                this.string_b = reader.ReadString();
                this.string_c = reader.ReadString();
            }
        }
        return Fmi2Status.Fmi2Ok;
    }

    private void UpdateOutputs()
    {
        this.real_c = real_a + real_b;
        this.integer_c = integer_a + integer_b;
        this.boolean_c = boolean_a || boolean_b;
        this.string_c = string_a + string_b;
    }

    private Fmi2Status SetValueReflection<T>(IEnumerable<uint> references, IEnumerable<T> values)
    {
        foreach (var (r, v) in references.Zip(values))
        {
            this.reference_to_attributes[r].SetValue(this, (object)v);
        }

        return Fmi2Status.Fmi2Ok;
    }

    private (Fmi2Status, IEnumerable<T>) GetValueReflection<T>(IEnumerable<uint> references)
    {

        var values = new List<T>(references.Count());

        foreach (var r in references)
        {
            values.Add((T)this.reference_to_attributes[r].GetValue(this));
        }

        return (Fmi2Status.Fmi2Ok, values);
    }


}