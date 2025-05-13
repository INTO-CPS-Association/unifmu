import pickle


class Model:
    def __init__(self) -> None:

        self.real_a = 0.0
        self.real_b = 0.0
        self.integer_a = 0
        self.integer_b = 0
        self.boolean_a = False
        self.boolean_b = False
        self.string_a = ""
        self.string_b = ""

        self.reference_to_attribute = {
            0: "real_a",
            1: "real_b",
            2: "real_c",
            3: "integer_a",
            4: "integer_b",
            5: "integer_c",
            6: "boolean_a",
            7: "boolean_b",
            8: "boolean_c",
            9: "string_a",
            10: "string_b",
            11: "string_c",
        }

        self.fmi2Reset()

    # ================= FMI2 =================

    def fmi2DoStep(
        self, current_time, step_size, no_set_fmu_state_prior_to_current_point
    ):
        self._update_outputs()
        return Fmi2Status.ok

    def fmi2SetDebugLogging(self, categories, logging_on):
        return Fmi2Status.ok

    def fmi2EnterInitializationMode(self):
        return Fmi2Status.ok

    def fmi2ExitInitializationMode(self):
        self._update_outputs()
        return Fmi2Status.ok

    def fmi2SetupExperiment(self, start_time, stop_time, tolerance):
        return Fmi2Status.ok

    def fmi2Terminate(self):
        return Fmi2Status.ok

    def fmi2Reset(self):
        self.real_a = 0.0
        self.real_b = 0.0
        self.integer_a = 0
        self.integer_b = 0
        self.boolean_a = False
        self.boolean_b = False
        self.string_a = ""
        self.string_b = ""
        self._update_outputs()

        return Fmi2Status.ok

    def fmi2SerializeFmuState(self):
        bytes = pickle.dumps(
            (
                self.real_a,
                self.real_b,
                self.real_c,
                self.integer_a,
                self.integer_b,
                self.integer_c,
                self.boolean_a,
                self.boolean_b,
                self.boolean_c,
                self.string_a,
                self.string_b,
                self.string_c,
            )
        )
        return Fmi2Status.ok, bytes

    def fmi2DeserializeFmuState(self, bytes):
        (
            real_a,
            real_b,
            real_c,
            integer_a,
            integer_b,
            integer_c,
            boolean_a,
            boolean_b,
            boolean_c,
            string_a,
            string_b,
            string_c,
        ) = pickle.loads(bytes)
        self.real_a = real_a
        self.real_b = real_b
        self.real_c = real_c
        self.integer_a = integer_a
        self.integer_b = integer_b
        self.integer_c = integer_c
        self.boolean_a = boolean_a
        self.boolean_b = boolean_b
        self.boolean_c = boolean_c
        self.string_a = string_a
        self.string_b = string_b
        self.string_c = string_c

        return Fmi2Status.ok

    def fmi2GetReal(self, references):
        return self._get_value(references)

    def fmi2GetInteger(self, references):
        return self._get_value(references)

    def fmi2GetBoolean(self, references):
        return self._get_value(references)

    def fmi2GetString(self, references):
        return self._get_value(references)

    def fmi2SetReal(self, references, values):
        return self._set_value(references, values)

    def fmi2SetInteger(self, references, values):
        return self._set_value(references, values)

    def fmi2SetBoolean(self, references, values):
        return self._set_value(references, values)

    def fmi2SetString(self, references, values):
        return self._set_value(references, values)

    # ================= Helpers =================

    def _set_value(self, references, values):

        for r, v in zip(references, values):
            setattr(self, self.reference_to_attribute[r], v)

        return Fmi2Status.ok

    def _get_value(self, references):

        values = []

        for r in references:
            values.append(getattr(self, self.reference_to_attribute[r]))

        return Fmi2Status.ok, values

    def _update_outputs(self):
        self.real_c = self.real_a + self.real_b
        self.integer_c = self.integer_a + self.integer_b
        self.boolean_c = self.boolean_a or self.boolean_b
        self.string_c = self.string_a + self.string_b


class Fmi2Status:
    """
    Represents the status of an FMI2 FMU or the results of function calls.

    Values:
        * ok: all well
        * warning: an issue has arisen, but the computation can continue.
        * discard: an operation has resulted in invalid output, which must be discarded
        * error: an error has ocurred for this specific FMU instance.
        * fatal: an fatal error has ocurred which has corrupted ALL FMU instances.
        * pending: indicates that the FMu is doing work asynchronously, which can be retrived later.

    Notes:
        FMI section 2.1.3

    """

    ok = 0
    warning = 1
    discard = 2
    error = 3
    fatal = 4
    pending = 5


if __name__ == "__main__":
    m = Model()

    assert m.real_a == 0.0
    assert m.real_b == 0.0
    assert m.real_c == 0.0
    assert m.integer_a == 0
    assert m.integer_b == 0
    assert m.integer_c == 0
    assert m.boolean_a == False
    assert m.boolean_b == False
    assert m.boolean_c == False
    assert m.string_a == ""
    assert m.string_b == ""
    assert m.string_c == ""

    m.real_a = 1.0
    m.real_b = 2.0
    m.integer_a = 1
    m.integer_b = 2
    m.boolean_a = True
    m.boolean_b = False
    m.string_a = "Hello "
    m.string_b = "World!"

    assert m.fmi2DoStep(0.0, 1.0, False) == Fmi2Status.ok

    assert m.real_c == 3.0
    assert m.integer_c == 3
    assert m.boolean_c == True
    assert m.string_c == "Hello World!"
