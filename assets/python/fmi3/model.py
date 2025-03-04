import pickle


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
    ) -> None:
        self.instance_name = instance_name
        self.instantiation_token = instantiation_token
        self.resource_path = resource_path
        self.visible = visible
        self.logging_on = logging_on
        self.event_mode_used = event_mode_used
        self.early_return_allowed = early_return_allowed
        self.required_intermediate_variables = required_intermediate_variables
     
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
            1001: "clock_a",
            1002: "clock_b",
            1003: "clock_c",
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
        }

        self.clock_reference_to_interval = {
            1001: 1.0,
        }

        self.all_references = {**self.tunable_parameters,**self.reference_to_attribute}

        self.fmi3Reset()

    # ================= FMI3 =================

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

    def fmi3EnterInitializationMode(
            self,
            tolerance_defined: bool,
            tolerance: float,
            start_time: float,
            stop_time_defined: bool,
            stop_time: float
    ):
        return Fmi3Status.ok

    def fmi3ExitInitializationMode(self):
        self._update_outputs()
        return Fmi3Status.ok

    def fmi3EnterEventMode(self):
        return Fmi3Status.ok

    def fmi3EnterStepMode(self):
        return Fmi3Status.ok
    
    def fmi3EnterConfigurationMode(self):
        self.configuration_mode = True
        return Fmi3Status.ok

    def fmi3ExitConfigurationMode(self):
        self.configuration_mode = False
        return Fmi3Status.ok

    def fmi3Terminate(self):
        return Fmi3Status.ok

    def fmi3Reset(self):
        self.configuration_mode = False
        self.float32_a = 0.0
        self.float32_b = 0.0
        self.float64_a = 0.0
        self.float64_b = 0.0
        self.int8_a = 0
        self.int8_b = 0
        self.uint8_a = 0
        self.uint8_b = 0
        self.int16_a = 0
        self.int16_b = 0
        self.uint16_a = 0
        self.uint16_b = 0
        self.int32_a = 0
        self.int32_b = 0
        self.uint32_a = 0
        self.uint32_b = 0
        self.int64_a = 0
        self.int64_b = 0
        self.uint64_a = 0
        self.uint64_b = 0
        self.boolean_a = False
        self.boolean_b = False
        self.string_a = ""
        self.string_b = ""
        self.binary_a = bytes([0])
        self.binary_b = bytes([0])
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
        self.clock_a = False
        self.clock_b = False
        self.clock_reference_to_interval = {
            1001: 1.0,
        }

        self._update_outputs()
        self._update_clocks()
        return Fmi3Status.ok

    def fmi3SerializeFmuState(self):

        bytes = pickle.dumps(
            (
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
                self.clock_a,
                self.clock_b,
            )
        )
        return Fmi3Status.ok, bytes

    def fmi3DeserializeFmuState(self, bytes: bytes):
        (
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
            clock_a,
            clock_b,
        ) = pickle.loads(bytes)
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
        self.clock_a = clock_a
        self.clock_b = clock_b

        self._update_outputs()
        self._update_clocks()

        return Fmi3Status.ok

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

    def fmi3SetBinary(self, value_references, values):
        return self._set_value(value_references, values)

    def fmi3SetClock(self, value_references, values):
        status = self._set_value(value_references, values)
        self._update_clocks()
        return status

    def fmi3UpdateDiscreteStates(self):
        status = Fmi3Status.ok
        discrete_states_need_update = False
        terminate_simulation = False
        nominals_continuous_states_changed = False
        values_continuous_states_changed = False
        next_event_time_defined = True
        next_event_time = 1.0
        return (status, discrete_states_need_update, terminate_simulation, nominals_continuous_states_changed,
                values_continuous_states_changed, next_event_time_defined, next_event_time)

    # ================= Helpers =================

    def _set_value(self, references, values):
        if (self.configuration_mode):
            for r, v in zip(references, values):
                setattr(self, self.all_references[r], v)
        else:
            for r, v in zip(references, values):
                if r in self.tunable_parameters:
                    return Fmi3Status.error
                setattr(self, self.reference_to_attribute[r], v)

        return Fmi3Status.ok

    def _get_value(self, references):

        values = []
        for r in references:
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


class Fmi3Status:
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
