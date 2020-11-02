from typing import List, Tuple


class Fmi2Status:
    """Represents the status of the FMU or the results of function calls.

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


class FMU:
    def __init__(self) -> None:
        pass

    def set_debug_logging(self, categories, logging_on):
        return Fmi2Status.ok

    def do_step(self, current_time: float, step_size: float, no_step_prior: bool):
        return Fmi2Status.ok

    def setup_experiment(self, tolerance=None, start_time=None, stop_time=None):
        return Fmi2Status.ok

    def enter_initialization_mode(self):
        return Fmi2Status.ok

    def exit_initialization_mode(self):
        return Fmi2Status.ok

    def terminate(self):
        return Fmi2Status.ok

    def reset(self):
        return Fmi2Status.ok

    def get_xxx(self, references):
        try:
            # variables are stored in order of their value reference
            attributes = [self.variables[i].name for i in references]

            values = [getattr(self, a) for a in attributes]

            return (Fmi2Status.ok, values)

        except Exception:
            self.log_err(
                "An exception was raised when reading variables from slave",
                exc_info=True,
            )
            return (Fmi2Status.error, None)

    def set_xxx(self, references, values):
        try:
            # variables are stored in order of their value reference
            attributes = [self.variables[i].name for i in references]

            for a, v in zip(attributes, values):
                setattr(self, a, v)

        except Exception:

            return Fmi2Status.error

        else:
            return Fmi2Status.ok
