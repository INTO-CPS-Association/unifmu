using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;

using System.Linq;
using Fmi2Messages;

public delegate void LogCallback(Fmi2Status status, String category, String message);

public class Model
{
    public double real_a { get; set; }
    public double real_b { get; set; }
    public double real_c { get; set; }
    public int integer_a { get; set; }
    public int integer_b { get; set; }
    public int integer_c { get; set; }
    public bool boolean_a { get; set; }
    public bool boolean_b { get; set; }
    public bool boolean_c { get; set; }
    public string string_a { get; set; }
    public string string_b { get; set; }
    public string string_c { get; set; }

    private Dictionary<uint, PropertyInfo> reference_to_attributes = new Dictionary<uint, PropertyInfo>();
    private LogCallback log_callback { get; set; }


    public Model(LogCallback log_callback)
    {
        this.log_callback = log_callback;

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

    Fmi2Reset();
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

    public Fmi2Status Fmi2SetReal(IEnumerable<uint> references, IEnumerable<double> values)
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
        this.real_a = 0;
        this.real_b = 0;
        this.integer_a = 0;
        this.integer_b = 0;
        this.boolean_a = false;
        this.boolean_b = false;
        this.string_a = "";
        this.string_b = "";
        this.UpdateOutputs();

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

    /// <summary>UniFMU logging function
    /// <para>
    /// Call this function whenever something should be logged.
    /// This will send a message thourgh the UniFMU layer to
    /// the importer if the importer has enabled logging and
    /// is interested in the given logging category.
    /// </para>
    /// </summary>
    /// <remarks>
    /// <para>
    /// Both status and category have default values, but they
    /// should be explicitly set in most cases.
    /// </para>
    /// <para>
    /// The status should always reflect the *expected* return
    /// value of the current operation. For example if the
    /// current operation is progressing as it should and the
    /// call to Log() is notifying of normal operation then the
    /// status parameter should be set to Fmi2Status.Fmi2Ok as
    /// we would currently expect the operation to return this
    /// state. As a further example, if the call to Log() is
    /// made when an error is encountered then we would expect
    /// to return an Fmi2Status.Fmi2Error to the importer, and
    /// therefore the state parameter should also be set to
    /// Fmi2Status.Fmi2Error.
    /// </para>
    /// <para>
    /// The value of the category parameter is used by the
    /// UniFMU API layer to decide whether or not to actually
    /// send the message to the FMU importer. The importer
    /// can designate which categories that is interested in
    /// and if it does so, the UniFMU API layer only forwards
    /// messages with such interesting categories.
    /// The categories can be any string, and must be defined
    /// in modelDescription.xml to be valid an visible to the
    /// importer. A number of categories are predefined by
    /// the FMI2 standard and included in the 
    /// modelDescription.xml by default:
    /// <list type="bullet">
    /// <item>
    /// <description>logStatusWarning</description>
    /// </item>
    /// <item>
    /// <description>logStatusDiscard</description>
    /// </item>
    /// <item>
    /// <description>logStatusError</description>
    /// </item>
    /// <item>
    /// <description>logStatusFatal</description>
    /// </item>
    /// <item>
    /// <description>logStatusPending</description>
    /// </item>
    /// <item>
    /// <description>logAll</description>
    /// </item>
    /// </list>
    /// The importer calls Fmi2SetDebugLogging() to specify
    /// which categories it is interested in and whether or not
    /// logging should even be enabled. You can use this to
    /// expand the Log() function to pre-filter the messages
    /// if you want to reduce the amount of messages exchanged
    /// between this model and the UniFMU API layer. (It is
    /// advised to read the FMI2 standard, specifically the
    /// sections on logging beforehand.)
    /// </para>
    /// </remarks>
    /// <param name="message">the message to be logged
    /// </param>
    /// <param name="status">the Fmi2Status that the FMU expects to next return (default: Fmi2Status.Fmi2Ok)</param>
    /// <param name="category">the logging category that this log event falls under</param>
    private void Log(String message, Fmi2Status status = Fmi2Status.Fmi2Ok, String category = "logAll") {
        this.log_callback(status, category, message);
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