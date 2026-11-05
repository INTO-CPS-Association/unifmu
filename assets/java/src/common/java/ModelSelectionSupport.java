package org.intocps.unifmu.model;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;

public final class ModelSelectionSupport {
    public static final String PROPERTY_NAME = "unifmu.model.impl";

    private ModelSelectionSupport() {}

    public static <T> T create(
        Class<T> modelType,
        Class<?> instantiationType,
        Object instantiation,
        Class<?> loggerType,
        Object logger,
        String defaultClassName
    ) throws Exception {
        String className = System.getProperty(PROPERTY_NAME, defaultClassName);
        final Class<?> implementation;
        try {
            implementation = Class.forName(className);
        } catch (ClassNotFoundException e) {
            throw new IllegalArgumentException(
                "Configured FMU model implementation '" + className + "' was not found.", e
            );
        }
        if (!modelType.isAssignableFrom(implementation)) {
            throw new IllegalArgumentException(
                "Configured FMU model implementation '" + className + "' must implement "
                    + modelType.getName() + "."
            );
        }

        try {
            Constructor<?> constructor = implementation.getConstructor(instantiationType, loggerType);
            return modelType.cast(constructor.newInstance(instantiation, logger));
        } catch (NoSuchMethodException e) {
            throw new IllegalArgumentException(
                "Configured FMU model implementation '" + className + "' must expose a public "
                    + "constructor accepting " + instantiationType.getName() + " and "
                    + loggerType.getName() + ".", e
            );
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            throw new IllegalStateException(
                "Configured FMU model implementation '" + className + "' failed to initialize.",
                cause
            );
        }
    }
}
