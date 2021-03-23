from typing import (
    Iterable,
    List,
    Literal,
    Optional,
    Set,
    Union,
    TypeVar,
    Type,
    get_args,
)


class Fmi2CoSim:
    def __init__(self) -> None:
        pass


T = TypeVar("T", float, int, bool, str)
_variability_all = Literal["constant", "fixed", "tunable", "discrete", "continuous"]
_causality_all = Literal["exact", "approx", "calculated"]


class Fmi2Variable:
    def __init__(
        self,
        name: str,
        type: Type[T],
        causality: Literal[
            "parameter",
            "calculated_parameter",
            "input",
            "output",
            "local",
            "independent",
        ],
        variability: _variability_all,
        initial: _causality_all,
        description: str = None,
        declared_type: str = None,
        quantity: str = None,
        unit: str = None,
        display_unit: str = None,
        relative_quantity: bool = None,
        min: T = None,
        max: T = None,
        nominal: T = None,
        unbounded: bool = None,
        start: T = None,
        derivative: int = None,
        reinit: bool = None,
    ) -> None:

        if type not in get_args(T):
            raise ValueError(
                f"Invalid data type for variable: {name}, choices are: '{get_args(T)}', got: {type}"
            )
        # TODO add validation logic here

        self.name = name
        self.type = type
        self.description = description
        self.causality = causality
        self.variability = variability
        self.initial = initial
        self.declared_type = declared_type
        self.quantity = quantity
        self.unit = unit
        self.display_unit = display_unit
        self.relative_quantity = relative_quantity
        self.min = min
        self.max = max
        self.nominal = nominal
        self.unbounded = unbounded
        self.start = start
        self.derivative = derivative
        self.reinit = reinit


class Fmi2FMU:
    def __init__(
        self,
        model_name: str,
        fmi2_version: str = None,
        guid: str = None,
        description: str = None,
        author: str = None,
        version: str = None,
        copyright: str = None,
        license: str = None,
        generation_tool: str = None,
        generation_data_and_time: str = None,
        variable_naming_convention: Literal["flat", "structured"] = None,
        number_of_event_indicators=None,
    ) -> None:
        self._variables: List[Fmi2Variable] = []

    def define_model_exchange(self):
        raise NotImplementedError("currently only co-sim is supported")

    def define_cosimulation(
        self,
        model_identifier: str,
        needs_execution_tool: bool = None,
        can_handle_variable_communication_step_size: bool = None,
        can_interpolate_inputs: bool = None,
        max_output_derivative_order: int = None,
        can_run_asynchronuously: bool = None,
        can_be_instantiated_only_once_per_process: bool = None,
        can_not_use_memory_management_functions: bool = None,
        can_get_and_set_fmu_state: bool = None,
        can_serialize_fmu_state: bool = None,
        provides_directional_derivative: bool = None,
    ):
        pass

    def define_log_categories(self, categories: Iterable[str]):
        pass

    def define_unit_definitions(self):
        pass

    def define_default_experiment(self):
        pass

    def define_vendor_annotations(self):
        pass

    def _add_variable(
        self,
        name: str,
        type: Type[T],
        causality: Literal[
            "parameter",
            "calculated_parameter",
            "input",
            "output",
            "local",
            "independent",
        ],
        variability: Literal["constant", "fixed", "tunable", "discrete", "continuous"],
        initial: Literal["exact", "approx", "calculated"],
        declared_type=None,
        quantity: str = None,
        unit: str = None,
        display_unit: str = None,
        relative_quantity: bool = None,
        min: T = None,
        max: T = None,
        nominal: T = None,
        unbounded: bool = None,
        start: T = None,
        derivative: int = None,
        reinit: bool = None,
        description: str = None,
    ):
        self._variables.append(
            Fmi2Variable(
                name=name,
                type=type,
                causality=causality,
                variability=variability,
                initial=initial,
                declared_type=declared_type,
                quantity=quantity,
                unit=unit,
                display_unit=display_unit,
                relative_quantity=relative_quantity,
                min=min,
                max=max,
                nominal=nominal,
                unbounded=unbounded,
                start=start,
                derivative=derivative,
                reinit=reinit,
                description=description,
            )
        )

    def add_parameter(
        self,
        name: str,
        type: Type[T],
        variability: Literal["fixed", "tunable"],
        start: T,
        description: str = None,
    ):
        self._add_variable(
            name=name,
            type=type,
            variability=variability,
            causality="parameter",
            initial="exact",
            start=start,
            description=description,
        )

    def add_calculated_parameter(
        self,
        name: str,
        type: Type[T],
        variability: Literal["fixed", "tunable"],
        initial=Literal["approx", "calculated"],
        description: str = None,
    ):
        pass

    def add_input(
        self,
        name: str,
        type: Type[T],
        variability: Literal["discrete", "continuous"],
        start: T = None,
        min: T = None,
        max: T = None,
        description: str = None,
    ):
        pass

    def add_output(
        self,
        name: str,
        type: Type[T],
        variability: Literal["constant", "discrete", "continuous"],
        initial: Literal["exact", "calculated"],
        start: T = None,
        description: str = None,
    ):
        pass

    def add_local_variable(
        self,
        name: str,
        type: Type[T],
        variability: Literal["constant", "fixed", "tunable", "discrete", "continuous"],
        initial: Literal["exact", "approx", "calculated"],
        description: str = None,
    ):
        pass

    def add_independent_variable(
        self, name: str, description: str = None,
    ):
        pass


class Fmi3FMU:
    pass


if __name__ == "__main__":

    cosim = Fmi2CoSim()

    adder = Fmi2FMU("adder")
    adder.add_input("real_a", type(float), "continuous", start=0.0)
    adder.add_input("real_b", type(float), "continuous", start=0.0)
    adder.add_output(
        "real_c", type(float), variability="continuous", initial="calculated",
    )

    adder.add_input("integer_c", int, "discrete")
    adder.add_input("integer_b", int, "discrete")
    adder.add_output(
        "integer_c", int, "discrete", initial="calculated",
    )

    adder.add_input("boolean_c", bool, "discrete")
    adder.add_input("boolean_b", bool, "discrete")
    adder.add_output(
        "boolean_c", bool, "discrete", initial="calculated",
    )

    adder.add_input("integer_c", str, "discrete")
    adder.add_input("integer_b", str, "discrete")
    adder.add_output(
        "integer_c", str, "discrete", initial="calculated",
    )

    # adder.export(outdir="...", language="python")

