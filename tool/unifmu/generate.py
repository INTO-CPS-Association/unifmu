import pdb
import shutil
from os import makedirs
from pathlib import Path
from shutil import copy, copytree
from tempfile import TemporaryDirectory
import shutil
from typing import List
from zipfile import ZipFile

import lxml.etree as ET
import pkg_resources
import toml

from unifmu.fmi2 import (
    ModelDescription,
    parse_model_description,
)


def list_resource_files(resource_name: str) -> List[str]:
    """Get a list containing all datafiles and directories available through setuptools pkg_resources interface

    Directories are appended a trailing slash to distinguish them from files, e.g. resources -> resources/
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
    Specifically each backend defines a list of source-to-destination expressions:

    ["backends/python/*.py","resources/"],

    """

    backend_manifest = toml.loads(
        pkg_resources.resource_string(__name__, "resources/backends.toml").decode()
    )["backend"][backend]

    if "files" not in backend_manifest:
        raise RuntimeError("'files' attribute is not defined in the configuration")

    # create physical files in tmpdir, such that the copy/mv semantics can be implemented with function of standard lib
    with TemporaryDirectory() as tmpdir_resources, TemporaryDirectory() as tmpdir_fmu:
        tmpdir_resources = Path(tmpdir_resources)
        tmpdir_fmu = Path(tmpdir_fmu)

        dirs_to_output = {}
        files_to_output = {}

        # dump all resources into a temporary directory
        # while this is not very effective, it ensures a file structure identical to the resources directory.
        # concretely it makes it easier to check which paths refer to directories or files
        for src in list_resource_files("resources"):
            file_out = tmpdir_resources / src
            makedirs(file_out.parent, exist_ok=True)

            stream = pkg_resources.resource_string(__name__, f"{src}")
            with open(file_out, "wb") as f:
                f.write(stream)

        # copy the files needed for the particular backend

        if "files" in backend_manifest:
            for src, dst in backend_manifest["files"]:
                files_to_output = {
                    **files_to_output,
                    **{src: dst},
                }

        if "dirs" in backend_manifest:
            for src, dst in backend_manifest["dirs"]:
                dirs_to_output = {
                    **dirs_to_output,
                    **{src: dst},
                }

        for src, dst in files_to_output.items():
            src = tmpdir_resources / "resources" / src

            if not src.exists():
                raise FileNotFoundError(f"The file {src} does not math any known resource")

            if not src.is_file():
                raise FileNotFoundError(
                    f"The path {src} exists, but does not refer to a file"
                )

            dst = tmpdir_fmu / dst
            makedirs(dst.parent, exist_ok=True)
            copy(src, dst)

        for src, dst in dirs_to_output.items():

            src = tmpdir_resources / "resources" / src
            dst = tmpdir_fmu / dst
            makedirs(dst.parent, exist_ok=True)
            copytree(src, dst)

        shutil.copytree(tmpdir_fmu, output_path)


def get_resource(resource_name) -> bytes:

    return pkg_resources.resource_string(__name__, resource_name)


def dockerize(backend, fmu_path):

    resources_path = Path(fmu_path) / "resources"

    # move all resouces into a subdirectory 'container_bundle'
    tmp_path = Path(fmu_path) / "tmp" / "container_bundle"
    shutil.move(resources_path, tmp_path)
    shutil.move(Path(fmu_path) / "tmp", Path(fmu_path) / "resources")

    docker_file = f"Dockerfile_{backend}"  # the "actual" Dockerfile
    all_docker_files = {
        "resources/backends/docker/launch.toml": "launch.toml",
        "resources/backends/docker/deploy.ps1": "deploy.ps1",
        "resources/backends/docker/deploy.sh": "deploy.sh",
        f"resources/backends/docker/{docker_file}": "Dockerfile",
    }
    for src, dest in all_docker_files.items():

        with open(resources_path / dest, "wb") as f:
            f.write(get_resource(src))


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



def export_model_description(md: ModelDescription) -> bytes:
    """Converts in memory representation of the model description into its XML representation"""

    # ---------------- write model description -------------------

    fmd = ET.Element("fmiModelDescription")
    fmd.set("fmiVersion", "2.0")
    fmd.set("modelName", md.modelName)
    fmd.set("guid", md.guid)
    fmd.set("author", md.author)
    fmd.set("generationDateAndTime", md.generationDateAndTime)
    fmd.set("variableNamingConvention", md.variableNamingConvention)
    fmd.set("generationTool", md.generationTool)
    fmd.set("description", md.description)

    # CoSimulation
    cs = ET.SubElement(fmd, "CoSimulation")
    cs.set("modelIdentifier", md.CoSimulation.modelIdentifier)
    cs.set(
        "needsExecutionTool", str(md.CoSimulation.needsExecutionTool).lower(),
    )
    cs.set(
        "canHandleVariableCommunicationStepSize",
        str(md.CoSimulation.canHandleVariableCommunicationStepSize).lower(),
    )
    cs.set(
        "canInterpolateInputs", str(md.CoSimulation.canInterpolateInputs).lower(),
    )

    cs.set(
        "maxOutputDerivativeOrder", str(md.CoSimulation.maxOutputDerivativeOrder),
    )
    cs.set(
        "canRunAsynchronuously", str(md.CoSimulation.canRunAsynchronuously).lower(),
    )
    cs.set(
        "canBeInstantiatedOnlyOncePerProcess",
        str(md.CoSimulation.canBeInstantiatedOnlyOncePerProcess).lower(),
    )
    cs.set(
        "canNotUseMemoryManagementFunctions",
        str(md.CoSimulation.canNotUseMemoryManagementFunctions).lower(),
    )
    cs.set(
        "canGetAndSetFMUstate", str(md.CoSimulation.canGetAndSetFMUstate).lower(),
    )
    cs.set(
        "canSerializeFMUstate", str(md.CoSimulation.canSerializeFMUstate).lower(),
    )
    cs.set(
        "providesDirectionalDerivative",
        str(md.CoSimulation.providesDirectionalDerivative).lower(),
    )

    # 2.2.4 p.42) Log categories:
    cs = ET.SubElement(fmd, "LogCategories")
    for ac in md.logCategories:
        c = ET.SubElement(cs, "Category")
        c.set("name", ac)

    # 2.2.7 p.47) ModelVariables
    mvs = ET.SubElement(fmd, "ModelVariables")

    variable_index = 0

    for var in md.modelVariables:
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

        val = ET.SubElement(sv, var.dataType)

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
        for idx, o in enumerate(md.modelVariables)
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

