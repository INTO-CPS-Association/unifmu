package org.intocps.unifmu.testfixtures;

import java.util.function.Consumer;
import org.intocps.unifmu.model.fmi2.Fmi2Model;
import org.intocps.unifmu.example.fmi2.Model;
import org.intocps.unifmu.protocol.Fmi2Messages;

public final class Fmi2DoStepFailureModel extends Model {
    public Fmi2DoStepFailureModel(
        Fmi2Messages.Fmi2Instantiate instantiation,
        Consumer<Fmi2Model.LogMessage> logHandler
    ) throws Exception {
        super(instantiation, logHandler);
    }

    @Override
    public Fmi2Status fmi2DoStep(double currentTime, double stepSize, boolean noStepPrior) {
        return Fmi2Status.Error;
    }
}
