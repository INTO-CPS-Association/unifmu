"""
Contains definitions of data structures mapping the XML schema defined FMI 2.0.x to python class.

Additionally, the module defines functions for determining the allowed combinations of attributes.
"""


from typing import Any, Dict, List, Optional, Tuple
from enum import Enum


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
