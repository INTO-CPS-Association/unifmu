package org.intocps.unifmu.model.fmi3;

import java.util.function.Consumer;
import org.intocps.unifmu.model.ModelSelectionSupport;
import org.intocps.unifmu.model.fmi3.Fmi3Model;
import org.intocps.unifmu.model.fmi3.Fmi3ModelFactory;
import org.intocps.unifmu.protocol.Fmi3Messages;

public final class ModelSelection {
    private ModelSelection() {}

    public static Fmi3ModelFactory fmi3Factory() {
        return (instantiation, logHandler) -> ModelSelectionSupport.create(
            Fmi3Model.class,
            Fmi3Messages.Fmi3InstantiateCoSimulation.class,
            instantiation,
            Consumer.class,
            logHandler,
            "org.intocps.unifmu.example.fmi3.Model"
        );
    }
}
