from os import makedirs
from pathlib import Path
from shutil import copy
from tempfile import TemporaryDirectory
import shutil
from typing import List
import zipfile
from zipfile import ZipFile

# import xml.etree.ElementTree as ET
import pkg_resources

# import xml.etree.ElementTree as ET
import lxml.etree as ET
import toml

from unifmu.fmi2 import ModelDescription, CoSimulation, ScalarVariable


def list_resource_files(resource_name: str) -> List[str]:
    """Get a list containing all datafiles and directories available through setuptools pkg_resources interface

    Directories are appended a trailing slash to disinguish them from files, e.g. resoures -> resources/
    """

    files = []
    dirs = []

    def inner_recursion(resource_name: str):

        if not pkg_resources.resource_exists(__name__, resource_name):
            raise RuntimeError(
                f"The resource {resource_name}, does not seem to be available "
            )

        # base case
        if not pkg_resources.resource_isdir(__name__, resource_name):
            files.append(resource_name)

        # recursive case
        else:
            if not resource_name.endswith("/"):
                resource_name += "/"

            dirs.append(resource_name)
            children = pkg_resources.resource_listdir(__name__, resource_name)

            resource_names_extended = [f"{resource_name}{c}" for c in children]

            for child_resource in resource_names_extended:
                inner_recursion(child_resource)

    inner_recursion(resource_name)

    return files


def get_backends() -> List[str]:

    return list(
        toml.loads(
            pkg_resources.resource_string(__name__, "resources/backends.toml").decode()
        )["backend"].keys()
    )


def generate_fmu_from_backend(backend: str, output_path):
    """Create a new FMU at specified location using a particular backend.

    The resources making up each backend are defined in a configuration file located in the resources directory.
    Specifically each backend defines a list of source-to-destination experssions:

    ["backends/python/*.py","resources/"],

    """

    backend_manifest = toml.loads(
        pkg_resources.resource_string(__name__, "resources/backends.toml").decode()
    )["backend"][backend]

    if "files" not in backend_manifest:
        raise RuntimeError("'files' attribute is not defined in the configuration")

    # create phyiscal files in tmpdir, such that the copy/mv semantics can be implemented with function of standard lib
    with TemporaryDirectory() as tmpdir_resources, TemporaryDirectory() as tmpdir_fmu:
        tmpdir_resources = Path(tmpdir_resources)
        tmpdir_fmu = Path(tmpdir_fmu)

        # apply glob expressions from manifest to identify files and directories to copy to the FMU archive
        resource_to_output = {}

        for src, dst in backend_manifest["files"]:
            resource_to_output = {
                **resource_to_output,
                **{src: dst},
            }

        # dump all needed into a temporary directory
        # this should ensures a file structure identical to the resources directory
        for src in resource_to_output:
            file_out = tmpdir_resources / src
            makedirs(file_out.parent, exist_ok=True)

            stream = pkg_resources.resource_string(__name__, f"resources/{src}")
            with open(file_out, "wb") as f:
                f.write(stream)

        for src, dst in resource_to_output.items():

            src = tmpdir_resources / src
            dst = tmpdir_fmu / dst
            makedirs(dst.parent, exist_ok=True)
            copy(src, dst)

        shutil.copytree(tmpdir_fmu, output_path)


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


def validate_model_description(model_description: ModelDescription):
    raise NotImplementedError()
    if causality not in {
        "parameter",
        "calculatedParameter",
        "input",
        "output",
        "local",
        "independent",
    }:
        raise ValueError("invalid causality")

    if variability not in {
        "constant",
        "fixed",
        "tunable",
        "discrete",
        "continuous",
    }:
        raise ValueError("invalid variability")

    err_a = "The combinations “constant / parameter”, “constant / calculatedParameter” and “constant / input” do not make sense, since parameters and inputs are set from the environment, whereas a constant has always a value."
    err_b = "The combinations “discrete / parameter”, “discrete / calculatedParameter”, “continuous / parameter” and continuous / calculatedParameter do not make sense, since causality = “parameter” and “calculatedParameter” define variables that do not depend on time, whereas “discrete” and “continuous” define variables where the values can change during simulation."
    err_c = "For an “independent” variable only variability = “continuous” makes sense."
    err_d = "A fixed or tunable “input” has exactly the same properties as a fixed or tunable parameter. For simplicity, only fixed and tunable parameters shall be defined."
    err_d = "A fixed or tunable “output” has exactly the same properties as a fixed or tunable calculatedParameter. For simplicity, only fixed and tunable calculatedParameters shall be defined."


