"""
Contains definitions of data structures mapping the XML schema defined FMI 2.0.x to python class.

Additionally, the module defines functions for determining the allowed combinations of attributes.
"""


from typing import Any, Dict, List, Optional, Tuple
from enum import Enum
import xml.etree.ElementTree as ET


class ScalarVariable:
    """input, output or parameter of a model, p.46"""

    def __init__(
        self,
        name: str,
        valueReference: str,
        dataType: str,
        description: str,
        causality: str = "local",
        variability: str = "continuous",
        initial: str = None,
        start: Optional[Any] = None,
        canHandleMultipleSetPerTimeInstant=None,
    ):
        self.name = name
        self.value_reference = valueReference
        self.dataType = dataType
        self.variability = variability
        self.causality = causality
        self.initial = initial
        self.description = description
        self.start = start
        self.canHandleMultipleSetPerTimeInstant = canHandleMultipleSetPerTimeInstant


class ModelExchange:
    pass


class CoSimulation:
    "based on fmi 2.0.1 p. 110"

    def __init__(
        self,
        modelIdentifier: str,
        needsExecutionTool: bool = None,
        canHandleVariableCommunicationStepSize: bool = None,
        canInterpolateInputs: bool = None,
        maxOutputDerivativeOrder: int = None,
        canRunAsynchronuously: bool = None,
        canBeInstantiatedOnlyOncePerProcess: bool = None,
        canNotUseMemoryManagementFunctions: bool = None,
        canGetAndSetFMUstate: bool = None,
        canSerializeFMUstate: bool = None,
        providesDirectionalDerivative: bool = None,
    ):

        self.modelIdentifier = modelIdentifier
        self.needsExecutionTool = needsExecutionTool
        self.canHandleVariableCommunicationStepSize = (
            canHandleVariableCommunicationStepSize
        )
        self.canInterpolateInputs = canInterpolateInputs
        self.maxOutputDerivativeOrder = maxOutputDerivativeOrder
        self.canRunAsynchronuously = canRunAsynchronuously
        self.canBeInstantiatedOnlyOncePerProcess = canBeInstantiatedOnlyOncePerProcess
        self.canNotUseMemoryManagementFunctions = canNotUseMemoryManagementFunctions
        self.canGetAndSetFMUstate = canGetAndSetFMUstate
        self.canSerializeFMUstate = canSerializeFMUstate
        self.providesDirectionalDerivative = providesDirectionalDerivative


class Unit:
    pass


class SimpleType:
    pass


class ModelDescription:
    "based on fmi 2.0.1 p. 32"

    def __init__(
        self,
        fmiVersion: str,
        modelName: str,
        guid: str,
        modelVariables: List[ScalarVariable] = [],
        modelStructure=[],
        description: str = None,
        author: str = None,
        version: str = None,
        copyright: str = None,
        license: str = None,
        generationTool: str = None,
        generationDateAndTime: str = None,
        variableNamingConvention: str = None,
        CoSimulation: CoSimulation = None,
        ModelExchange: ModelExchange = None,
        unitDefinitions=None,
        typeDefintions=None,
        logCategories: List[str] = None,
        defaultExperiment=None,
        vendorAnnotations=None,
        numberOfEventIndicators=None,
    ):
        self.fmiVersion = fmiVersion
        self.modelName = modelName
        self.guid = guid
        self.description = description
        self.author = author
        self.version = version
        self.copyright = copyright
        self.license = license
        self.generationTool = generationTool
        self.generationDateAndTime = generationDateAndTime
        self.variableNamingConvention = variableNamingConvention
        self.modelVariables = modelVariables
        self.modelStructure = modelStructure
        self.CoSimulation = CoSimulation
        self.ModelExchange = ModelExchange
        self.unitDefinitions = unitDefinitions
        self.typeDefintions = typeDefintions
        self.logCategories = logCategories
        self.defaultExperiment = defaultExperiment
        self.vendorAnnotations = vendorAnnotations
        self.numberOfEventIndicators = numberOfEventIndicators


def get_causality_to_variability_choices(causality: str) -> List[str]:
    """Returns the possible choices of variability for the specified causality, see p.50."""

    if causality not in {
        "parameter",
        "calculatedParameter",
        "input",
        "output",
        "local",
        "independent",
    }:
        raise ValueError(f"Unrecognized value of causality: '{causality}'")

    return {
        "parameter": ["fixed", "tunable"],
        "calculatedParameter": ["fixed", "tunable"],
        "input": ["discrete", "continuous"],
        "output": ["constant", "discrete", "continuous"],
        "local": ["constant", "fixed", "tunable", "discrete", "continuous"],
        "independent": ["continuous"],
    }[causality]


