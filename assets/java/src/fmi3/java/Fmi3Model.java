package org.intocps.unifmu.model.fmi3;

import java.nio.ByteBuffer;
import java.util.List;
import java.util.function.Consumer;

public interface Fmi3Model {
    enum Fmi3Status {
        OK, Warning, Discard, Error, Fatal, Pending
    }

    record LogMessage(Fmi3Status status, String category, String message) {}

    class Fmi3DoStepResult {
        public Fmi3Status status;
        public Boolean event_handling_needed;
        public Boolean terminate_simulation;
        public Boolean early_return;
        public Double last_successful_time;

        public Fmi3DoStepResult(Fmi3Status status, Boolean eventHandlingNeeded,
            Boolean terminateSimulation, Boolean earlyReturn, Double lastSuccessfulTime) {
            this.status = status;
            this.event_handling_needed = eventHandlingNeeded;
            this.terminate_simulation = terminateSimulation;
            this.early_return = earlyReturn;
            this.last_successful_time = lastSuccessfulTime;
        }
    }

    class Fmi3UpdateDiscreteStatesResult {
        public Fmi3Status status;
        public Boolean discrete_states_need_update;
        public Boolean terminate_simulation;
        public Boolean nominals_continuous_states_changed;
        public Boolean values_continuous_states_changed;
        public Boolean next_event_time_defined;
        public Double next_event_time;

        public Fmi3UpdateDiscreteStatesResult(Fmi3Status status, Boolean discreteStatesNeedUpdate,
            Boolean terminateSimulation, Boolean nominalsChanged, Boolean valuesChanged,
            Boolean nextEventTimeDefined, Double nextEventTime) {
            this.status = status;
            this.discrete_states_need_update = discreteStatesNeedUpdate;
            this.terminate_simulation = terminateSimulation;
            this.nominals_continuous_states_changed = nominalsChanged;
            this.values_continuous_states_changed = valuesChanged;
            this.next_event_time_defined = nextEventTimeDefined;
            this.next_event_time = nextEventTime;
        }
    }

    class Fmi3GetValuePair<T> {
        public Fmi3Status status;
        public List<T> values;
        public Fmi3GetValuePair(Fmi3Status status, List<T> values) {
            this.status = status;
            this.values = values;
        }
    }

    class Fmi3GetIntervalDecimalPair<T, U> {
        public Fmi3Status status;
        public List<T> intervals;
        public List<U> qualifiers;
        public Fmi3GetIntervalDecimalPair(Fmi3Status status, List<T> intervals, List<U> qualifiers) {
            this.status = status;
            this.intervals = intervals;
            this.qualifiers = qualifiers;
        }
    }

    class Fmi3GetIntervalFractionPair<T, U> {
        public Fmi3Status status;
        public List<T> counters;
        public List<T> resolutions;
        public List<U> qualifiers;
        public Fmi3GetIntervalFractionPair(Fmi3Status status, List<T> counters,
            List<T> resolutions, List<U> qualifiers) {
            this.status = status;
            this.counters = counters;
            this.resolutions = resolutions;
            this.qualifiers = qualifiers;
        }
    }

    class Fmi3GetShiftDecimalPair<T> {
        public Fmi3Status status;
        public List<T> shifts;
        public Fmi3GetShiftDecimalPair(Fmi3Status status, List<T> shifts) {
            this.status = status;
            this.shifts = shifts;
        }
    }

    class Fmi3GetShiftFractionPair<T> {
        public Fmi3Status status;
        public List<T> counters;
        public List<T> resolutions;
        public Fmi3GetShiftFractionPair(Fmi3Status status, List<T> counters, List<T> resolutions) {
            this.status = status;
            this.counters = counters;
            this.resolutions = resolutions;
        }
    }

    class Fmi3SerializeFmuStatePair {
        public Fmi3Status status;
        public ByteBuffer bytes;
        public Fmi3SerializeFmuStatePair(Fmi3Status status, ByteBuffer bytes) {
            this.status = status;
            this.bytes = bytes;
        }
    }

