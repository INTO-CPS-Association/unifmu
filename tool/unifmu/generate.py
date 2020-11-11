import datetime
from pathlib import Path
from tempfile import TemporaryDirectory
import xml.etree.ElementTree as ET
from unifmu.fmi2 import ModelDescription


def generate_fmu(md: ModelDescription, path, backend: str, zipped: bool):
    """Creates new FMU archive based on the specified model description and backend.
    """

    with TemporaryDirectory() as tmpdir:

        tmpdir = Path(tmpdir)

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
        cs.set("needsExecutionTool", md.co_simulation.needs_execution_tool)
        cs.set(
            "canNotUseMemoryManagementFunctions",
            md.co_simulation.can_not_use_memory_management_functions,
        )
        cs.set(
            "canHandleVariableCommunicationStepSize",
            md.co_simulation.can_handle_variable_communication_step_size,
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
                val.set("start", val.start)

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
            md: bytes = ET.tostring(
                fmd,
                encoding="utf-8",
                xml_declaration=True
                # fmd, pretty_print=True, encoding="utf-8", xml_declaration=True
            )

        except Exception as e:
            raise RuntimeError(
                f"Failed to parse model description. Write resulted in error: {e}"
            ) from e

        md_out_path = tmpdir / "modelDescription.xml"

        with open(md_out_path, "w") as f:
            f.write(md)

    # ------------------- copy backend ----------------------------


def import_fmu(path) -> ModelDescription:
    raise NotImplementedError()
