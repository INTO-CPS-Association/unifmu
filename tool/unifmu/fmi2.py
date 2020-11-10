"""Contains definitions of concepts related to FMI 2.0.x"""


from typing import Any


class ScalarVariable:
    def __init__(
        self,
        name: str,
        value_reference: str,
        variability: str,
        causality: str,
        start: Any = None,
    ):
        self.name = name
        self.value_reference = value_reference
        self.variability = variability
        self.causality = causality
        self.start = start


class ModelExchange:
    pass


class CoSimulation:
    "based on fmi 2.0.1 p. 110"

    def __init__(
        self,
        model_identifier: str,
        needs_execution_tool: bool,
        can_handle_variable_communication_step_size: bool,
        can_interpolate_inputs: bool,
        max_output_derivative_order: int,
        can_run_asynchronously: bool,
        can_be_instantiated_only_once_per_process: bool,
        can_not_use_memory_management_functions: bool,
        can_get_and_set_fmu_state: bool,
        can_serialize_fmu_state: bool,
        provides_directional_derivatives: bool,
    ) -> None:

        self.model_identifier = model_identifier
        self.needs_execution_tool = needs_execution_tool
        self.can_handle_variable_communication_step_size = (
            can_handle_variable_communication_step_size
        )
        self.can_interpolate_inputs = can_interpolate_inputs
        self.max_output_derivative_order = max_output_derivative_order
        self.can_run_asynchronously = can_run_asynchronously
        self.can_be_instantiated_only_once_per_process = (
            can_be_instantiated_only_once_per_process
        )
        self.can_not_use_memory_management_functions = (
            can_not_use_memory_management_functions
        )
        self.can_get_and_set_fmu_state = can_get_and_set_fmu_state
        self.can_serialize_fmu_state = can_serialize_fmu_state
        self.provides_directional_derivatives = provides_directional_derivatives


class Unit:
    pass


class SimpleType:
    pass


class ModelDescription:
    "based on fmi 2.0.1 p. 32"

    def __init__(
        self,
        fmi_version: str,
        model_name: str,
        guid: str,
        description: str,
        author: str,
        version: str,
        copyright: str,
        license: str,
        generation_tool: str,
        generation_date_and_time: str,
        variable_naming_convention: str,
        model_variables,
        model_structure,
        co_simulation: CoSimulation,
        model_exchange: ModelExchange,
        unit_definitions,
        type_defintions,
        log_categories,
        default_experiment,
        vendor_annotations,
    ):
        self.fmi_version = fmi_version
        self.model_name = model_name
        self.guid = guid
        self.description = description
        self.author = author
        self.version = version
        self.copyright = copyright
        self.license = license
        self.generation_tool = generation_tool
        self.generation_date_and_time = generation_date_and_time
        self.variable_naming_convention = variable_naming_convention
        self.model_variables = model_variables
        self.model_structure = model_structure
        self.co_simulation = co_simulation
        self.model_exchange = model_exchange
        self.unit_definitions = unit_definitions
        self.type_defintions = type_defintions
        self.log_categories = log_categories
        self.default_experiment = default_experiment
        self.vendor_annotations = vendor_annotations