def get_intitial_choices_and_default(
    causality: str, variability: str
) -> Tuple[Optional[str], List[str]]:
    """
    Returns the possible default and possible choices of initial for the specified combination of causality and variability, see p.50.
    
    In cases where initial MUST NOT be defined (None, []) is returned. 
    If an illegal combination of causality and variability is defined an execption is raised.
    """

    # case (A)
    if (variability == "constant" and causality in {"output", "local"}) or (
        variability in {"fixed", "tunable"} and causality == "parameter"
    ):
        return "exact", ["exact"]
    # case (B)
    elif variability in {"fixed", "tunable"} and causality in {
        "calculatedParameter",
        "local",
    }:
        return "calculated", ["approx", "calculated"]
    # case (C)
    elif variability in {"discrete", "continuous"} and causality in {"output", "local"}:
        return "calculated", ["exact", "approx", "calculated"]
    # case (D)
    elif variability in {"discrete", "continuous"} and causality == "input":
        return None, []
    elif variability == "continuous" and causality == "independent":
        return None, []
    else:
        raise ValueError(
            f"invalid combination of causality: '{causality}' and variability: '{variability}'"
        )


def get_should_define_start(initial: str) -> bool:
    return initial in {"exact", "approx"}


def get_default_attribute_values() -> Dict[str, str]:
    """Returns the default values for various attributes defined by the model description"""
    return {
        # ModelDescription
        "variableNamingConvention": "flat",
        # CoSimulation
        "needsExecutionTool": "false",
        "canHandleVariableCommunicationStepSize": "false",
        "canInterpolateInputs": "false",
        "maxOutputDerivativeOrder": "0",
        "canRunAsynchronuously": "false",
        "canBeInstantiatedOnlyOncePerProcess": "false",
        "canNotUseMemoryManagementFunctions": "false",
        "canGetAndSetFMUstate": "false",
        "canSerializeFMUstate": "false",
        "providesDirectionalDerivative": "false",
    }


def _get_attribute_default_values():
    return {
        # ModelDescription
        "variableNamingConvention": "flat",
        # CoSimulation
        "needsExecutionTool": "false",
        "canHandleVariableCommunicationStepSize": "false",
        "canInterpolateInputs": "false",
        "maxOutputDerivativeOrder": "0",
        "canRunAsynchronuously": "false",
        "canBeInstantiatedOnlyOncePerProcess": "false",
        "canNotUseMemoryManagementFunctions": "false",
        "canGetAndSetFMUstate": "false",
        "canSerializeFMUstate": "false",
        "providesDirectionalDerivative": "false",
    }