def _get_default_initial_for(variability: str, causality: str):
    # case (A)
    if (variability == "constant" and causality in {"output", "local"}) or (
        variability in {"fixed", "tunable"} and causality == "parameter"
    ):
        initial = "exact"
    # case (B)
    elif variability in {"fixed", "tunable"} and causality in {
        "calculatedParameter",
        "local",
    }:
        initial = "calculated"
    # case (C)
    elif variability in {"discrete", "continuous"} and causality in {"output", "local"}:
        initial = "calculated"
    # case (D)
    elif variability in {"discrete", "continuous"} and causality == "input":
        initial = None
    elif variability == "continuous" and causality == "independent":
        intial = None
    else:
        raise ValueError("invalid combination of variability and causality")


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
    number_of_event_indicators = root.get("numberOfEventIndicators", default=0)

    model_variables = []

    """ Iterate over model variables:
    <ScalarVariable name="real_a" valueReference="0" variability="continuous" causality="input">
        <Real start="0.0" />
    </ScalarVariable>
    """
    for scalar_variable in root.iter("ScalarVariable"):

        causality = scalar_variable.get("causality", default="local")
        variability = scalar_variable.get("variability", default="continuous")

        initial = scalar_variable.get("initial", default=None)
        # defaults of initial depend on causality and variablilty
        # the combinations lead to 5 different cases denoted A-E on p.50
        if initial is None:
            initial = _get_default_initial_for(variability, causality)

        var = list(scalar_variable)[0]
        start = var.get("start", default=None)
        data_type = var.tag

        model_variables.append(
            ScalarVariable(
                name=scalar_variable.get("name"),
                value_reference=scalar_variable.get("valueReference"),
                variability=variability,
                causality=causality,
                description=scalar_variable.get("description", default=None),
                initial=initial,
                start=start,
                data_type=data_type,
            )
        )

    log_categories = []
    for category in root.iter("Category"):
        log_categories.append(category.get("name"))

    model_structure = []

    # cosimulation
    co_simulation = root.find("CoSimulation")

    model_identifier = co_simulation.get("modelIdentifier")
    needs_execution_tool = co_simulation.get(
        "needsExecutionTool", default=defaults["needsExecutionTool"]
    )
    can_handle_variable_communication_step_size = co_simulation.get(
        "canHandleVariableCommunicationStepSize",
        default=defaults["canHandleVariableCommunicationStepSize"],
    )
    can_interpolate_inputs = co_simulation.get(
        "canInterpolateInputs", default=defaults["canInterpolateInputs"]
    )
    max_output_derivative_order = co_simulation.get(
        "maxOutputDerivativeOrder", default=defaults["maxOutputDerivativeOrder"]
    )
    can_run_asynchronuously = co_simulation.get(
        "canRunAsynchronuously", default=defaults["canRunAsynchronuously"]
    )
    can_be_instantiated_only_once_per_process = co_simulation.get(
        "canBeInstantiatedOnlyOncePerProcess",
        default=defaults["canBeInstantiatedOnlyOncePerProcess"],
    )
    can_not_use_memory_management_functions = co_simulation.get(
        "canNotUseMemoryManagementFunctions",
        default=defaults["canNotUseMemoryManagementFunctions"],
    )
    can_get_and_set_fmu_state = co_simulation.get(
        "canGetAndSetFMUstate", default=defaults["canGetAndSetFMUstate"]
    )
    can_serialize_fmu_state = co_simulation.get(
        "canSerializeFMUstate", default=defaults["canSerializeFMUstate"]
    )
    provides_directional_derivative = co_simulation.get(
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

    co_simulation = CoSimulation(
        model_identifier=model_identifier,
        needs_execution_tool=xs_boolean(needs_execution_tool),
        can_handle_variable_communication_step_size=xs_boolean(
            can_handle_variable_communication_step_size
        ),
        can_interpolate_inputs=xs_boolean(can_interpolate_inputs),
        max_output_derivative_order=xs_unsigned_int(max_output_derivative_order),
        can_run_asynchronuously=xs_boolean(can_run_asynchronuously),
        can_be_instantiated_only_once_per_process=xs_boolean(
            can_be_instantiated_only_once_per_process
        ),
        can_not_use_memory_management_functions=xs_boolean(
            can_not_use_memory_management_functions
        ),
        can_get_and_set_fmu_state=xs_boolean(can_get_and_set_fmu_state),
        can_serialize_fmu_state=xs_boolean(can_serialize_fmu_state),
        provides_directional_derivative=xs_boolean(provides_directional_derivative),
    )

    return ModelDescription(
        fmi_version=fmi_version,
        model_name=model_name,
        guid=guid,
        author=author,
        description=description,
        version=version,
        copyright=copyright,
        log_categories=log_categories,
        license=license,
        generation_tool=generation_tool,
        generation_date_and_time=generation_date_and_time,
        variable_naming_convention=variable_naming_convention,
        number_of_event_indicators=number_of_event_indicators,
        co_simulation=co_simulation,
        model_variables=model_variables,
        model_structure=model_structure,
    )


def export_model_description(md: ModelDescription) -> bytes:
    """Converts in memory representation of the model description into its XML representation"""

    # ---------------- write model description -------------------

    fmd = ET.Element("fmiModelDescription")
    fmd.set("fmiVersion", "2.0")
    fmd.set("modelName", md.model_name)
    fmd.set("guid", md.guid)
    fmd.set("author", md.author)
    fmd.set("generationDateAndTime", md.generation_date_and_time)
    fmd.set("variableNamingConvention", md.variable_naming_convention)
    fmd.set("generationTool", md.generation_tool)
    fmd.set("description", md.description)

    # CoSimulation
    cs = ET.SubElement(fmd, "CoSimulation")
    cs.set("modelIdentifier", md.co_simulation.model_identifier)
    cs.set(
        "needsExecutionTool", str(md.co_simulation.needs_execution_tool).lower(),
    )
    cs.set(
        "canHandleVariableCommunicationStepSize",
        str(md.co_simulation.can_handle_variable_communication_step_size).lower(),
    )
    cs.set(
        "canInterpolateInputs", str(md.co_simulation.can_interpolate_inputs).lower(),
    )

    cs.set(
        "maxOutputDerivativeOrder", str(md.co_simulation.max_output_derivative_order),
    )
    cs.set(
        "canRunAsynchronuously", str(md.co_simulation.can_run_asynchronously).lower(),
    )
    cs.set(
        "canBeInstantiatedOnlyOncePerProcess",
        str(md.co_simulation.can_be_instantiated_only_once_per_process).lower(),
    )
    cs.set(
        "canNotUseMemoryManagementFunctions",
        str(md.co_simulation.can_not_use_memory_management_functions).lower(),
    )
    cs.set(
        "canGetAndSetFMUstate", str(md.co_simulation.can_get_and_set_fmu_state).lower(),
    )
    cs.set(
        "canSerializeFMUstate", str(md.co_simulation.can_serialize_fmu_state).lower(),
    )
    cs.set(
        "providesDirectionalDerivative",
        str(md.co_simulation.provides_directional_derivatives).lower(),
    )

    # 2.2.4 p.42) Log categories:
    cs = ET.SubElement(fmd, "LogCategories")
    for ac in md.log_categories:
        c = ET.SubElement(cs, "Category")
        c.set("name", ac)

    # 2.2.7 p.47) ModelVariables
    mvs = ET.SubElement(fmd, "ModelVariables")

    variable_index = 0

    for var in md.model_variables:
        var.variability
        value_reference = str(var.value_reference)

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

        val = ET.SubElement(sv, var.data_type)

        # 2.2.7. p.48) start values
        if var.initial in {"exact", "approx"} or var.causality == "input":
            assert (
                var.start != None
            ), "a start value must be defined for intial ∈ {exact, approx}"
            val.set("start", var.start)

        variable_index += 1

    ms = ET.SubElement(fmd, "ModelStructure")

    # 2.2.8) For each output we must declare 'Outputs' and 'InitialUnknowns'
    outputs = [
        (idx + 1, o)
        for idx, o in enumerate(md.model_variables)
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
    #
    # See 2.2 p.28
    return ET.tostring(fmd, pretty_print=True, encoding="utf-8", xml_declaration=True)


def import_fmu(archive_or_dir) -> ModelDescription:
    """Reads an FMU archive and returns the parsed model description.

    In case the path points to a file, its contents will be extracted into a temporary folder.

    Note that this function assumes that model description is valid
    """

    archive_or_dir = Path(archive_or_dir)
    model_description_str = None

    if archive_or_dir.is_file():
        with TemporaryDirectory() as tmpdir, ZipFile(archive_or_dir) as zip_ref:

            tmpdir = Path(tmpdir())
            zip_ref.extractall(tmpdir)

            model_description_path = tmpdir / "modelDescription.xml"

            if not model_description_path.is_file():
                raise FileNotFoundError(
                    "No modelDescription.xml file was found inside the FMU archive"
                )

            with open(model_description_path, "r") as f:
                model_description_str = f.read()
    else:
        model_description_path = archive_or_dir / "modelDescription.xml"

        if not model_description_path.is_file():
            raise FileNotFoundError(
                "No modelDescription.xml file was found inside the FMU directory"
            )

        with open(model_description_path, "rb") as f:
            model_description_str = f.read()

    return parse_model_description(model_description_str)

