import argparse
import logging
import os
from pathlib import Path

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__file__)


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--model-description",
        action="store_true",
        help="Generate modelDescription.xml one directory above this script and exit.",
    )
    parser.add_argument(
        "--pack",
        action="store_true",
        help="Create an .fmu package from the project contents and exit.",
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Optional output path for the generated .fmu file.",
    )
    return parser.parse_args()

def pack_fmu(output: str | None = None):
    import zipfile
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    package_name = project_root.name
    output_path = Path(output).expanduser().resolve() if output else project_root / f"{package_name}.fmu"

    model_description_found = False

    with zipfile.ZipFile(output_path, mode="w", compression=zipfile.ZIP_DEFLATED) as zf:

        for path in project_root.iterdir():
            if path.resolve() == output_path.resolve() or path.name.lower().endswith("*.fmu") or path.name.lower().startswith(".") or not any([True for pfx in ['binaries','resources','modelDescription.xml'] if path.name.startswith(pfx)]):
                continue
            if path.is_dir():
                for file_path in path.rglob("*"):
                    if file_path.is_file():
                        zf.write(file_path, arcname=file_path.relative_to(project_root))
            elif path.is_file():
                if path.name=="modelDescription.xml":
                    model_description_found = True
                zf.write(path, arcname=path.name)

        if not model_description_found:
            import fmi2_model_description
            import model
            my_model = model.Model(None)
            xml_content = fmi2_model_description.generate_model_description_xml(my_model)
            zf.writestr("modelDescription.xml", xml_content)


    logger.info("FMU written to %s", output_path)


def generate_model_description_file():

    import fmi2_model_description
    import model
    my_model = model.Model(None)
    xml_content = fmi2_model_description.generate_model_description_xml(my_model)
    output_path = Path(__file__).resolve().parent.parent / "modelDescription.xml"
    output_path.write_text(xml_content, encoding="utf-8")
    logger.info("model description written to %s", output_path)


def run_backend():
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    logger.info(f"dispatcher endpoint received: {dispatcher_endpoint}")
    from backend import Backend
    backend = Backend()
    backend.connect_to_endpoint(dispatcher_endpoint)
    backend.handshake()
    backend.command_reply_loop()

if __name__ == "__main__":
    """
    Main function of this FMU. If called with no arguments the simulation backend is started.
    
    The user can use this to produce a model description or to pack the FMU.
    """
    args = parse_args()
    if args.model_description:
        generate_model_description_file()
    elif args.pack:
        pack_fmu(args.output)
    else:
        run_backend()
