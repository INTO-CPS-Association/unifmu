using System.Collections.Generic;
using System.Reflection;
using System;
using System.IO;
using System.Linq;

public enum Fmi2Status : ushort
{
    /// Represents the status of the FMU or the results of function calls.
    /// Values:
    ///    * ok: all well
    ///     * warning: an issue has arisen, but the computation can continue.
    ///     * discard: an operation has resulted in invalid output, which must be discarded
    ///     * error: an error has ocurred for this specific FMU instance.
    ///     * fatal: an fatal error has ocurred which has corrupted ALL FMU instances.
    ///     * pending: indicates that the FMu is doing work asynchronously, which can be retrived later.
    /// Notes:
    ///     FMI section 2.1.3

    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
    Pending = 5,
}

/// <summary> Enum <c>Fmi2StatusKind</c>
/// Defines the different types of statuses the master can inquire the slave about, see p.104
/// These are used for async related functionality of FMI2.
/// Values:
///     * do_step_status: request the status of the step function. If not completed fmi2Pending is returned,
///                     if complete the status of the step function is returned. 
///     * pending_status: request a string description of describing the progress of the step function.
///     * last_successfull_time: returns the simulation time of the last successfull simulation step.
///     * terminated: ask the slave if it wants to terminate the simulation. This can be called after the 
///                    step function returns the discard status.
/// </summary>
public enum Fmi2StatusKind : ushort
{
    do_step_status = 0,
    pending_status = 1,
    last_successfull_time = 2,
    terminated = 3
}


/// <summary> Class <c>Fmi2FMU</c>
/// Base class for FMUs implemented using UniFMU's Python backend.
/// Deriving from this class provides dummy implementation for FMI2 function, 
/// eliminating the need to implement functionality not needed by the FMU.
/// An additional ulility of the base class is to provide function-prototypes
/// which the an IDE may use to provide code completion hints to the author.
/// The behavior of the FMU can be implemented by overwriting these methods.
/// </summary>
public abstract class Fmi2FMU
{
    protected Dictionary<uint, string> referenceToAttr;
    public StreamWriter sw;
    public Fmi2FMU(Dictionary<uint, string> referenceToAttr)
    {
        this.referenceToAttr = referenceToAttr;
        // Set up logging
        StreamWriter sw = new StreamWriter(Console.OpenStandardOutput());
        sw.AutoFlush = true;
        Console.SetOut(sw);
        this.sw = sw;
    }

    protected void SetPropertyValue(uint valueReference, object value)
    {
        try{
            this.GetType().GetProperty(referenceToAttr[valueReference]).SetValue(this, value);
        }
        catch {
            throw new ArgumentException("Could not set value of property with value reference {0}", valueReference.ToString());
        }
    }

    protected object GetPropertyValue(uint valueReference)
    {
        try {
            return this.GetType().GetProperty(referenceToAttr[valueReference]).GetValue(this, null);
        }
        catch{
            throw new ArgumentException("Could not get value of property with value reference {0}", valueReference.ToString());
        }
        
    }


    /********************************************************* COMMON **********************************************************/
    public virtual Fmi2Status SetDebugLogging(IEnumerable<string> categories, bool loggingOn)
    {
        // The categories parameter may potentially be a string array
        return Fmi2Status.Ok;
    }

    public virtual Fmi2Status SetupExperiment(double startTime, double? stopTime = null, double? tolerance = null)
    {
        return Fmi2Status.Ok;
    }


    /// <summary>
    /// Informs the FMU to enter initialization mode. 
    /// Before this all inputs with 'initial ∈ {exact, approx}', have been set by the tool.
    /// At this stage all outputs of 'initial ∈ {calculated}' can be assigned.
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status EnterInitializationMode()
    {
        return Fmi2Status.Ok;
    }
    /// <summary>
    /// Informs the fmu to exit initialziation mode.
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status ExitInitializationMode()
    {
        return Fmi2Status.Ok;
    }

    /// <summary>
    /// Informs the FMU that the simulation has finished, after this the final values of the FMU can be enquired by the tool.
    /// Note that termination is not the same as the FMU be freed; the fmu may be reset and used for another simulation run.
    /// As such it may be sensible to preserve expensive to construct resources, that would otherwise have to be recreated.
    /// If you need to add destructor like functionality, instead overwrite the objects __del__ method, which is invOked when the 
    /// FMU is finally dropped.
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status Terminate()
    {
        return Fmi2Status.Ok;
    }

    /// <summary>
    /// Restores the FMU to the same state as it would be after instantiation
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status Reset()
    {
        return Fmi2Status.Ok;
    }