def parse_model_description(model_description: str) -> ModelDescription:
    """Parse the contents of the xml tree and return an in memory representation.
    """
    root = ET.fromstring(model_description)

    defaults = _get_attribute_default_values()

    # mandatory p.32
    fmi_version = root.get("fmiVersion")
    model_name = root.get("modelName")
    guid = root.get("guid")
    # optional
    description = root.get("description", default="")
    author = root.get("author", default="")
    copyright = root.get("copyright", default="")
    version = root.get("version", default="")
    license = root.get("license", default="")
    generation_tool = root.get("generationTool", default="")
    generation_date_and_time = root.get("generationDateAndTime", default="")
    variable_naming_convention = root.get("variableNamingConvention", default="flat")
    numberOfEventIndicators = root.get("numberOfEventIndicators", default=0)

    model_variables = []

    """ Iterate over model variables:
    <ScalarVariable name="real_a" valueReference="0" variability="continuous" causality="input">
        <Real start="0.0" />
    </ScalarVariable>
    """
    for scalarVariable in root.iter("ScalarVariable"):

        causality = scalarVariable.get("causality", default="local")
        variability = scalarVariable.get("variability", default="continuous")

        initial = scalarVariable.get("initial", default=None)
        # defaults of initial depend on causality and variablilty
        # the combinations lead to 5 different cases denoted A-E on p.50
        if initial is None:
            initial, _ = get_intitial_choices_and_default(causality, variability)

        var = list(scalarVariable)[0]
        start = var.get("start", default=None)
        dataType = var.tag

        model_variables.append(
            ScalarVariable(
                name=scalarVariable.get("name"),
                valueReference=scalarVariable.get("valueReference"),
                variability=variability,
                causality=causality,
                description=scalarVariable.get("description", default=""),
                initial=initial,
                start=start,
                dataType=dataType,
            )
        )

    log_categories = []
    for category in root.iter("Category"):
        log_categories.append(category.get("name"))

    model_structure = []

    # cosimulation
    cosim_element = root.find("CoSimulation")

    modelIdentifier = cosim_element.get("modelIdentifier")
    needsExecutionTool = cosim_element.get(
        "needsExecutionTool", default=defaults["needsExecutionTool"]
    )
    canHandleVariableCommunicationStepSize = cosim_element.get(
        "canHandleVariableCommunicationStepSize",
        default=defaults["canHandleVariableCommunicationStepSize"],
    )
    canInterpolateInputs = cosim_element.get(
        "canInterpolateInputs", default=defaults["canInterpolateInputs"]
    )
    maxOutputDerivativeOrder = cosim_element.get(
        "maxOutputDerivativeOrder", default=defaults["maxOutputDerivativeOrder"]
    )
    canRunAsynchronuously = cosim_element.get(
        "canRunAsynchronuously", default=defaults["canRunAsynchronuously"]
    )
    canBeInstantiatedOnlyOncePerProcess = cosim_element.get(
        "canBeInstantiatedOnlyOncePerProcess",
        default=defaults["canBeInstantiatedOnlyOncePerProcess"],
    )
    canNotUseMemoryManagementFunctions = cosim_element.get(
        "canNotUseMemoryManagementFunctions",
        default=defaults["canNotUseMemoryManagementFunctions"],
    )
    canGetAndSetFMUstate = cosim_element.get(
        "canGetAndSetFMUstate", default=defaults["canGetAndSetFMUstate"]
    )
    canSerializeFMUstate = cosim_element.get(
        "canSerializeFMUstate", default=defaults["canSerializeFMUstate"]
    )
    providesDirectionalDerivative = cosim_element.get(
        "providesDirectionalDerivative",
        default=defaults["providesDirectionalDerivative"],
    )

    def xs_boolean(s):
        if s is None:
            return None
        if s in {"false", "0"}:
            return False
        elif s in {"true", "1"}:
            return True
        else:
            raise ValueError(f"Unable to convert {s} to xsd boolean")

    def xs_normalized_string(s: str):
        if s is None:
            return None
        if not s.isprintable():
            raise ValueError(r"normalized string can not contain: \n, \t or \r")
        return s

    def xs_unsigned_int(s: str):
        if s is None:
            return None
        value = int(s)
        if value > 4294967295:
            raise ValueError("xs:unsingedInt cannot exceed the value 4294967295")
        return value

    cosimulation = CoSimulation(
        modelIdentifier=modelIdentifier,
        needsExecutionTool=xs_boolean(needsExecutionTool),
        canHandleVariableCommunicationStepSize=xs_boolean(
            canHandleVariableCommunicationStepSize
        ),
        canInterpolateInputs=xs_boolean(canInterpolateInputs),
        maxOutputDerivativeOrder=xs_unsigned_int(maxOutputDerivativeOrder),
        canRunAsynchronuously=xs_boolean(canRunAsynchronuously),
        canBeInstantiatedOnlyOncePerProcess=xs_boolean(
            canBeInstantiatedOnlyOncePerProcess
        ),
        canNotUseMemoryManagementFunctions=xs_boolean(
            canNotUseMemoryManagementFunctions
        ),
        canGetAndSetFMUstate=xs_boolean(canGetAndSetFMUstate),
        canSerializeFMUstate=xs_boolean(canSerializeFMUstate),
        providesDirectionalDerivative=xs_boolean(providesDirectionalDerivative),
    )

    return ModelDescription(
        fmiVersion=fmi_version,
        modelName=model_name,
        guid=guid,
        author=author,
        description=description,
        version=version,
        copyright=copyright,
        logCategories=log_categories,
        license=license,
        generationTool=generation_tool,
        generationDateAndTime=generation_date_and_time,
        variableNamingConvention=variable_naming_convention,
        numberOfEventIndicators=numberOfEventIndicators,
        CoSimulation=cosimulation,
        modelVariables=model_variables,
        modelStructure=model_structure,
    )
