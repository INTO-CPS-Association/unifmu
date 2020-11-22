"""
Contains definitions of concepts related to FMI 2.0.x

"""


from typing import Any, List, Optional
from enum import Enum


class ScalarVariable:
    """input, output or parameter of a model, p.46"""

    def __init__(
        self,
        name: str,
        value_reference: str,
        data_type: str,
        description: str = None,
        causality: str = "local",
        variability: str = "continuous",
        initial: str = None,
        start: Optional[Any] = None,
        can_handle_multiple_set_per_time_instant=None,
    ):
        self.name = name
        self.value_reference = value_reference
        self.data_type = data_type
        self.variability = variability
        self.causality = causality
        self.initial = initial
        self.description = description
        self.start = start
        self.can_handle_multiple_set_per_time_instant = (
            can_handle_multiple_set_per_time_instant
        )


class ModelExchange:
    pass


class CoSimulation:
    "based on fmi 2.0.1 p. 110"

    def __init__(
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

        self.model_identifier = model_identifier
        self.needs_execution_tool = needs_execution_tool
        self.can_handle_variable_communication_step_size = (
            can_handle_variable_communication_step_size
        )
        self.can_interpolate_inputs = can_interpolate_inputs
        self.max_output_derivative_order = max_output_derivative_order
        self.can_run_asynchronously = can_run_asynchronuously
        self.can_be_instantiated_only_once_per_process = (
            can_be_instantiated_only_once_per_process
        )
        self.can_not_use_memory_management_functions = (
            can_not_use_memory_management_functions
        )
        self.can_get_and_set_fmu_state = can_get_and_set_fmu_state
        self.can_serialize_fmu_state = can_serialize_fmu_state
        self.provides_directional_derivatives = provides_directional_derivative


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
        model_variables: List[ScalarVariable],
        model_structure,
        description: str = None,
        author: str = None,
        version: str = None,
        copyright: str = None,
        license: str = None,
        generation_tool: str = None,
        generation_date_and_time: str = None,
        variable_naming_convention: str = None,
        co_simulation: CoSimulation = None,
        model_exchange: ModelExchange = None,
        unit_definitions=None,
        type_defintions=None,
        log_categories: List[str] = None,
        default_experiment=None,
        vendor_annotations=None,
        number_of_event_indicators=None,
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
        self.number_of_event_indicators = number_of_event_indicators