    /********************************************************* Getters and Setters implemented **********************************************************/
    public virtual (Fmi2Status, IEnumerable<double>) GetReal(IEnumerable<uint> valueReferences)
    {
        try {
            IList<double> values = new List<double>(){};
            int i = 0;
            foreach (uint vref in valueReferences)
            {
                values.Add((double)GetPropertyValue(vref));
                sw.WriteLine("At vref: {0}", vref);
                sw.WriteLine("Got value: {0}", values[i++]);
            }
            sw.WriteLine("Got values: {0}", values);
            return (Fmi2Status.Ok,values);
        }
        catch{
            this.sw.WriteLine("Unable to read variable from slave, the variable is not declared as an attribute of the C# object");
            return (Fmi2Status.Error, null);
        }
    }
    public virtual (Fmi2Status, IEnumerable<int>) GetInt(IEnumerable<uint> valueReferences)
    {
        try {
            IList<int> values = new List<int>(){};
            foreach (uint vref in valueReferences)
            {
                values.Add((int)GetPropertyValue(vref));
            }
            return (Fmi2Status.Ok,values);
        }
        catch{
            this.sw.WriteLine("Unable to read variable from slave, the variable is not declared as an attribute of the C# object");
            return (Fmi2Status.Error, null);
        }
    }
    public virtual (Fmi2Status,IEnumerable<bool>) GetBool(IEnumerable<uint> valueReferences)
    {
        try {
            IList<bool> values = new List<bool>(){};
            foreach (uint vref in valueReferences)
            {
                values.Add((bool)GetPropertyValue(vref));
            }
            return (Fmi2Status.Ok,values);
        }
        catch{
            this.sw.WriteLine("Unable to read variable from slave, the variable is not declared as an attribute of the C# object");
            return (Fmi2Status.Error, null);
        }
    }
    public virtual (Fmi2Status,IEnumerable<string>) GetString(IEnumerable<uint> valueReferences)
    {
        try {
            IList<string> values = new List<string>(){};
            foreach (uint vref in valueReferences)
            {
                values.Add((string)GetPropertyValue(vref));
            }
            return (Fmi2Status.Ok,values);
        }
        catch{
            this.sw.WriteLine("Unable to read variable from slave, the variable is not declared as an attribute of the C# object");
            return (Fmi2Status.Error, null);
        }
    }
    public virtual Fmi2Status SetReal(IEnumerable<uint> valueReferences, IEnumerable<double> values)
    {
        try {
            for (int i = 0; i < valueReferences.Count(); ++i)
            {
                //sw.WriteLine("valuereferece");
                this.SetPropertyValue(valueReferences.ElementAt(i), values.ElementAt(i));
            }
            return Fmi2Status.Ok;
        }
        catch{
            sw.WriteLine("Unable to set variable on slave, the variable is not declated as an attribute of the C# object.");
            return Fmi2Status.Error;
        }
    }

    public virtual Fmi2Status SetInt(IEnumerable<uint> valueReferences, IEnumerable<int> values)
    {
        try {
            for (int i = 0; i < valueReferences.Count(); ++i)
            {
                this.SetPropertyValue(valueReferences.ElementAt(i), values.ElementAt(i));
            }
            return Fmi2Status.Ok;
        }
        catch{
            sw.WriteLine("Unable to set variable on slave, the variable is not declated as an attribute of the C# object.");
            return Fmi2Status.Error;
        }
    }

    public virtual Fmi2Status SetBool(IEnumerable<uint> valueReferences, IEnumerable<bool> values)
    {
        try {
            for (int i = 0; i < valueReferences.Count(); ++i)
            {
                this.SetPropertyValue(valueReferences.ElementAt(i), values.ElementAt(i));
            }
            return Fmi2Status.Ok;
        }
        catch{
            sw.WriteLine("Unable to set variable on slave, the variable is not declated as an attribute of the C# object.");
            return Fmi2Status.Error;
        }
    }

    public virtual Fmi2Status SetString(IEnumerable<uint> valueReferences, IEnumerable<string> values)
    {
        try {
            for (int i = 0; i < valueReferences.Count(); ++i)
            {
                this.SetPropertyValue(valueReferences.ElementAt(i), values.ElementAt(i));
            }
            return Fmi2Status.Ok;
        }
        catch{
            sw.WriteLine("Unable to set variable on slave, the variable is not declated as an attribute of the C# object.");
            return Fmi2Status.Error;
        }
    }


    /// <summary>
    /// Convert the state of the FMU into a sequences of bytes which can later be used to roll-back the state of the FMU to that point
    /// </summary>
    /// <returns></returns>
    public virtual (byte[], Fmi2Status) Serialize()
    {
        throw new NotImplementedException();
    }

    /// <summary>
    /// Restore a FMU to the state recoreded by the serialize method.
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status Deserialize(byte[] state)
    {
        throw new NotImplementedException();
    }


    /********************************************************* Co-sim **********************************************************/

    public virtual Fmi2Status SetInputDerivatives()
    {
        throw new NotImplementedException();
    }

    public virtual Fmi2Status GetInputDerivatives()
    {
        throw new NotImplementedException();
    }

    public virtual Fmi2Status DoStep(double currentTime, double stepSize, bool noStepPrior)
    {
        return Fmi2Status.Ok;
    }

    public virtual Fmi2Status CancelStep()
    {
        throw new NotImplementedException();
    }

    /// <summary>
    /// Inquire about the status of an async FMU's step methods progress.
    /// </summary>
    /// <param name="kind"></param>
    /// <returns>Tuple of Fmi2StatusKind and object which can be of the types: Fmi2Status, string, double, bool</returns>
    public virtual (Fmi2StatusKind, object) GetXXXStatus(Fmi2StatusKind kind)
    {
        throw new NotImplementedException();
    }

}
