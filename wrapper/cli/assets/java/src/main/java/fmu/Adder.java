package fmu;

public class Adder extends FMI2FMU {

    public double real_a;
    public double real_b;
    public double real_c;

    public int integer_a;
    public int integer_b;
    public int integer_c;

    public boolean boolean_a;
    public boolean boolean_b;
    public boolean boolean_c;

    public String string_a;
    public String string_b;
    public String string_c;

    public Adder() {
        super();

    }

    @Override
    public FMI2Status exitInitializationMode() {
        real_a = 0;
        real_b = 0;
        integer_a = 0;
        integer_b = 0;
        boolean_a = false;
        boolean_b = false;
        string_a = "";
        string_b = "";

        update_outputs();

        return FMI2Status.OK;
    }

    private void update_outputs() {
        real_c = real_a + real_b;
        integer_c = integer_a + integer_b;
        boolean_c = boolean_a || boolean_b;
        string_c = string_a + string_b;
    }

    @Override
    public FMI2Status doStep(double currentTime, double stepSize, boolean noStepPrior) {
        update_outputs();

        return FMI2Status.OK;
    }

    @Override
    public byte[] serialize() throws RuntimeException {
        // TODO Auto-generated method stub
        return super.serialize();
    }

    @Override
    public FMI2Status deserialize(byte[] state) throws RuntimeException {
        // TODO Auto-generated method stub
        return super.deserialize(state);
    }

}
