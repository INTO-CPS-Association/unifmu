import pickle
from fractions import Fraction
from enum import IntFlag

class Model:
    def __init__(
            self,
            instance_name,
            instantiation_token,
            resource_path,
            visible,
            logging_on,
            event_mode_used,
            early_return_allowed,
            required_intermediate_variables,
            _log_callback
    ) -> None:
        self.instance_name = instance_name
        self.instantiation_token = instantiation_token
        self.resource_path = resource_path
        self.visible = visible
        self.logging_on = logging_on
        self.event_mode_used = event_mode_used
        self.early_return_allowed = early_return_allowed
        self.required_intermediate_variables = required_intermediate_variables
        self._log_callback = _log_callback
        self.state = FMIState.FMIInstantiatedState
        self.float32_a = 0.0
        self.float32_b = 0.0
        self.float32_c = 0.0
        self.float64_a = 0.0
        self.float64_b = 0.0
        self.float64_c = 0.0
        self.int8_a = 0
        self.int8_b = 0
        self.int8_c = 0
        self.uint8_a = 0
        self.uint8_b = 0
        self.uint8_c = 0
        self.int16_a = 0
        self.int16_b = 0
        self.int16_c = 0
        self.uint16_a = 0
        self.uint16_b = 0
        self.uint16_c = 0
        self.int32_a = 0
        self.int32_b = 0
        self.int32_c = 0
        self.uint32_a = 0
        self.uint32_b = 0
        self.uint32_c = 0
        self.int64_a = 0
        self.int64_b = 0
        self.int64_c = 0
        self.uint64_a = 0
        self.uint64_b = 0
        self.uint64_c = 0
        self.boolean_a = False
        self.boolean_b = False
        self.boolean_c = False
        self.string_a = ""
        self.string_b = ""
        self.string_c = ""
        self.binary_a = bytes([0])
        self.binary_b = bytes([0])
        self.binary_c = bytes([0])
        self.float32_tunable_parameter = 0.0
        self.float64_tunable_parameter = 0.0
        self.int8_tunable_parameter = 0
        self.uint8_tunable_parameter = 0
        self.int16_tunable_parameter = 0
        self.uint16_tunable_parameter = 0
        self.int32_tunable_parameter = 0
        self.uint32_tunable_parameter = 0
        self.int64_tunable_parameter = 0
        self.uint64_tunable_parameter = 0
        self.boolean_tunable_parameter = False
        self.string_tunable_parameter = ""
        self.binary_tunable_parameter = bytes([0])
        self.uint64_tunable_structural_parameter = 5
        self.float32_vector_using_tunable_structural_parameter = [0.1,0.2,0.3,0.4,0.5]
        self.clock_a = False
        self.clock_b = False
        self.clock_c = False
        self.clocked_variable_a = 0
        self.clocked_variable_b = 0
        self.clocked_variable_c = 0
        self.clock_reference_to_interval = {
            1001: 1.0,
        }
        self.clock_reference_to_shift = {
            1001: 1.0,
        }

        self.reference_to_attribute = {
            999: "time",
            0: "float32_a",
            1: "float32_b",
            2: "float32_c",
            3: "float64_a",
            4: "float64_b",
            5: "float64_c",
            6: "int8_a",
            7: "int8_b",
            8: "int8_c",
            9: "uint8_a",
            10: "uint8_b",
            11: "uint8_c",
            12: "int16_a",
            13: "int16_b",
            14: "int16_c",
            15: "uint16_a",
            16: "uint16_b",
            17: "uint16_c",
            18: "int32_a",
            19: "int32_b",
            20: "int32_c",
            21: "uint32_a",
            22: "uint32_b",
            23: "uint32_c",
            24: "int64_a",
            25: "int64_b",
            26: "int64_c",
            27: "uint64_a",
            28: "uint64_b",
            29: "uint64_c",
            30: "boolean_a",
            31: "boolean_b",
            32: "boolean_c",
            33: "string_a",
            34: "string_b",
            35: "string_c",
            36: "binary_a",
            37: "binary_b",
            38: "binary_c",            
        }

        self.clocked_variables = {
            1001: "clock_a",
            1002: "clock_b",
            1003: "clock_c",
            1100: "clocked_variable_a",
            1101: "clocked_variable_b",
            1102: "clocked_variable_c",
        }

        self.parameters = {
            
        }

        self.tunable_parameters = {
            100: "float32_tunable_parameter",
            101: "float64_tunable_parameter",
            102: "int8_tunable_parameter",
            103: "uint8_tunable_parameter",
            104: "int16_tunable_parameter",
            105: "uint16_tunable_parameter",
            106: "int32_tunable_parameter",
            107: "uint32_tunable_parameter",
            108: "int64_tunable_parameter",
            109: "uint64_tunable_parameter",
            110: "boolean_tunable_parameter",
            111: "string_tunable_parameter",
            112: "binary_tunable_parameter",
            114: "float32_vector_using_tunable_structural_parameter",
        }

        self.tunable_structural_parameters = {
            113: "uint64_tunable_structural_parameter",
            
        }

        self.all_references = {**self.tunable_structural_parameters,
                               **self.parameters,
                               **self.tunable_parameters,
                               **self.clocked_variables,
                               **self.reference_to_attribute}
        
        self.all_parameters = {**self.tunable_structural_parameters,
                               **self.parameters,
                               **self.tunable_parameters}

        self._update_outputs()
        self._update_clocks()
        self._update_clocked_outputs()

    # ================= FMI3 =================

    # ================= doStep and updateDiscreteStates =================
    def fmi3DoStep(
            self,
            current_communication_point: float,
            communication_step_size: float,
            no_set_fmu_state_prior_to_current_point: bool,
    ):
        self._update_outputs()

        event_handling_needed = False
        terminate_simulation = False
        early_return = False
        last_successful_time = current_communication_point + communication_step_size

        return (
            Fmi3Status.ok,
            event_handling_needed,
            terminate_simulation,
            early_return,
            last_successful_time,
        )
    
    def fmi3UpdateDiscreteStates(self):
        status = Fmi3Status.ok
        discrete_states_need_update = False
        terminate_simulation = False
        nominals_continuous_states_changed = False
        values_continuous_states_changed = False
        next_event_time_defined = True
        next_event_time = 1.0


        self._update_clocked_outputs()

        return (status, discrete_states_need_update, terminate_simulation, nominals_continuous_states_changed,
                values_continuous_states_changed, next_event_time_defined, next_event_time)

    # ================= Initialization, Enter, Termination, and Reset =================

    def fmi3EnterInitializationMode(
            self,
            tolerance_defined: bool,
            tolerance: float,
            start_time: float,
            stop_time_defined: bool,
            stop_time: float
    ):
        self.state = FMIState.FMIInitializationModeState
        return Fmi3Status.ok

    def fmi3ExitInitializationMode(self):
        self.state = FMIState.FMIEventModeState if self.event_mode_used else FMIState.FMIStepModeState
        self._update_outputs()
        return Fmi3Status.ok

    def fmi3EnterEventMode(self):
        self.state = FMIState.FMIEventModeState
        return Fmi3Status.ok

    def fmi3EnterStepMode(self):
        self.state = FMIState.FMIStepModeState
        return Fmi3Status.ok
    
    def fmi3EnterConfigurationMode(self):
        if len(self.tunable_structural_parameters)>0:
            self.state = FMIState.FMIConfigurationModeState if self.state == FMIState.FMIInstantiatedState else FMIState.FMIReconfigurationModeState
        else:
            return Fmi3Status.error
        return Fmi3Status.ok

    def fmi3ExitConfigurationMode(self):
        if self.state == FMIState.FMIConfigurationModeState:
            self.state = FMIState.FMIInstantiatedState
        elif self.state == FMIState.FMIReconfigurationModeState:
            self.state = FMIState.FMIStepModeState
        else:
            return Fmi3Status.error
        return Fmi3Status.ok

    def fmi3Terminate(self):
        self.state = FMIState.FMITerminatedState
        return Fmi3Status.ok

    def fmi3Reset(self):
        self.state = FMIState.FMIInstantiatedState
        self.float32_a = 0.0
        self.float32_b = 0.0
        self.float32_c = 0.0
        self.float64_a = 0.0
        self.float64_b = 0.0
        self.float64_c = 0.0
        self.int8_a = 0
        self.int8_b = 0
        self.int8_c = 0
        self.uint8_a = 0
        self.uint8_b = 0
        self.uint8_c = 0
        self.int16_a = 0
        self.int16_b = 0
        self.int16_c = 0
        self.uint16_a = 0
        self.uint16_b = 0
        self.uint16_c = 0
        self.int32_a = 0
        self.int32_b = 0
        self.int32_c = 0
        self.uint32_a = 0
        self.uint32_b = 0
        self.uint32_c = 0
        self.int64_a = 0
        self.int64_b = 0
        self.int64_c = 0
        self.uint64_a = 0
        self.uint64_b = 0
        self.uint64_c = 0
        self.boolean_a = False
        self.boolean_b = False
        self.boolean_c = False
        self.string_a = ""
        self.string_b = ""
        self.string_c = ""
        self.binary_a = bytes([0])
        self.binary_b = bytes([0])
        self.binary_c = bytes([0])
        self.float32_tunable_parameter = 0.0
        self.float64_tunable_parameter = 0.0
        self.int8_tunable_parameter = 0
        self.uint8_tunable_parameter = 0
        self.int16_tunable_parameter = 0
        self.uint16_tunable_parameter = 0
        self.int32_tunable_parameter = 0
        self.uint32_tunable_parameter = 0
        self.int64_tunable_parameter = 0
        self.uint64_tunable_parameter = 0
        self.boolean_tunable_parameter = False
        self.string_tunable_parameter = ""
        self.binary_tunable_parameter = bytes([0])
        self.uint64_tunable_structural_parameter = 5
        self.float32_vector_using_tunable_structural_parameter = [0.1,0.2,0.3,0.4,0.5]
        self.clock_a = False
        self.clock_b = False
        self.clock_c = False
        self.clocked_variable_a = 0
        self.clocked_variable_b = 0
        self.clocked_variable_c = 0
        self.clock_reference_to_interval = {
            1001: 1.0,
        }
        self.clock_reference_to_shift = {
            1001: 1.0,
        }

        self._update_outputs()
        self._update_clocks()
        self._update_clocked_outputs()
        return Fmi3Status.ok

    # ================= Serialization =================

    def fmi3SerializeFmuState(self):

        bytes = pickle.dumps(
            (
                self.state,
                self.float32_a,
                self.float32_b,
                self.float64_a,
                self.float64_b,
                self.int8_a,
                self.int8_b,
                self.uint8_a,
                self.uint8_b,
                self.int16_a,
                self.int16_b,
                self.uint16_a,
                self.uint16_b,
                self.int32_a,
                self.int32_b,
                self.uint32_a,
                self.uint32_b,
                self.int64_a,
                self.int64_b,
                self.uint64_a,
                self.uint64_b,
                self.boolean_a,
                self.boolean_b,
                self.string_a,
                self.string_b,
                self.binary_a,
                self.binary_b,
                self.float32_tunable_parameter,
                self.float64_tunable_parameter,
                self.int8_tunable_parameter,
                self.uint8_tunable_parameter,
                self.int16_tunable_parameter,
                self.uint16_tunable_parameter,
                self.int32_tunable_parameter,
                self.uint32_tunable_parameter,
                self.int64_tunable_parameter,
                self.uint64_tunable_parameter,
                self.boolean_tunable_parameter,
                self.string_tunable_parameter,
                self.binary_tunable_parameter,
                self.uint64_tunable_structural_parameter,
                self.float32_vector_using_tunable_structural_parameter,
                self.clock_a,
                self.clock_b,
                self.clock_c,
                self.clocked_variable_a,
                self.clocked_variable_b,
                self.clocked_variable_c,
                self.clock_reference_to_interval,
                self.clock_reference_to_shift,
            )
        )
        return Fmi3Status.ok, bytes

    def fmi3DeserializeFmuState(self, bytes: bytes):
        (
            state,
            float32_a,
            float32_b,
            float64_a,
            float64_b,
            int8_a,
            int8_b,
            uint8_a,
            uint8_b,
            int16_a,
            int16_b,
            uint16_a,
            uint16_b,
            int32_a,
            int32_b,
            uint32_a,
            uint32_b,
            int64_a,
            int64_b,
            uint64_a,
            uint64_b,
            boolean_a,
            boolean_b,
            string_a,
            string_b,
            binary_a,
            binary_b,
            float32_tunable_parameter,
            float64_tunable_parameter,
            int8_tunable_parameter,
            uint8_tunable_parameter,
            int16_tunable_parameter,
            uint16_tunable_parameter,
            int32_tunable_parameter,
            uint32_tunable_parameter,
            int64_tunable_parameter,
            uint64_tunable_parameter,
            boolean_tunable_parameter,
            string_tunable_parameter,
            binary_tunable_parameter,
            uint64_tunable_structural_parameter,
            float32_vector_using_tunable_structural_parameter,
            clock_a,
            clock_b,
            clock_c,
            clocked_variable_a,
            clocked_variable_b,
            clocked_variable_c,
            clock_reference_to_interval,
            clock_reference_to_shift,
        ) = pickle.loads(bytes)
        self.state = state
        self.float32_a = float32_a
        self.float32_b = float32_b
        self.float64_a = float64_a
        self.float64_b = float64_b
        self.int8_a = int8_a
        self.int8_b = int8_b
        self.uint8_a = uint8_a
        self.uint8_b = uint8_b
        self.int16_a = int16_a
        self.int16_b = int16_b
        self.uint16_a = uint16_a
        self.uint16_b = uint16_b
        self.int32_a = int32_a
        self.int32_b = int32_b
        self.uint32_a = uint32_a
        self.uint32_b = uint32_b
        self.int64_a = int64_a
        self.int64_b = int64_b
        self.uint64_a = uint64_a
        self.uint64_b = uint64_b
        self.boolean_a = boolean_a
        self.boolean_b = boolean_b
        self.string_a = string_a
        self.string_b = string_b
        self.binary_a = binary_a
        self.binary_b = binary_b
        self.float32_tunable_parameter = float32_tunable_parameter
        self.float64_tunable_parameter = float64_tunable_parameter
        self.int8_tunable_parameter = int8_tunable_parameter
        self.uint8_tunable_parameter = uint8_tunable_parameter
        self.int16_tunable_parameter = int16_tunable_parameter
        self.uint16_tunable_parameter = uint16_tunable_parameter
        self.int32_tunable_parameter = int32_tunable_parameter
        self.uint32_tunable_parameter = uint32_tunable_parameter
        self.int64_tunable_parameter = int64_tunable_parameter
        self.uint64_tunable_parameter = uint64_tunable_parameter
        self.boolean_tunable_parameter = boolean_tunable_parameter
        self.string_tunable_parameter = string_tunable_parameter
        self.binary_tunable_parameter = binary_tunable_parameter
        self.uint64_tunable_structural_parameter = uint64_tunable_structural_parameter
        self.float32_vector_using_tunable_structural_parameter = float32_vector_using_tunable_structural_parameter
        self.clock_a = clock_a
        self.clock_b = clock_b
        self.clock_c = clock_c
        self.clocked_variable_a = clocked_variable_a
        self.clocked_variable_b = clocked_variable_b
        self.clocked_variable_c = clocked_variable_c
        self.clock_reference_to_interval = clock_reference_to_interval
        self.clock_reference_to_shift = clock_reference_to_shift

        self._update_outputs()
        self._update_clocks()
        self._update_clocked_outputs()

        return Fmi3Status.ok
    
    # ================= Getters =================

    def fmi3GetFloat32(self, value_references):
        return self._get_value(value_references)

    def fmi3GetFloat64(self, value_references):
        return self._get_value(value_references)

    def fmi3GetInt8(self, value_references):
        return self._get_value(value_references)

    def fmi3GetUInt8(self, value_references):
        return self._get_value(value_references)

    def fmi3GetInt16(self, value_references):
        return self._get_value(value_references)

    def fmi3GetUInt16(self, value_references):
        return self._get_value(value_references)

    def fmi3GetInt32(self, value_references):
        return self._get_value(value_references)

    def fmi3GetUInt32(self, value_references):
        return self._get_value(value_references)

    def fmi3GetInt64(self, value_references):
        return self._get_value(value_references)

    def fmi3GetUInt64(self, value_references):
        return self._get_value(value_references)

    def fmi3GetBoolean(self, value_references):
        return self._get_value(value_references)

    def fmi3GetString(self, value_references):
        return self._get_value(value_references)

    def fmi3GetBinary(self, value_references):
        return self._get_value(value_references)

    def fmi3GetClock(self, value_references):
        return self._get_value(value_references)

    def fmi3GetIntervalDecimal(self, value_references):
        intervals = []
        qualifiers = []

        for r in value_references:
            intervals.append(self.clock_reference_to_interval[r])
            qualifiers.append(2)

        return Fmi3Status.ok, intervals, qualifiers
    
    def fmi3GetIntervalFraction(self, value_references):
        counters = []
        resolutions = []
        qualifiers = []

        for r in value_references:
            fraction = Fraction(str(self.clock_reference_to_interval[r]))
            numerator = fraction.numerator
            denominator = fraction.denominator
            counters.append(numerator)
            resolutions.append(denominator)
            qualifiers.append(2)

        return Fmi3Status.ok, counters, resolutions, qualifiers
    
    def fmi3GetShiftDecimal(self, value_references):
        shifts = []

        for r in value_references:
            shifts.append(self.clock_reference_to_shift[r])

        return Fmi3Status.ok, shifts
    
    def fmi3GetShiftFraction(self, value_references):
        counters = []
        resolutions = []

        for r in value_references:
            fraction = Fraction(str(self.clock_reference_to_shift[r]))
            numerator = fraction.numerator
            denominator = fraction.denominator
            counters.append(numerator)
            resolutions.append(denominator)

        return Fmi3Status.ok, counters, resolutions
    
    # ================= Setters =================

    def fmi3SetFloat32(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetFloat64(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetInt8(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetUInt8(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetInt16(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetUInt16(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetInt32(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetUInt32(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetInt64(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetUInt64(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetBoolean(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetString(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetBinary(self, value_references, value_sizes, values):
        # Store 'value_sizes' somewhere if needed
        return self._set_value(value_references, values)

    def fmi3SetClock(self, value_references, values):
        status = self._set_value(value_references, values)
        self._update_clocks()
        return status
    
    def fmi3SetIntervalDecimal(self, value_references, intervals):
        for idx,r in enumerate(value_references):
            self.clock_reference_to_interval[r] = intervals[idx]
        return Fmi3Status.ok
    
    def fmi3SetIntervalFraction(self, value_references, counters, resolutions):
        for idx,r in enumerate(value_references):
            self.clock_reference_to_interval[r] = float(counters[idx])/float(resolutions[idx])
        return Fmi3Status.ok
    
    def fmi3SetShiftDecimal(self, value_references, shifts):
        for idx,r in enumerate(value_references):
            self.clock_reference_to_shift[r] = shifts[idx]
        return Fmi3Status.ok
    
    def fmi3SetShiftFraction(self, value_references, counters, resolutions):
        for idx,r in enumerate(value_references):
            self.clock_reference_to_shift[r] = float(counters[idx])/float(resolutions[idx])
        return Fmi3Status.ok

    # ================= Logging =================

    """ UniFMU logging function

    Call this function whenever something should be logged.
    This will send a message thourgh the UniFMU layer to the importer if the
    importer has enabled logging and is interested in the given logging category.

    Keyword arguments:
    message  -- The message to log.
    status   -- The Fmi3Status that the program is expected to return when log() is
                called. For example, if the log() call is simply informing of normal
                operation, it would expect to return Fmi3Status.ok, while if the log()
                call is informing of an error it would expect to return Fmi3Status.error.
    category -- The logging category. Used by the UniFMU layer to dertermine whether or
                not a message should be send to the importer. A category must be defined
                in the modelDescription.xml to be valid an visible to the importer, but
                can otherwise be any string. A number of categories are predefined by the
                FMI standard and included by default in UniFMU:
                - logStatusWarning
                - logStatusDiscard
                - logStatusError
                - logStatusFatal
                - logEvents
    """
    def log(self, message, status, category = "logEvents"):
        # Feel free to expand on the functionality of the function.
        # The model will be informed of whether or not to output logging and
        # what categories to log through a call to fmi3SetDebugLogging().
        # The UniFMU layer already handles filtering of messages so that
        # the FMU importer only receives logging events that is interested in,
        # but if you want to filter before sending the events to the UniFMU 
        # layer to save on message bandwidth, feel free to do so.
        
        # Removing the line below will break logging.
        self._log_callback(status, category, message)

    # ================= Helpers =================

    def _set_value(self, references, values):
        for r, v in zip(references, values):
            if (r in self.clocked_variables or r in self.tunable_parameters):
                if (self.state == FMIState.FMIEventModeState or self.state == FMIState.FMIInitializationModeState):
                    pass
                else:
                    return Fmi3Status.error
            elif (r in self.tunable_structural_parameters):
                if (self.state == FMIState.FMIConfigurationModeState or self.state == FMIState.FMIReconfigurationModeState or self.state == FMIState.FMIInitializationModeState):
                    pass
                else:
                    return Fmi3Status.error
            elif (r in self.parameters):
                if (self.state == FMIState.FMIInitializationModeState):
                    pass
                else:
                    return Fmi3Status.error
            setattr(self, self.all_references[r], v)
        return Fmi3Status.ok

    def _get_value(self, references):

        values = []
        for r in references:
            if r in self.clocked_variables:
                if not ((self.state == FMIState.FMIEventModeState) or (self.state == FMIState.FMIInitializationModeState)):
                    return Fmi3Status.error
            values.append(getattr(self, self.all_references[r]))

        return Fmi3Status.ok, values

    def _update_outputs(self):
        self.float32_c = self.float32_a + self.float32_b
        self.float64_c = self.float64_a + self.float64_b
        self.int8_c = self.int8_a + self.int8_b
        self.uint8_c = self.uint8_a + self.uint8_b
        self.int16_c = self.int16_a + self.int16_b
        self.uint16_c = self.uint16_a + self.uint16_b
        self.int32_c = self.int32_a + self.int32_b
        self.uint32_c = self.uint32_a + self.uint32_b
        self.int64_c = self.int64_a + self.int64_b
        self.uint64_c = self.uint64_a + self.uint64_b
        self.boolean_c = self.boolean_a or self.boolean_b
        self.string_c = self.string_a + self.string_b
        self.binary_c = bytes(a ^ b for a, b in zip(self.binary_a, self.binary_b))
    
    def _update_clocks(self):
        self.clock_c = self.clock_a and self.clock_b

    def _update_clocked_outputs(self):
        self.clocked_variable_c += self.clocked_variable_a + self.clocked_variable_b


class Fmi3Status():
    """
    Represents the status of an FMI3 FMU or the results of function calls.

    Values:
        * ok: all well
        * warning: an issue has arisen, but the computation can continue.
        * discard: an operation has resulted in invalid output, which must be discarded
        * error: an error has ocurred for this specific FMU instance.
        * fatal: an fatal error has ocurred which has corrupted ALL FMU instances.
    """

    ok = 0
    warning = 1
    discard = 2
    error = 3
    fatal = 4

class FMIState(IntFlag):
    FMIStartAndEndState         = 1 << 0,
    FMIInstantiatedState        = 1 << 1,
    FMIInitializationModeState  = 1 << 2,
    FMITerminatedState          = 1 << 3,
    FMIConfigurationModeState   = 1 << 4,
    FMIReconfigurationModeState = 1 << 5,
    FMIEventModeState           = 1 << 6,
    FMIContinuousTimeModeState  = 1 << 7,
    FMIStepModeState            = 1 << 8,
    FMIClockActivationMode      = 1 << 9


if __name__ == "__main__":
    m = Model()

    assert m.float32_a == 0.0
    assert m.float32_b == 0.0
    assert m.float32_c == 0.0
    assert m.float64_a == 0.0
    assert m.float64_b == 0.0
    assert m.float64_c == 0.0
    assert m.int8_a == 0
    assert m.int8_b == 0
    assert m.int8_c == 0
    assert m.uint8_a == 0
    assert m.uint8_b == 0
    assert m.uint8_c == 0
    assert m.int16_a == 0
    assert m.int16_b == 0
    assert m.int16_c == 0
    assert m.uint16_a == 0
    assert m.uint16_b == 0
    assert m.uint16_c == 0
    assert m.int32_a == 0
    assert m.int32_b == 0
    assert m.int32_c == 0
    assert m.uint32_a == 0
    assert m.uint32_b == 0
    assert m.uint32_c == 0
    assert m.int64_a == 0
    assert m.int64_b == 0
    assert m.int64_c == 0
    assert m.uint64_a == 0
    assert m.uint64_b == 0
    assert m.uint64_c == 0
    assert m.boolean_a == False
    assert m.boolean_b == False
    assert m.boolean_c == False
    assert m.string_a == ""
    assert m.string_b == ""
    assert m.string_c == ""
    assert m.binary_a == 0
    assert m.binary_b == 0
    assert m.binary_c == 0
    assert m.clock_a == False
    assert m.clock_b == False
    assert m.clock_c == False

    m.float32_a = 1.0
    m.float32_b = 2.0
    m.float64_a = 1.0
    m.float64_b = 2.0
    m.int8_a = 1
    m.int8_b = 2
    m.uint8_a = 1
    m.uint8_b = 2
    m.int16_a = 1
    m.int16_b = 2
    m.uint16_a = 1
    m.uint16_b = 2
    m.int32_a = 1
    m.int32_b = 2
    m.uint32_a = 1
    m.uint32_b = 2
    m.int64_a = 1
    m.int64_b = 2
    m.uint64_a = 1
    m.uint64_b = 2
    m.boolean_a = True
    m.boolean_b = False
    m.string_a = "Hello "
    m.string_b = "World!"
    m.binary_a = 1
    m.binary_b = 2

    assert m.fmi3DoStep(0.0, 1.0, False)[0] == Fmi3Status.ok

    assert m.float32_c == 3.0
    assert m.float64_c == 3.0
    assert m.int8_c == 3
    assert m.uint8_c == 3
    assert m.int16_c == 3
    assert m.uint16_c == 3
    assert m.int32_c == 3
    assert m.uint32_c == 3
    assert m.int64_c == 3
    assert m.uint64_c == 3
    assert m.boolean_c == True
    assert m.string_c == "Hello World!"
    assert m.binary_c == 3
