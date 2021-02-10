import pickle
from typing import Tuple

from fmi2 import Fmi2FMU
import sys
from fmi2 import Fmi2FMU, Fmi2Status

class Adder(Fmi2FMU):
    def __init__(self,reference_to_attr=None) -> None:
        super().__init__(reference_to_attr)
        self.real_a = 0.0
        self.real_b = 0.0

        self.integer_a = 0
        self.integer_b = 0
        
        self.boolean_a = False
        self.boolean_b = False

        self.string_a = ""
        self.string_b = ""

        self._update_outputs()

        

    def __repr__(self):
        return "Adder"

    def serialize(self):

        bytes = pickle.dumps(
            (
                self.real_a,
                self.real_b,
                self.integer_a,
                self.integer_b,
                self.boolean_a,
                self.boolean_b,
                self.string_a,
                self.string_b,
            )
        )
        return Fmi2Status.ok, bytes

    def deserialize(self, bytes) -> int:
        (
            real_a,
            real_b,
            integer_a,
            integer_b,
            boolean_a,
            boolean_b,
            string_a,
            string_b,
        ) = pickle.loads(bytes)
        self.real_a = real_a
        self.real_b = real_b
        self.integer_a = integer_a
        self.integer_b = integer_b
        self.boolean_a = boolean_a
        self.boolean_b = boolean_b
        self.string_a = string_a
        self.string_b = string_b

        return Fmi2Status.ok

    def _update_outputs(self):
        self.real_c = self.real_a + self.real_b
        self.integer_c = self.integer_a + self.integer_b
        self.boolean_c = self.boolean_a and self.boolean_b
        self.string_c = self.string_a + self.string_b

    def do_step(self, current_time, step_size, no_step_prior):
        
        self._update_outputs()

        return Fmi2Status.ok
        

