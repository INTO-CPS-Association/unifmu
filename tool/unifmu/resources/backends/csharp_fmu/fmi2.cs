using System.Collections.Generic;
using System.Reflection;
using System;
using flatbuffers;


// /// <summary> Enum <c>Fmi2StatusKind</c>
// /// Defines the different types of statuses the master can inquire the slave about, see p.104
// /// These are used for async related functionality of FMI2.
// /// Values:
// ///     * do_step_status: request the status of the step function. If not completed fmi2Pending is returned,
// ///                     if complete the status of the step function is returned. 
// ///     * pending_status: request a string description of describing the progress of the step function.
// ///     * last_successfull_time: returns the simulation time of the last successfull simulation step.
// ///     * terminated: ask the slave if it wants to terminate the simulation. This can be called after the 
// ///                    step function returns the discard status.
// /// </summary>
// public enum Fmi2StatusKind : ushort
// {
//     do_step_status = 0,
//     pending_status = 1,
//     last_successfull_time = 2,
//     terminated = 3
// }


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

    /** c# specific **/
    public object this[string name]
    {
        get
        {
            var properties = GetType()
                    .GetProperties(BindingFlags.Public | BindingFlags.Instance);

            foreach (var property in properties)
            {
                if (property.Name == name && property.CanRead)
                    return property.GetValue(this, null);
            }

            throw new ArgumentException("Can't find property");

        }
        set
        {
            Type myType = GetType();
            PropertyInfo myPropInfo = myType.GetProperty(name);
            myPropInfo.SetValue(this, value);
        }
    }

    /********************************************************* COMMON **********************************************************/
    public virtual Fmi2Status SetDebugLogging(string[] categories, bool loggingOn)
    {
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

    /********************************************************* Getters and Setters implemented in launch.cs **********************************************************/

    /// <summary>
    /// Convert the state of the FMU into a sequences of bytes which can later be used to roll-back the state of the FMU to that point
    /// </summary>
    /// <returns>string</returns>
    public virtual (string, Fmi2Status) Serialize()
    {
        throw new NotImplementedException();
    }

    /// <summary>
    /// Restore a FMU to the state recoreded by the serialize method.
    /// </summary>
    /// <returns>Fmi2Status</returns>
    public virtual Fmi2Status Deserialize(string state)
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
    public virtual (Fmi2CommandArg, object) GetDoStepStatus(Fmi2CommandArg kind)
    {
        throw new NotImplementedException();
    }

}
