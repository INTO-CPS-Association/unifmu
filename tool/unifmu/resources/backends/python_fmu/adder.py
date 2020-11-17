import pickle

from fmi2 import Fmi2FMU


class Adder(Fmi2FMU):
    def __init__(self) -> None:

        self.real_a = 0.0
        self.real_b = 0.0

        self.integer_a = 0
        self.integer_b = 0

        self.boolean_a = False
        self.boolean_b = False

        self.string_a = ""
        self.string_b = ""

    def serialize(self) -> bytes:
        return pickle.dumps(
            (
                self.real_a,
                self.real_b,
                self.real_c,
                self.integer_a,
                self.integer_b,
                self.boolean_a,
                self.boolean_c,
                self.string_a,
                self.string_b,
            )
        )

    def deserialize(self, bytes):
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

    @property
    def real_c(self):
        return self.real_a + self.real_b

    @property
    def integer_c(self):
        return self.integer_a + self.integer_b

    @property
    def boolean_c(self):
        return self.boolean_a and self.boolean_b

    @property
    def string_c(self):
        return self.string_a + self.string_b
