from os import makedirs
from pathlib import Path
from shutil import copy, copytree
from tempfile import TemporaryDirectory, mkdtemp, tempdir
import shutil
import fnmatch

import tempfile
from typing import List

# import xml.etree.ElementTree as ET
import pkg_resources
import lxml.etree as ET
import toml

from unifmu.fmi2 import ModelDescription


def generate_fmu(md: ModelDescription, output_path, backend: str):
    """Creates new FMU archive based on the specified model description and backend.
    """

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


def generate_fmu_from_backend(backend: str, output_path):
    """ We need to jump thorugh some hoops to get a hold of the datafiles stored by setuptools
    """

    # since setuptools may compress files, we cant access we need to use some tricks to get these
    def collect_resource_names(resource_name: str, list: List[str]):

        # base case
        if not pkg_resources.resource_isdir(__name__, resource_name):
            list.append(resource_name)

        # recursive case
        else:
            children = pkg_resources.resource_listdir(__name__, resource_name)

            resource_names_extended = [f"{resource_name}/{c}" for c in children]

            for child_resource in resource_names_extended:
                collect_resource_names(child_resource, list)

    # we collect a list of resource files, on which we can run glob expression from backends.toml
    resources = []
    collect_resource_names("resources", resources)

    backend_manifest = toml.loads(
        pkg_resources.resource_string(__name__, "resources/backends.toml").decode()
    )["backend"][backend]

    with TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        tmpdir = Path(mkdtemp())

        resource_to_output = {}

        if "files" in backend_manifest:

            for pattern, dst in backend_manifest["files"]:
                pattern = "resources/" + pattern

                resource_to_output = {
                    **resource_to_output,
                    **{match: dst for match in fnmatch.filter(resources, pattern)},
                }

            for resource, dst in resource_to_output.items():

                out: Path = tmpdir / dst / Path(resource).name

                makedirs(out.parent, exist_ok=True)
                assert out.parent.is_dir()

                print(f"writing {resource} to {out}")
                stream = pkg_resources.resource_string(__name__, resource)

                with open(out, "wb") as f:
                    f.write(stream)

        copytree(tmpdir, output_path)


def import_fmu(path) -> ModelDescription:
    raise NotImplementedError()
