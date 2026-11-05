package org.intocps.unifmu.testfixtures;

import java.util.function.Consumer;
import org.intocps.unifmu.model.fmi3.Fmi3Model;
import org.intocps.unifmu.example.fmi3.Model;
import org.intocps.unifmu.protocol.Fmi3Messages;

public final class Fmi3DoStepFailureModel extends Model {
    public Fmi3DoStepFailureModel(
        Fmi3Messages.Fmi3InstantiateCoSimulation instantiation,
        Consumer<Fmi3Model.LogMessage> logHandler
    ) throws Exception {
        super(instantiation, logHandler);
    }

    @Override
    public Fmi3DoStepResult fmi3DoStep(
        double currentCommunicationPoint,
        double communicationStepSize,
        boolean noStepPrior
    ) {
        return new Fmi3DoStepResult(
            Fmi3Status.Error, false, false, false, currentCommunicationPoint
        );
    }
}
