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
