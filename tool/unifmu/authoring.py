from collections import defaultdict
from typing import (
    Dict,
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
from os import PathLike


class Fmi2CoSim:
    def __init__(self) -> None:
        pass


T = TypeVar("T", float, int, bool, str)
_variability_all = Literal["constant", "fixed", "tunable", "discrete", "continuous"]
_causality_all = Literal["exact", "approx", "calculated"]


class Fmi2Variable:
    """FMI2 variable representing part of the external interface of the model such as
    an input, outpur or parameter of the model. 

    Each variable is associated with the following:

    causality, the type of the variable. Choices are:
    - parameter
    """

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
        model_name,
        language,
        backend,
        fmi2_version=None,
        guid=None,
        description=None,
        author=None,
        version=None,
        copyright=None,
        license=None,
        generation_tool=None,
        generation_date_and_time=None,
        variable_naming_convention=None,
        number_of_event_indicators=None,
        supports_cosim=None,
        support_model_exchange=None,
    ) -> None:
        """Creates an FMU 

        Supported languages: python, csharp, java, c, cpp, rust

        Backends:
        - grpc, supported by python, csharp, java
        - schemaless, supported by python
        - abi: application binary interface, supported by c, cpp, rust

        An FMU can be created as follows:

        >>> fmu = Fmi2FMU("adder", "python", "grpc")
        ... adder = Fmi2FMU(model_name="", language="python", backend="grpc")
        ... adder.add_input("real_a", "real", "continuous", start=0.0)
        ... adder.add_input("real_b", "real", "continuous", start=0.0)
        ... adder.add_output(
            "real_c", "real", variability="continuous", initial="calculated",
            )

        """

        # common
        self.model_name = model_name
        self.fmi2_version = fmi2_version
        self.guid = guid
        self.description = description
        self.author = author
        self.version = version
        self.copyright = copyright
        self.license = license
        self.generation_tool = generation_tool
        self.generation_date_and_time = generation_date_and_time
        self.variable_naming_convetion = variable_naming_convention
        self.number_of_event_indicators = number_of_event_indicators
        self._model_variables: List[Fmi2Variable] = []
        self.dependencies = {}
        self.log_categories = []
        self.log_categories = defaultdict(dict)
        self.units = defaultdict(dict)
        # model exchange

        # cosimulation

        self.needs_execution_tool = (
            True if language in {"python", "java", "csharp"} else False
        )

        self.model_identifier = "unifmu"
        self.can_handle_variable_communication_step_size = True
        self.can_interpolate_inputs = True
        self.max_output_derivative_order = 0
        self.can_run_asynchronuously = False
        self.can_be_instantiated_only_once_per_process = False
        self.can_not_use_memory_management_functions = True
        self.can_get_and_set_fmu_state = True
        self.can_serialize_fmu_state = True
        self.provides_directional_derivative = False

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

    def declare_log_category(self, category_name: str, description=None):
        """Declares a new category of messages that can be printed by the FMU.
        The classification in categories is used by the environment to filter
        the number of messages shown to the user.

        See section: 2.2.4
        """

        if category_name in self.log_categories:
            raise ValueError(
                f"The category '{category_name}' has already been declared."
            )

        self.log_categories[category_name][description] = description

    def declare_base_unit(
        self, name: str, base_unit=None, factor: float = 1.0, offset: float = 0.0
    ):
        """Declares a new unit.

        A unit can be defined using it's name:
        >>> fmu.define_unit("rad/s")
        
        Alternatively, a base unit expressed as the expornents of SI units may be defined:
        >>> fmu.define_unit("rad/s", {"s" : -1, "rad" : 1})

        The list of available 
        >>> fmu.declare_base_unit(
            "complex unit",
            {"kg": 0, "m": 0, "s": 0, "A": 0, "K": "0", "cd": 0, "rad": 0},
        )

        See section: 2.2.2
        """
        #
        # If no units are defined, element <UnitDefinitions> must not be present. If 1 or more units are defined, this element
        # must be present.

        self.units[name]["base_unit"] = base_unit

    def declare_display_unit(
        self,
        base_unit: str,
        display_name: str,
        factor: float = None,
        offset: float = None,
    ):
        """ Declares an alias for an existing unit defined by a scaling and offset.
        The intention of this functionality is to provide an human readable alternative
        to raw SI values.

        For example a 'rotations' per minut can be defined as an 'display' unit of 'rad/s':

        >>> fmu.define_base_unit("rad/s", {"s" : -1, "rad" : 1}, 0.0)
        >>> fmu.define_display_unit("rad/s","rpm", 9.549296585513721, 0.0)

        Multiple Display values are allowed for a single base unit:
        >>> fmu.define_base_unit("deg/s", rad/s, 57.29577951308232)
        """

        if base_unit not in self.units:
            raise ValueError(f"Base unit '{base_unit} is not defined.'")

        self.units[base_unit]["display_name"] = (display_name, factor, offset)

    def declare_default_experiment(
        self,
        start_time: float = None,
        stop_time: float = None,
        tolerance: float = None,
        step_size: float = None,
    ):
        """Defines the optional default experiment.

        See section: 2.2.5
        """
        self.start_time = start_time
        self.stop_time = stop_time
        self.tolerance = tolerance
        self.step_size = step_size

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
        # "must be unique with respect to all other modelVariables" - section 2.2.7
        if any([v.name == name for v in self._model_variables]):
            raise ValueError(f"A variable with name '{name}' already exists.")

        if initial in {"exact", "approx"} or causality == "input" and start is None:
            raise ValueError(
                f"a start value must be defined for '{name}', since intial ∈ {{exact, approx}} or causality 'input'"
            )

        self._model_variables.append(
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

    def add_real_input(
        self,
        name: str,
        start: float,
        variability: Literal["discrete", "continuous"] = "continuous",
        min: float = None,
        max: float = None,
        unit: str = None,
        display_unit: str = None,
        nominal: float = None,
        unbounded: bool = False,
        description: str = None,
    ):
        """Declare a new input of the model.
        
        Inputs must specifiy a start value:
        >>> fmu.add_real_input("speed", start=0.0, unit="m/s")

        """
        self._add_variable(
            name=name,
            type=type,
            causality="input",
            variability=variability,
            initial=None,
            start=start,
            min=min,
            max=max,
            description=description,
            unit=unit,
            display_unit=display_unit,
            nominal=nominal,
            unbounded=unbounded,
        )

    def add_real_parameter(
        self,
        name: str,
        variability: Literal["fixed", "tunable"],
        start: float,
        min: float = None,
        max: float = None,
        unit: str = None,
        display_unit: str = None,
        nominal: float = None,
        unbounded: bool = False,
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

    def add_real_output(
        self,
        name: str,
        start: float,
        initial: Literal["exact", "calculated"],
        variability: Literal["discrete", "continuous"] = "continuous",
        description: str = None,
    ):
        self._add_variable(
            name=name,
            type=type,
            causality="output",
            variability=variability,
            initial=initial,
            start=start,
            description=description,
        )

    def add_real_local_variable(
        self,
        name: str,
        type: Type[T],
        variability: Literal["constant", "fixed", "tunable", "discrete", "continuous"],
        initial: Literal["exact", "approx", "calculated"],
        description: str = None,
    ):
        self._add_variable(
            name=name,
            type=type,
            causality="local",
            variability=variability,
            initial=initial,
            description=description,
        )

    def add_independent_variable(
        self, name: str, description: str = None,
    ):
        self._add_variable(
            name=name, causality="independent", description=description,
        )

    def export_fmu(
        self,
        language: Literal["python", "csharp", "java"],
        backend: Literal["grpc", "schemaless"],
        outdir: PathLike,
    ):
        """Generate FMU-template for a specific choice of language and backend."""

    def declare_dependency(
        self,
        dependent: str,
        dependency: str,
        kind: Literal["constant", "dependent"] = "dependent",
    ):
        """Declares a dependency from one variable to another.

        See 2.2.8
        """
        if kind not in {"constant", "dependent"}:
            raise ValueError(
                "The kind of a dependency must be either 'constant' or 'dependent'."
            )

        if dependent in self.dependencies:
            self.dependencies[dependent].append((dependency, kind))
        else:
            self.dependencies[dependent] = [(dependency, kind)]

    def get_model_description(self) -> str:
        """Converts in memory representation of the model description into its XML representation"""

        # ---------------- write model description -------------------
        import xml.etree.ElementTree as ET

        fmd = ET.Element("fmiModelDescription")

        fmd.set("fmiVersion", "2.0")

        if self.model_name:
            fmd.set("modelName", str(self.model_name))

        if self.guid:
            fmd.set("guid", str(self.guid))

        if self.author:
            fmd.set("author", str(self.author))

        if self.generation_date_and_time:
            fmd.set("generationDateAndTime", self.generation_date_and_time)
        if self.variable_naming_convetion:
            fmd.set("variableNamingConvention", self.variable_naming_convetion)

        if self.generation_tool:
            fmd.set("generationTool", self.generation_tool)

        if self.description:
            fmd.set("description", self.description)

        # CoSimulation
        cs = ET.SubElement(fmd, "CoSimulation")
        cs.set("modelIdentifier", self.model_identifier)
        cs.set(
            "needsExecutionTool", str(self.needs_execution_tool).lower(),
        )
        cs.set(
            "canHandleVariableCommunicationStepSize",
            str(self.can_handle_variable_communication_step_size).lower(),
        )
        cs.set(
            "canInterpolateInputs", str(self.can_interpolate_inputs).lower(),
        )

        cs.set(
            "maxOutputDerivativeOrder", str(self.max_output_derivative_order),
        )
        cs.set(
            "canRunAsynchronuously", str(self.can_run_asynchronuously).lower(),
        )
        cs.set(
            "canBeInstantiatedOnlyOncePerProcess",
            str(self.can_be_instantiated_only_once_per_process).lower(),
        )
        cs.set(
            "canNotUseMemoryManagementFunctions",
            str(self.can_not_use_memory_management_functions).lower(),
        )
        cs.set(
            "canGetAndSetFMUstate", str(self.can_get_and_set_fmu_state).lower(),
        )
        cs.set(
            "canSerializeFMUstate", str(self.can_serialize_fmu_state).lower(),
        )
        cs.set(
            "providesDirectionalDerivative",
            str(self.provides_directional_derivative).lower(),
        )

        # 2.2.4 p.42) Log categories:
        cs = ET.SubElement(fmd, "LogCategories")
        for ac in self.log_categories:
            c = ET.SubElement(cs, "Category")
            c.set("name", ac)

        # 2.2.7 p.47) ModelVariables
        mvs = ET.SubElement(fmd, "ModelVariables")

        variable_index = 0

        # assign value-references based on order of definition
        var_to_value_reference = {
            var: idx for idx, var in enumerate(self._model_variables)
        }

        for var in self._model_variables:
            value_reference = str(var_to_value_reference[var])

            idx_comment = ET.Comment(f'Index of variable = "{variable_index + 1}"')
            mvs.append(idx_comment)
            sv = ET.SubElement(mvs, "ScalarVariable")
            sv.set("name", var.name)
            sv.set("valueReference", value_reference)
            sv.set("variability", var.variability)
            sv.set("causality", var.causality)

            if var.description:
                sv.set("description", var.description)

            if var.initial:
                i = var.initial
                sv.set("initial", i)

            val = ET.SubElement(sv, var.type)

            # 2.2.7. p.48) start values
            if var.initial in {"exact", "approx"} or var.causality == "input":
                if var.start is None:
                    raise ValueError(
                        f"a start value must be defined for '{var.name}', since intial ∈ {{exact, approx}}"
                    )
                val.set("start", str(var.start))

            variable_index += 1

        ms = ET.SubElement(fmd, "ModelStructure")

        # 2.2.8) For each output we must declare 'Outputs' and 'InitialUnknowns'
        outputs = [
            (idx + 1, o)
            for idx, o in enumerate(self._model_variables)
            if o.causality == "output"
        ]

        if outputs:
            os = ET.SubElement(ms, "Outputs")
            for idx, o in outputs:
                ET.SubElement(os, "Unknown", {"index": str(idx), "dependencies": ""})

            os = ET.SubElement(ms, "InitialUnknowns")
            for idx, o in outputs:
                ET.SubElement(os, "Unknown", {"index": str(idx), "dependencies": ""})

        # FMI requires encoding to be encoded as UTF-8 and contain a header:
        # See 2.2 p.28

        def indent(elem, level=0):
            i = "\n" + level * "  "
            j = "\n" + (level - 1) * "  "
            if len(elem):
                if not elem.text or not elem.text.strip():
                    elem.text = i + "  "
                if not elem.tail or not elem.tail.strip():
                    elem.tail = i
                for subelem in elem:
                    indent(subelem, level + 1)
                if not elem.tail or not elem.tail.strip():
                    elem.tail = j
            else:
                if level and (not elem.tail or not elem.tail.strip()):
                    elem.tail = j
            return elem

        try:
            fmd.indent()  # Python 3.9
        except Exception:
            fmd = indent(fmd)

        return ET.tostring(fmd, encoding="utf-8", xml_declaration=True)


class Fmi3FMU:
    pass


if __name__ == "__main__":

    cosim = Fmi2CoSim()

    fmu = Fmi2FMU(
        model_name="Robot2D", author="unifmu-devs", language="python", backend="grpc"
    )

    fmu.declare_default_experiment(0.0, 1.0, None, None)
    fmu.declare_base_unit("rad/s", {"s": -1, "rad": 1})
    fmu.declare_base_unit("pos", {"m": 1})
    fmu.declare_base_unit("angle", {"r": 1})
    fmu.declare_display_unit("rad/s", "rpm", 9.549296585513721, 0.0)

    fmu.declare_log_category("solver", "messages emitted by numerical solver")

    # issues:
    # 1. only real's can have min,max,nominal, etc

    # reals

    # fmu.add_fixed_input()
    # fmu.add_parameter_tunable_real

    # fmu.define_limits("a",)

    # fmu.add_calculated_parameter_fixed_real
    # fmu.add_calculated_parameter_tunable_real
    # fmu.add_discrete_input_real
    # fmu.add_continuous_input_real

    # #
    # fmu.add_constant_output_real
    # fmu.add_discrete_output_real
    # fmu.add_continuous_output_real

    # fmu.add_constant_local_real
    # fmu.add_fixed_local_real
    # fmu.add_tunable_local_real
    # fmu.add_discrete_local_real
    # fmu.add_continuous_local_real

    fmu.add_real_input(
        "control_signal",
        "continous",
        start=0.0,
        description="control signal",
        min=-1.0,
        max=1.0,
    )

    for var in ["x", "y"]:
        fmu.add_real_input(
            f"{var}_pos",
            "real",
            "continous",
            start=0.0,
            description=f"{var} position represented as euclidean coordinates.",
            unit="pos",
        )
        fmu.add_real_input(
            var,
            "real",
            "continous",
            start=0.0,
            description=f"{var} angles represented as radians.",
            unit="angle",
        )
        fmu.add_real_input(
            f"ω_{var}",
            "real",
            "continous",
            start=0.0,
            description=f"{var} angular velocity around the {var}-axis .",
            unit="rad/s",
        )

    # adder.export_fmu("python", "schemaless", ".")
    md = fmu.get_model_description()

    with open("test.xml", "wb") as f:
        f.write(md)

