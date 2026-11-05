package org.intocps.unifmu.model.fmi2;

import java.util.function.Consumer;
import org.intocps.unifmu.model.ModelSelectionSupport;
import org.intocps.unifmu.model.fmi2.Fmi2Model;
import org.intocps.unifmu.model.fmi2.Fmi2ModelFactory;
import org.intocps.unifmu.protocol.Fmi2Messages;

public final class ModelSelection {
    private ModelSelection() {}

    public static Fmi2ModelFactory fmi2Factory() {
        return (instantiation, logHandler) -> ModelSelectionSupport.create(
            Fmi2Model.class,
            Fmi2Messages.Fmi2Instantiate.class,
            instantiation,
            Consumer.class,
            logHandler,
            "org.intocps.unifmu.example.fmi2.Model"
        );
    }
}
