import logging
from collections.abc import Sequence

import matlab.engine

from fmi2 import Fmi2Status

logger = logging.getLogger("MatlabInterface")

class MatlabInterface:

    @staticmethod
    def get_real(references, eng):
        margs = matlab.int64(references)
        status, values = eng.get_real(margs, nargout=2)

        result = None
        if not isinstance(values, Sequence):
            result = [float(values)]
        else:
            result = [float(v) for v in values[0]]

        return status, result

    @staticmethod
    def set_real(references, values, eng):
        return eng.set_real(matlab.int64(references), matlab.double(values))

    @staticmethod
    def get_string(references, eng):
        status, values = eng.get_string(matlab.int64(references), nargout=2)

        result = None
        if isinstance(values, str):
            result = [str(values)]
        else:
            result = [str(v) for v in values]

        return status, result

    @staticmethod
    def set_string(references, values, eng):
        return eng.set_string(matlab.int64(references), values)

    @staticmethod
    def get_boolean(references, eng):
        status, values = eng.get_boolean(matlab.int64(references), nargout=2)

        result = None
        if not isinstance(values, Sequence):
            result = [bool(values)]
        else:
            result = [bool(v) for v in values[0]]

        return status, result

    @staticmethod
    def set_boolean(references, values, eng):
        return eng.set_boolean(matlab.int64(references), matlab.logical(values))

    @staticmethod
    def get_integer(references, eng):
        status, values = eng.get_integer(matlab.int64(references), nargout=2)

        result = None
        if not isinstance(values, Sequence):
            result = [int(values)]
        else:
            result = [int(v) for v in values[0]]

        return status, result

    @staticmethod
    def set_integer(references, values, eng):
        return eng.set_integer(matlab.int64(references), matlab.int64(values))

    @staticmethod
    def set_debug_logging(categories, logging_on, eng) -> int:
        return eng.set_debug_logging(categories, logging_on)

    @staticmethod
    def setup_experiment(
            start_time: float, stop_time=None, tolerance=None, eng=None
    ) -> int:
        return eng.setup_experiment(start_time, stop_time, -1.0)

    @staticmethod
    def enter_initialization_mode(eng) -> int:
        return Fmi2Status.ok

    @staticmethod
    def exit_initialization_mode(eng) -> int:
        return Fmi2Status.ok

    @staticmethod
    def terminate(eng) -> int:
        return Fmi2Status.ok

    @staticmethod
    def reset(eng) -> int:
        return Fmi2Status.ok

    @staticmethod
    def serialize(eng) -> bytes:
        status, values = eng.serialize(nargout=2)

        if not isinstance(values, Sequence):
            result = [values]
        else:
            result = [v for v in values[0]]

        return status, bytes(result)

    @staticmethod
    def deserialize(state: bytes, eng):
        return eng.deserialize(matlab.uint8(state), nargout=1)

    @staticmethod
    def do_step(
            current_time: float, step_size: float, no_step_prior: bool, eng
    ) -> int:
        status = eng.do_step(current_time, step_size, no_step_prior, nargout=1)
        return status
