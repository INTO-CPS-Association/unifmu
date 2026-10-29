"""
FMI 2.0 ScalarVariable representation based on fmi2ScalarVariable.xsd

This module provides Python classes to represent FMI 2.0 scalar variables
with proper type safety using enums and dataclasses.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, Union, List
import xml.etree.ElementTree as ET
from typing import get_type_hints, get_origin, Annotated, get_args
from datetime import datetime, timezone

class DependencyKind(Enum):
    """The types of dependency"""
    CONSTANT = "constant"
    FIXED = "fixed"
    TUNABLE = "tunable"
    DISCRETE = "discrete"

class Causality(Enum):
    """Causality of a scalar variable"""
    PARAMETER = "parameter"
    CALCULATED_PARAMETER = "calculatedParameter"
    INPUT = "input"
    OUTPUT = "output"
    LOCAL = "local"
    INDEPENDENT = "independent"


class Variability(Enum):
    """Variability of a scalar variable"""
    CONSTANT = "constant"
    FIXED = "fixed"
    TUNABLE = "tunable"
    DISCRETE = "discrete"
    CONTINUOUS = "continuous"


class Initial(Enum):
    """Initial value setting"""
    EXACT = "exact"
    APPROX = "approx"
    CALCULATED = "calculated"


@dataclass
class RealType:
    """Real variable type attributes"""
    declared_type: Optional[str] = None
    quantity: Optional[str] = None
    unit: Optional[str] = None
    display_unit: Optional[str] = None
    relative_quantity: Optional[bool] = None
    min: Optional[float] = None
    max: Optional[float] = None
    nominal: Optional[float] = None
    unbounded: Optional[bool] = None
    start: Optional[float] = None
    derivative: Optional[int] = None  # Index of the variable this is a derivative of
    reinit: bool = False  # Can be reinitialized at an event


@dataclass
class IntegerType:
    """Integer variable type attributes"""
    declared_type: Optional[str] = None
    quantity: Optional[str] = None
    min: Optional[int] = None
    max: Optional[int] = None
    start: Optional[int] = None


@dataclass
class BooleanType:
    """Boolean variable type attributes"""
    declared_type: Optional[str] = None
    start: Optional[bool] = None


@dataclass
class StringType:
    """String variable type attributes"""
    declared_type: Optional[str] = None
    start: Optional[str] = None


@dataclass
class EnumerationType:
    """Enumeration variable type attributes"""
    declared_type: str  # Required for enumeration
    quantity: Optional[str] = None
    min: Optional[int] = None
    max: Optional[int] = None
    start: Optional[int] = None


# Union type for all variable types

VariableType = Union[RealType, IntegerType, BooleanType, StringType, EnumerationType]


@dataclass
class Annotation:
    """Annotation for additional metadata"""
    # Simplified - can be expanded based on fmi2Annotation.xsd if needed
    data: Optional[dict] = None


@dataclass
class FmiModelMetadata:
    """Optional class-level metadata for fmiModelDescription attributes."""
    author: Optional[str] = None


def fmi_model(author: str):
    """Class decorator that stores FMI metadata on the model class."""
    metadata = FmiModelMetadata(author=author)

    def decorator(cls):
        cls.__fmi_model_metadata__ = metadata
        return cls

    return decorator


@dataclass
class ScalarVariable:
    """
    FMI 2.0 Scalar Variable representation

    Attributes:
        name: Identifier of variable (required, must be unique)
        value_reference: Identifier for FMI2 function calls (required)
        variable_type: The type and attributes of the variable (Real, Integer, Boolean, String, or Enumeration)
        description: Optional description text
        causality: Causality of the variable (default: LOCAL)
        variability: Variability of the variable (default: CONTINUOUS)
        initial: Initial value setting
        can_handle_multiple_set_per_time_instant: For ModelExchange inputs only
        annotations: Additional metadata
    """
    name:Optional[ str] = None
    value_reference: Optional[int] = None
    variable_type: VariableType = field(default_factory=lambda: RealType(start=0.0))
    description: Optional[str] = None
    causality: Causality = Causality.LOCAL
    variability: Variability = Variability.CONTINUOUS
    initial: Optional[Initial] = None
    can_handle_multiple_set_per_time_instant: Optional[bool] = None
    annotations: Optional[Annotation] = None
    dependencies: Optional[List[str]] = None
    dependency_kind: Optional[List[DependencyKind]] = None

    def __post_init__(self):
        if self.value_reference is None:
            # This is a placeholder, it should be set before serialization
            # to a correct and unique value.
            pass

    def is_real(self) -> bool:
        """Check if this variable is of Real type"""
        return isinstance(self.variable_type, RealType)

    def is_integer(self) -> bool:
        """Check if this variable is of Integer type"""
        return isinstance(self.variable_type, IntegerType)

    def is_boolean(self) -> bool:
        """Check if this variable is of Boolean type"""
        return isinstance(self.variable_type, BooleanType)

    def is_string(self) -> bool:
        """Check if this variable is of String type"""
        return isinstance(self.variable_type, StringType)

    def is_enumeration(self) -> bool:
        """Check if this variable is of Enumeration type"""
        return isinstance(self.variable_type, EnumerationType)

    def get_start_value(self) -> Optional[Union[float, int, bool, str]]:
        """Get the start value regardless of variable type"""
        return getattr(self.variable_type, 'start', None)



class ModelDescriptionXmlWriter:
    def get_class_model_metadata(cls) -> FmiModelMetadata:
        metadata = getattr(cls, "__fmi_model_metadata__", None)
        if isinstance(metadata, FmiModelMetadata):
            return metadata
        return FmiModelMetadata()

    def scalar_variable_to_xml(var: ScalarVariable, indent: str = "  ") -> str:
        """
        Convert a ScalarVariable to FMI 2.0 XML format.

        Args:
            var: The ScalarVariable to convert
            indent: Indentation string for nested elements (default: two spaces)

        Returns:
            XML string representation of the ScalarVariable
        """
        element = ModelDescriptionXmlWriter.scalar_variable_to_element(var)
        if indent:
            ET.indent(element, space=indent)
        return ET.tostring(element, encoding="unicode")

    def scalar_variable_to_element(var: ScalarVariable) -> ET.Element:
        """Convert a ScalarVariable to an ElementTree element."""
        attrs = {
            "name": str(var.name),
            "valueReference": str(var.value_reference),
        }

        if var.description:
            attrs["description"] = var.description
        # if var.causality != Causality.LOCAL:
        attrs["causality"] = str(var.causality.value)
        # if var.variability != Variability.CONTINUOUS:
        attrs["variability"] = str(var.variability.value)
        if var.initial is not None:
            attrs["initial"] = var.initial.value
        if var.can_handle_multiple_set_per_time_instant is not None:
            attrs["canHandleMultipleSetPerTimeInstant"] = (
                "true" if var.can_handle_multiple_set_per_time_instant else "false"
            )

        scalar = ET.Element("ScalarVariable", attrs)
        scalar.append(ModelDescriptionXmlWriter._variable_type_to_element(var.variable_type))
        return scalar

    def _variable_type_to_element(variable_type) -> ET.Element:
        if isinstance(variable_type, RealType):
            return ModelDescriptionXmlWriter._real_type_to_element(variable_type)
        if isinstance(variable_type, IntegerType):
            return ModelDescriptionXmlWriter._integer_type_to_element(variable_type)
        if isinstance(variable_type, BooleanType):
            return ModelDescriptionXmlWriter._boolean_type_to_element(variable_type)
        if isinstance(variable_type, StringType):
            return ModelDescriptionXmlWriter._string_type_to_element(variable_type)
        if isinstance(variable_type, EnumerationType):
            return ModelDescriptionXmlWriter._enumeration_type_to_element(variable_type)
        raise TypeError(f"Unsupported variable type: {type(variable_type).__name__}")

    def _bool_text(value: bool) -> str:
        return "true" if value else "false"

    def _real_type_to_element(real: RealType) -> ET.Element:
        attrs = {}
        if real.declared_type:
            attrs["declaredType"] = real.declared_type
        if real.quantity:
            attrs["quantity"] = real.quantity
        if real.unit:
            attrs["unit"] = real.unit
        if real.display_unit:
            attrs["displayUnit"] = real.display_unit
        if real.relative_quantity is not None:
            attrs["relativeQuantity"] = ModelDescriptionXmlWriter._bool_text(real.relative_quantity)
        if real.min is not None:
            attrs["min"] = str(real.min)
        if real.max is not None:
            attrs["max"] = str(real.max)
        if real.nominal is not None:
            attrs["nominal"] = str(real.nominal)
        if real.unbounded is not None:
            attrs["unbounded"] = ModelDescriptionXmlWriter._bool_text(real.unbounded)
        if real.start is not None:
            attrs["start"] = str(real.start)
        if real.derivative is not None:
            attrs["derivative"] = str(real.derivative)
        if real.reinit:
            attrs["reinit"] = "true"
        return ET.Element("Real", attrs)

    def _integer_type_to_element(integer: IntegerType) -> ET.Element:
        attrs = {}
        if integer.declared_type:
            attrs["declaredType"] = integer.declared_type
        if integer.quantity:
            attrs["quantity"] = integer.quantity
        if integer.min is not None:
            attrs["min"] = str(integer.min)
        if integer.max is not None:
            attrs["max"] = str(integer.max)
        if integer.start is not None:
            attrs["start"] = str(integer.start)
        return ET.Element("Integer", attrs)

    def _boolean_type_to_element(boolean: BooleanType) -> ET.Element:
        attrs = {}
        if boolean.declared_type:
            attrs["declaredType"] = boolean.declared_type
        if boolean.start is not None:
            attrs["start"] = ModelDescriptionXmlWriter._bool_text(boolean.start)
        return ET.Element("Boolean", attrs)

    def _string_type_to_element(string: StringType) -> ET.Element:
        attrs = {}
        if string.declared_type:
            attrs["declaredType"] = string.declared_type
        if string.start is not None:
            attrs["start"] = string.start
        return ET.Element("String", attrs)

    def _enumeration_type_to_element(enum: EnumerationType) -> ET.Element:
        attrs = {"declaredType": enum.declared_type}
        if enum.quantity:
            attrs["quantity"] = enum.quantity
        if enum.min is not None:
            attrs["min"] = str(enum.min)
        if enum.max is not None:
            attrs["max"] = str(enum.max)
        if enum.start is not None:
            attrs["start"] = str(enum.start)
        return ET.Element("Enumeration", attrs)

    def _real_type_to_xml(real: RealType, indent: str) -> str:
        """Convert RealType to XML element"""
        attrs = []

        if real.declared_type:
            attrs.append(f'declaredType="{real.declared_type}"')
        if real.quantity:
            attrs.append(f'quantity="{real.quantity}"')
        if real.unit:
            attrs.append(f'unit="{real.unit}"')
        if real.display_unit:
            attrs.append(f'displayUnit="{real.display_unit}"')
        if real.relative_quantity is not None:
            value = "true" if real.relative_quantity else "false"
            attrs.append(f'relativeQuantity="{value}"')
        if real.min is not None:
            attrs.append(f'min="{real.min}"')
        if real.max is not None:
            attrs.append(f'max="{real.max}"')
        if real.nominal is not None:
            attrs.append(f'nominal="{real.nominal}"')
        if real.unbounded is not None:
            value = "true" if real.unbounded else "false"
            attrs.append(f'unbounded="{value}"')
        if real.start is not None:
            attrs.append(f'start="{real.start}"')
        if real.derivative is not None:
            attrs.append(f'derivative="{real.derivative}"')
        if real.reinit:
            attrs.append(f'reinit="true"')

        if attrs:
            return f'{indent}<Real {" ".join(attrs)}/>'
        else:
            return f'{indent}<Real/>'

    def _integer_type_to_xml(integer: IntegerType, indent: str) -> str:
        """Convert IntegerType to XML element"""
        attrs = []

        if integer.declared_type:
            attrs.append(f'declaredType="{integer.declared_type}"')
        if integer.quantity:
            attrs.append(f'quantity="{integer.quantity}"')
        if integer.min is not None:
            attrs.append(f'min="{integer.min}"')
        if integer.max is not None:
            attrs.append(f'max="{integer.max}"')
        if integer.start is not None:
            attrs.append(f'start="{integer.start}"')

        if attrs:
            return f'{indent}<Integer {" ".join(attrs)}/>'
        else:
            return f'{indent}<Integer/>'

    def _boolean_type_to_xml(boolean: BooleanType, indent: str) -> str:
        """Convert BooleanType to XML element"""
        attrs = []

        if boolean.declared_type:
            attrs.append(f'declaredType="{boolean.declared_type}"')
        if boolean.start is not None:
            value = "true" if boolean.start else "false"
            attrs.append(f'start="{value}"')

        if attrs:
            return f'{indent}<Boolean {" ".join(attrs)}/>'
        else:
            return f'{indent}<Boolean/>'

    def _string_type_to_xml(string: StringType, indent: str) -> str:
        """Convert StringType to XML element"""
        attrs = []

        if string.declared_type:
            attrs.append(f'declaredType="{string.declared_type}"')
        if string.start is not None:
            # Escape XML special characters
            escaped = string.start.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;').replace('"',
                                                                                                           '&quot;')
            attrs.append(f'start="{escaped}"')

        if attrs:
            return f'{indent}<String {" ".join(attrs)}/>'
        else:
            return f'{indent}<String/>'

    def _enumeration_type_to_xml(enum: EnumerationType, indent: str) -> str:
        """Convert EnumerationType to XML element"""
        attrs = [f'declaredType="{enum.declared_type}"']  # Required for enumeration

        if enum.quantity:
            attrs.append(f'quantity="{enum.quantity}"')
        if enum.min is not None:
            attrs.append(f'min="{enum.min}"')
        if enum.max is not None:
            attrs.append(f'max="{enum.max}"')
        if enum.start is not None:
            attrs.append(f'start="{enum.start}"')

        return f'{indent}<Enumeration {" ".join(attrs)}/>'

    def find_class_scalar_variables(cls):
        """
        Find all class attributes with ScalarVariable type annotation.

        Args:
            cls: The class to inspect

        Returns:
            List of tuples containing (attribute_name, ScalarVariable_instance)
        """
        scalar_vars = []

        # Get type hints for the class (with include_extras=True to get Annotated metadata)
        try:
            type_hints = get_type_hints(cls, include_extras=True)
        except Exception:
            type_hints = {}

        # Iterate through type hints to find Annotated types with ScalarVariable
        for name, hint in type_hints.items():
            # Check if it's an Annotated type
            if get_origin(hint) is Annotated:
                args = get_args(hint)
                # args[0] is the actual type (float, int, etc.)
                # args[1:] are the metadata annotations
                for metadata in args[1:]:
                    if isinstance(metadata, ScalarVariable):
                        scalar_vars.append((name, metadata))
                        break

        return scalar_vars

    def build_model_description_xml(scs, model_metadata: Optional[FmiModelMetadata] = None):
        model_metadata = model_metadata or FmiModelMetadata()
        root = ET.Element(
            "fmiModelDescription",
            {
                "fmiVersion": "2.0",
                "modelName": "unifmu",
                "guid": "77236337-210e-4e9c-8f2c-c1a0677db21b",
                # this is not model-specific but to ensure the native library matches so its build into the FMI library
                "author": model_metadata.author or "Unknown",
                "generationDateAndTime": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                "variableNamingConvention": "flat",
                "generationTool": "unifmu",
            },
        )

        ET.SubElement(
            root,
            "CoSimulation",
            {
                "modelIdentifier": "unifmu",
                "needsExecutionTool": "true",
                "canNotUseMemoryManagementFunctions": "true",
                "canHandleVariableCommunicationStepSize": "true",
                "canGetAndSetFMUstate": "true",
                "canSerializeFMUstate": "true",
            },
        )

        log_categories = ET.SubElement(root, "LogCategories")
        for category in [
            {"name": "logStatusWarning"},
            {"name": "logStatusDiscard"},
            {"name": "logStatusError"},
            {"name": "logStatusFatal"},
            {"name": "logStatusPending"},
            {"name": "logAll"},
            {
                "name": "logUnifmuMessages",
                "description": "Messages related to internal UniFMU functionality. Enabling this category is required for distributed UniFMUs.",
            },
        ]:
            ET.SubElement(log_categories, "Category", category)

        model_variables = ET.SubElement(root, "ModelVariables")
        for idx, (_, sv) in enumerate(scs):
            model_variables.append(ET.Comment(f'Index of variable = "{idx + 1}"'))
            model_variables.append(ModelDescriptionXmlWriter.scalar_variable_to_element(sv))

        model_structure = ET.SubElement(root, "ModelStructure")
        outputs = ET.SubElement(model_structure, "Outputs")
        initial_unknowns = ET.SubElement(model_structure, "InitialUnknowns")

        for idx, (_, var) in enumerate(scs):

            attributes = {"index": str(idx+1),
                          "dependencies": " ".join([str(d_idx+1) for d_idx, (_, dsv) in enumerate(scs) if
                                                    dsv.name in var.dependencies]) if var.dependencies else "",
                          }
            if var.dependency_kind:
                attributes["dependenciesKind"] = " ".join([str(kind.value) for kind in var.dependency_kind])

            if var.causality == var.causality.OUTPUT:
                outputs.append(ET.Comment(f"{var.name}"))
                ET.SubElement(outputs, "Unknown", attributes)
            if var.causality == Causality.CALCULATED_PARAMETER or (var.causality ==Causality.OUTPUT  and var.initial in [Initial.APPROX, Initial.CALCULATED]): # and all derivatives
                initial_unknowns.append(ET.Comment(f"{var.name}"))
                ET.SubElement(initial_unknowns, "Unknown", attributes)

        ET.indent(root, space="  ")
        return ET.tostring(root, encoding="utf-8", xml_declaration=True).decode("utf-8")

    def generate_model_description_xml(my_model):
        """
        This function generates an FMI 2.0 XML model description file for a given model instance.
        Returns:
            the model description XML as a string
        """
        scs = ModelDescriptionXmlWriter.find_class_scalar_variables(my_model.__class__)

        for idx, (name, var) in enumerate(scs):
            var.value_reference = idx
            if var.name is None:
                var.name = name
            if var.causality == var.causality.INPUT and var.variable_type.start is None:
                var.variable_type.start = getattr(my_model, name)

        metadata = ModelDescriptionXmlWriter.get_class_model_metadata(my_model.__class__)
        return ModelDescriptionXmlWriter.build_model_description_xml(scs, model_metadata=metadata)

def build_reference_to_attribute_map(my_model):
    """Builds a map of value references to scalar variable attributes for a given model instance.
    Returns:
        a dictionary mapping value references to scalar variable attributes
    """
    return {idx: name for idx,(name, _) in enumerate(ModelDescriptionXmlWriter.find_class_scalar_variables(my_model))}

def generate_model_description_xml(my_model):
    """Convenience wrapper to import XML generation directly from this module."""
    return ModelDescriptionXmlWriter.generate_model_description_xml(my_model)


