package org.intocps.unifmu.model.fmi3;

import java.util.function.Consumer;
import org.intocps.unifmu.model.fmi3.Fmi3Model;
import org.intocps.unifmu.protocol.Fmi3Messages;

@FunctionalInterface
public interface Fmi3ModelFactory {
    Fmi3Model create(
        Fmi3Messages.Fmi3InstantiateCoSimulation instantiation,
        Consumer<Fmi3Model.LogMessage> logHandler
    ) throws Exception;
}
