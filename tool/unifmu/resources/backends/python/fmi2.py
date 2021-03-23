from typing import Any, List, Tuple
import logging

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


class Fmi2StatusKind:
    """Defines the different types of statuses the master can inquire the slave about, see p.104
    
    These are used for async related functionality of FMI2.

    Values:
        * do_step_status: request the status of the step function. If not completed fmi2Pending is returned,
                        if complete the status of the step function is returned. 
        * pending_status: request a string description of describing the progress of the step function.
        * last_successfull_time: returns the simulation time of the last successfull simulation step.
        * terminated: ask the slave if it wants to terminate the simulation. This can be called after the 
                        step function returns the discard status.
    """

    do_step_status = 0
    pending_status = 1
    last_successfull_time = 2
    terminated = 3


class Fmi2FMU:
    """Base class for FMUs implemented using UniFMU's Python backend.
    
    Deriving from this class provides dummy implementation for FMI2 function, 
    eliminating the need to implement functionality not needed by the FMU.
    An additional ulility of the base class is to provide function-prototypes
    which the an IDE may use to provide code completion hints to the author.

    The behavior of the FMU can be implemented by overwriting these methods.
    """

    def __init__(self, reference_to_attr=None) -> None:
        self.reference_to_attr = reference_to_attr
        self.logger = logging.getLogger("Python FMI backend")
        self.logger.setLevel(logging.DEBUG)
        formatter = logging.Formatter("%(levelname)s: %(message)s")
        ch = logging.StreamHandler()
        ch.setFormatter(formatter)
        self.logger.addHandler(ch)

    # --------- common --------------
    def set_debug_logging(self, categories, logging_on) -> int:
        return Fmi2Status.ok

    def setup_experiment(
        self, start_time: float, stop_time=None, tolerance=None
    ) -> int:
        return Fmi2Status.ok

    def enter_initialization_mode(self) -> int:
        """Informs the FMU to enter initialization mode. 
        Before this all inputs with 'initial ∈ {exact, approx}', have been set by the tool.
        
        At this stage all outputs of 'initial ∈ {calculated}' can be assigned.
        """
        return Fmi2Status.ok

    def exit_initialization_mode(self) -> int:
        """Informs the fmu to exit initialziation mode."""
        return Fmi2Status.ok

    def terminate(self) -> int:
        """Informs the FMU that the simulation has finished, after this the final values of the FMU can be enquired by the tool.
        
        Note that termination is not the same as the FMU be freed; the fmu may be reset and used for another simulation run.
        As such it may be sensible to preserve expensive to construct resources, that would otherwise have to be recreated.
        
        If you need to add destructor like functionality, instead overwrite the objects __del__ method, which is invoked when the 
        FMU is finally dropped.
        """
        return Fmi2Status.ok

    def reset(self) -> int:
        """Restores the FMU to the same state as it would be after instantiation"""
        return Fmi2Status.ok

    # getters and setters implemented in launch.py
    def get_xxx(self, references):
        if not self.reference_to_attr:
            raise RuntimeError("Unable to get variables using value references. Init was called without references_to_attr.")
        try:
            attributes = [self.reference_to_attr[vref] for vref in references]
            values = [getattr(self, a) for a in attributes]
            logging.debug(f"read vref: {references} with value: {values}")
            return Fmi2Status.ok, values
        except AttributeError as e:
            logging.error(f"Unable to read variable from slave, the variable is not declared as an attribute of the Python object", exc_info=True)
            return Fmi2Status.error, None

    def set_xxx(self, references, values):
        if not self.reference_to_attr:
            raise RuntimeError("Unable to get variables using value references. Init was called without references_to_attr.")
        try:
            logging.debug(f"setting {references} to {values}")
            attributes = [self.reference_to_attr[vref] for vref in references]
            for a, v in zip(attributes, values):
                setattr(self, a, v)
            return Fmi2Status.ok
        except AttributeError as e:
            logging.error(f"Unable to set variable of slave, the variable is not declared as an attribute of the Python object", exc_info=True)
            return Fmi2Status.error

    def serialize(self) -> bytes:
        """Convert the state of the FMU into a sequences of bytes which can later be used to roll-back the state of the FMU to that point"""
        raise NotImplementedError()

    def deserialize(self, state: bytes):
        """Restore a FMU to the state recoreded by the serialize method"""
        raise NotImplementedError()

    def get_directional_derivative(
        self,
        references_unknown: List[int],
        references_known: List[int],
        values_known: List[float],
    ) -> List[float]:
        raise NotImplementedError()

    # --------- co-sim --------------

    def set_input_derivatives(self):
        raise NotImplementedError()

    def get_output_derivatives(self):
        raise NotImplementedError()

    def do_step(
        self, current_time: float, step_size: float, no_step_prior: bool
    ) -> int:
        return Fmi2Status.ok

    def cancel_step(self) -> int:
        raise NotImplementedError()

    def get_xxx_status(self, kind: int) -> Tuple[int, Any]:
        """Inquire about the status of an async FMU's step methods progress."""
        raise NotImplementedError()
