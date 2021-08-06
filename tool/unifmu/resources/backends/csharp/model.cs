using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;

public class Model : Fmi2FMU
{
  // Make all class variables properties, in order to access them all in a similar manner. 
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


  public Model(Dictionary<uint, string> referenceToAttr) : base(referenceToAttr)
  {
    this.real_a = 0.0f;
    this.real_b = 0.0f;

    this.integer_a = 0;
    this.integer_b = 0;

    this.boolean_a = false;
    this.boolean_b = false;

    this.string_a = "";
    this.string_b = "";

    UpdateOutputs();
  }


  public override (byte[], Fmi2Status) Serialize()
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
      return (m.ToArray(), Fmi2Status.Ok);
    }
  }

  public override Fmi2Status Deserialize(byte[] state)
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
    return Fmi2Status.Ok;
  }

  // Implementation of properties

  // public double real_c
  // {
  //     get { return this.real_a + this.real_b; }
  //     set { }
  // }

  // public int integer_c
  // {
  //     get { return this.integer_a + this.integer_b; }
  //     set { }
  // }

  // public bool boolean_c
  // {
  //     get { return this.boolean_a & this.boolean_b; }
  //     set { }
  // }

  // public string string_c
  // {
  //     get { return this.string_a + this.string_b; }
  //     set { }
  // }

  private void UpdateOutputs()
  {
    this.real_c = real_a + real_b;
    this.integer_c = integer_a + integer_b;
    this.boolean_c = boolean_a || boolean_b;
    this.string_c = string_a + string_b;
  }

  public override Fmi2Status Fmi2DoStep(double currentTime, double stepSize, bool noStepPrior)
  {
    UpdateOutputs();
    return Fmi2Status.Ok;
  }

}