    void setLogHandler(Consumer<LogMessage> logHandler);
    Fmi3Status fmi3SetDebugLogging(Iterable<String> categories, Boolean loggingOn) throws Exception;
    Fmi3DoStepResult fmi3DoStep(double currentPoint, double stepSize, boolean noStepPrior) throws Exception;
    Fmi3UpdateDiscreteStatesResult fmi3UpdateDiscreteStates() throws Exception;
    Fmi3Status fmi3SetFloat32(Iterable<Integer> refs, Iterable<Float> values) throws Exception;
    Fmi3Status fmi3SetFloat64(Iterable<Integer> refs, Iterable<Double> values) throws Exception;
    Fmi3Status fmi3SetInt8(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetUInt8(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetInt16(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetUInt16(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetInt32(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetUInt32(Iterable<Integer> refs, Iterable<Integer> values) throws Exception;
    Fmi3Status fmi3SetInt64(Iterable<Integer> refs, Iterable<Long> values) throws Exception;
    Fmi3Status fmi3SetUInt64(Iterable<Integer> refs, Iterable<Long> values) throws Exception;
    Fmi3Status fmi3SetBoolean(Iterable<Integer> refs, Iterable<Boolean> values) throws Exception;
    Fmi3Status fmi3SetString(Iterable<Integer> refs, Iterable<String> values) throws Exception;
    Fmi3Status fmi3SetBinary(Iterable<Integer> refs, Iterable<Long> sizes, Iterable<ByteBuffer> values) throws Exception;
    Fmi3Status fmi3SetClock(Iterable<Integer> refs, Iterable<Boolean> values) throws Exception;
    Fmi3Status fmi3SetIntervalDecimal(Iterable<Integer> refs, Iterable<Double> values) throws Exception;
    Fmi3Status fmi3SetIntervalFraction(Iterable<Integer> refs, Iterable<Long> counters, Iterable<Long> resolutions) throws Exception;
    Fmi3Status fmi3SetShiftDecimal(Iterable<Integer> refs, Iterable<Double> values) throws Exception;
    Fmi3Status fmi3SetShiftFraction(Iterable<Integer> refs, Iterable<Long> counters, Iterable<Long> resolutions) throws Exception;
    Fmi3GetValuePair<Float> fmi3GetFloat32(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Double> fmi3GetFloat64(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetInt8(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetUInt8(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetInt16(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetUInt16(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetInt32(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Integer> fmi3GetUInt32(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Long> fmi3GetInt64(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Long> fmi3GetUInt64(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Boolean> fmi3GetBoolean(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<String> fmi3GetString(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<ByteBuffer> fmi3GetBinary(Iterable<Integer> refs) throws Exception;
    Fmi3GetValuePair<Boolean> fmi3GetClock(Iterable<Integer> refs) throws Exception;
    Fmi3GetIntervalDecimalPair<Double, Integer> fmi3GetIntervalDecimal(Iterable<Integer> refs);
    Fmi3GetIntervalFractionPair<Long, Integer> fmi3GetIntervalFraction(Iterable<Integer> refs);
    Fmi3GetShiftDecimalPair<Double> fmi3GetShiftDecimal(Iterable<Integer> refs);
    Fmi3GetShiftFractionPair<Long> fmi3GetShiftFraction(Iterable<Integer> refs);
    Fmi3Status fmi3EnterInitializationMode() throws Exception;
    Fmi3Status fmi3ExitInitializationMode() throws Exception;
    Fmi3Status fmi3EnterEventMode() throws Exception;
    Fmi3Status fmi3EnterStepMode() throws Exception;
    Fmi3Status fmi3EnterConfigurationMode() throws Exception;
    Fmi3Status fmi3ExitConfigurationMode() throws Exception;
    Fmi3Status fmi3Reset() throws Exception;
    Fmi3Status fmi3Terminate() throws Exception;
    Fmi3SerializeFmuStatePair fmi3SerializeFmuState() throws Exception;
    Fmi3Status fmi3DeserializeFmuState(ByteBuffer bytes) throws Exception;
}
