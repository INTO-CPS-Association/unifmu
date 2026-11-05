package org.intocps.unifmu.model.fmi2;

import java.util.function.Consumer;
import org.intocps.unifmu.model.fmi2.Fmi2Model;
import org.intocps.unifmu.protocol.Fmi2Messages;

@FunctionalInterface
public interface Fmi2ModelFactory {
    Fmi2Model create(
        Fmi2Messages.Fmi2Instantiate instantiation,
        Consumer<Fmi2Model.LogMessage> logHandler
    ) throws Exception;
}
