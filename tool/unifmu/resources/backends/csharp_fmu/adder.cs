
using flatbuffers;

public class Adder : Fmi2FMU
{
    // Make all class variables properties, in order to access them all in a similar manner. 
    public double real_a { get; set; }
    public double real_b { get; set; }
    public int integer_a { get; set; }
    public int integer_b { get; set; }
    public bool boolean_a { get; set; }
    public bool boolean_b { get; set; }
    public string string_a { get; set; }
    public string string_b { get; set; }

    public Adder() : base()
    {
        this.real_a = 0.0f;
        this.real_b = 0.0f;

        this.integer_a = 0;
        this.integer_b = 0;

        this.boolean_a = false;
        this.boolean_b = false;

        this.string_a = "";
        this.string_b = "";
    }

    // TODO: implement correctly
    public override (byte[], Fmi2Status) Serialize()
    {
        return (null, Fmi2Status.Ok);
    }

    // TODO: implement correctly
    public override Fmi2Status Deserialize(byte[] state)
    {
        return base.Deserialize(state);
    }

    // Implementation of properties

    public double real_c
    {
        get { return this.real_a + this.real_b; }
        set { }
    }

    public int integer_c
    {
        get { return this.integer_a + this.integer_b; }
        set { }
    }

    public bool boolean_c
    {
        get { return this.boolean_a & this.boolean_b; }
        set { }
    }

    public string string_c
    {
        get { return this.string_a + this.string_b; }
        set { }
    }

}