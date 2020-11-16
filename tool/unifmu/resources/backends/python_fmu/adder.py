from fmi2 import FMU


class Adder(FMU):
    def __init__(self) -> None:

        self.real_a = 0.0
        self.real_b = 0.0

        self.integer_a = 0
        self.integer_b = 0

        self.boolean_a = False
        self.boolean_b = False

        self.string_a = ""
        self.string_b = ""

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
