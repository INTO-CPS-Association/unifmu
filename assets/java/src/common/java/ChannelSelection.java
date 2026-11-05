package org.intocps.unifmu.communication;

import java.io.IOException;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import org.intocps.unifmu.communication.CommunicationChannel;
import org.intocps.unifmu.communication.ZeroMqCommunicationChannel;

public final class ChannelSelection {
    public static final String PROPERTY_NAME = "unifmu.channel.impl";

    private ChannelSelection() {}

    public static CommunicationChannel connect(String endpoint) throws IOException {
        String className = System.getProperty(
            PROPERTY_NAME, ZeroMqCommunicationChannel.class.getName()
        );
        final Class<?> implementation;
        try {
            implementation = Class.forName(className);
        } catch (ClassNotFoundException e) {
            throw new IllegalArgumentException(
                "Configured communication channel '" + className + "' was not found.", e
            );
        }
        if (!CommunicationChannel.class.isAssignableFrom(implementation)) {
            throw new IllegalArgumentException(
                "Configured communication channel '" + className
                    + "' must implement " + CommunicationChannel.class.getName() + "."
            );
        }
        try {
            Constructor<?> constructor = implementation.getConstructor(String.class);
            return (CommunicationChannel) constructor.newInstance(endpoint);
        } catch (NoSuchMethodException e) {
            throw new IllegalArgumentException(
                "Configured communication channel '" + className
                    + "' must expose a public constructor accepting a String endpoint.", e
            );
        } catch (InstantiationException | IllegalAccessException e) {
            throw new IllegalArgumentException(
                "Could not create configured communication channel '" + className + "'.", e
            );
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof IOException ioException) {
                throw ioException;
            }
            if (cause instanceof RuntimeException runtimeException) {
                throw runtimeException;
            }
            throw new IOException(
                "Configured communication channel '" + className + "' failed to initialize.",
                cause
            );
        }
    }
}
