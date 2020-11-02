from fmi2 import FMU


class Adder(FMU):
    def __init__(self) -> None:

        self.a = 0.0
        self.b = 0.0
        self.s = self.a + self.b
