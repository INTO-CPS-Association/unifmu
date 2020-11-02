from fmi2 import FMU


class Adder(FMU):
    def __init__(self) -> None:

        self.a = 0.0
        self.b = 0.0

    @property
    def s(self):
        return self.a + self.b
