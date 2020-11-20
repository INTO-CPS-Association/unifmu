from gui import md
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
import lxml.etree as ET
import toml

from unifmu.fmi2 import ModelDescription


def generate_fmu(md: ModelDescription, output_path, backend: str):
    """Creates new FMU archive based on the specified model description and backend."""

    with TemporaryDirectory() as tmpdir:

        fmu_dir_tmp = Path(tmpdir)

        # ---------------- write model description -------------------

        fmd = ET.Element("fmiModelDescription")
        fmd.set("fmiVersion", "2.0")
        fmd.set("modelName", md.model_name)
        fmd.set("guid", md.guid)
        fmd.set("author", md.author)
        fmd.set("generationDateAndTime", md.generation_date_and_time)
        fmd.set("variableNamingConvention", md.variable_naming_convention)
        fmd.set("generationTool", md.generation_tool)

        #
        cs = ET.SubElement(fmd, "CoSimulation")
        cs.set("modelIdentifier", md.co_simulation.model_identifier)
        cs.set("needsExecutionTool", str(md.co_simulation.needs_execution_tool))
        cs.set(
            "canNotUseMemoryManagementFunctions",
            str(md.co_simulation.can_not_use_memory_management_functions),
        )
        cs.set(
            "canHandleVariableCommunicationStepSize",
            str(md.co_simulation.can_handle_variable_communication_step_size),
        )

        # 2.2.4 p.42) Log categories:
        cs = ET.SubElement(fmd, "LogCategories")
        for ac in md.log_categories:
            c = ET.SubElement(cs, "Category")
            c.set("name", ac)

        # 2.2.7 p.47) ModelVariables
        mvs = ET.SubElement(fmd, "ModelVariables")

        variable_index = 0

        type_to_fmitype = {
            "real": "Real",
            "integer": "Integer",
            "boolean": "Boolean",
            "string": "String",
        }

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

            val = ET.SubElement(sv, type_to_fmitype[var.data_type])

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

        try:
            # FMI requires encoding to be encoded as UTF-8 and contain a header:
            #
            # See 2.2 p.28
            md_xml: bytes = ET.tostring(
                fmd, pretty_print=True, encoding="utf-8", xml_declaration=True
            )

        except Exception as e:
            raise RuntimeError(
                f"Failed to parse model description. Write resulted in error: {e}"
            ) from e

        md_out_path = fmu_dir_tmp / "modelDescription.xml"

        with open(md_out_path, "wb") as f:
            f.write(md_xml)

        # ------------------- copy backend ----------------------------

        binaries_dir = [
            fmu_dir_tmp / "binareis" / ext for ext in ["linux64", "win64", "darwin64"]
        ]

        # ------------------ copy to output directory ----------------
        assert fmu_dir_tmp.exists()
        shutil.copytree(fmu_dir_tmp, output_path)


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


def parse_model_description(xml_str: str) -> ModelDescription:

    pass


def import_fmu(archive_or_dir) -> ModelDescription:
    """Reads an FMU archive and returns the parsed model description.

    In case the path points to a file, its contents will be extracted into a temporary folder.

    Note that this function assumes that model description is valid
    """

    archive_or_dir = Path(archive_or_dir)
    model_description_xml = None

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
                model_description_xml = ET.parse(f)
    else:
        model_description_path = archive_or_dir / "modelDescription.xml"

        if not model_description_path.is_file():
            raise FileNotFoundError(
                "No modelDescription.xml file was found inside the FMU directory"
            )

        with open(model_description_path, "r") as f:
            model_description_xml = ET.parse(f)

    print(model_description_xml)

    md = ModelDescription()
