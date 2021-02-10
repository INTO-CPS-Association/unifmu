import argparse
from os import mkdir, popen, system
from pathlib import Path
import logging
import pathlib
import shutil
from shutil import SameFileError, rmtree
import subprocess
import os
import sys
from sys import executable, platform
import platform
from tempfile import TemporaryDirectory
from os import makedirs


from unifmu.generate import generate_fmu_from_backend, get_backends


class Chdir:
    def __init__(self, wd):
        wd = Path(wd)
        self.old_wd = Path.cwd()
        self.wd = wd

    def __enter__(self):
        os.chdir(self.wd)

    def __exit__(self, type, value, traceback):
        os.chdir(self.old_wd)


if __name__ == "__main__":

    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__file__)

    parser = argparse.ArgumentParser()

    # ----------- here we resolve file paths -----------------------

    s = platform.system()

    binary_basename = "unifmu"

    # note that lib prefix is removed
    input, output = {
        "Linux": (f"lib{binary_basename}.so", f"linux64/{binary_basename}.so"),
        "Windows": (f"{binary_basename}.dll", f"win64/{binary_basename}.dll"),
        "Darwin": (f"lib{binary_basename}.dylib", f"darwin64/{binary_basename}.dylib"),
    }[s]

    wrapper_in = Path(f"wrapper/target/debug/{input}").absolute().__fspath__()
    wrapper_lib = Path(
        f"tool/unifmu/resources/common/unifmu_binaries/{output}"
    ).absolute()
    makedirs(wrapper_lib.parent, exist_ok=True)
    wrapper_lib = wrapper_lib.__fspath__()

    # -------------- parse args -------------------------

    parser.add_argument(
        "--update-wrapper",
        "-u",
        dest="update_wrapper",
        action="store_true",
        help="updates the shared object inside the example FMUs",
    )

    parser.add_argument(
        "--test-integration",
        dest="test_integration",
        action="store_true",
        help="run rust integration tests",
    )

    parser.add_argument(
        "--export-examples",
        dest="export_examples",
        action="store_true",
        help="copy example FMUs to the examples directory",
    )

    parser.add_argument(
        "--update-schemas",
        dest="update_schemas",
        action="store_true",
        help="update resource files generated based on FlatBuffer schemas",
    )

    args = parser.parse_args()

    if args.update_wrapper:

        logger.info("building wrapper")
        with Chdir("wrapper"):
            res = subprocess.Popen(args=["cargo", "build"]).wait()

        if res != 0:
            logger.error("wrapper failed to build")
            sys.exit(-1)

        logger.info(
            f"wrapper was build, copying from '{wrapper_in}' to '{wrapper_lib}'"
        )

        try:
            shutil.copy(src=wrapper_in, dst=wrapper_lib)
        except SameFileError:
            pass

        logger.info("wrapper updated")

    if args.update_schemas:

        try:
            targets = [
                ("java", "tool/unifmu/resources/backends/java_fmu/src/main/java", []),
                (
                    "python",
                    "tool/unifmu/resources/backends/python_fmu/flatbuffers",
                    ["--gen-onefile"],
                ),
                (
                    "csharp",
                    "tool/unifmu/resources/backends/csharp_fmu/",
                    ["--gen-onefile"],
                ),
            ]
            for lang, outdir, kwargs in targets:
                res = subprocess.Popen(
                    [
                        "tmp/flatc",
                        f"--{lang}",
                        "-o",
                        outdir,
                        "schemas/unifmu_fmi2.fbs",
                        *kwargs,
                    ]
                ).wait()

                if res != 0:
                    logger.error(
                        f"Failed compile flatbuffer schemas, for target language: {lang}"
                    )

        except Exception:
            logger.error(
                f"FlatBuffer could failed to execute. Ensure that it is installed and available in the systems path",
                exc_info=True,
            )

    if args.export_examples:

        for b in get_backends():
            outdir = Path(f"examples/{b}_fmu")
            if outdir.is_dir():
                rmtree(outdir)
            generate_fmu_from_backend(b, outdir)

    if args.test_integration:

        if not args.update_wrapper:
            logger.warn(
                "program was called without --update-wrapper. Integration tests will use the existing wrapper in the resources directory."
            )

        # export test examples into tmp directory and execute tests
        from tempfile import mkdtemp

        # with TemporaryDirectory() as tmpdir:
        

        for backend in ["python_schemaless_rpc"]:
            tmpdir = Path(mkdtemp())
            fmu_path = tmpdir / "fmu"
            generate_fmu_from_backend(backend, fmu_path)

            resources_uri = (fmu_path / "resources").absolute().as_uri()
            os.environ["UNIFMU_ADDER_RESOURCES_URI"] = resources_uri
            os.environ["UNIFMU_ADDER_LIBRARY"] = wrapper_lib
            logger.info(
                f"running integration tests, with resource-uri: {resources_uri} and library: {wrapper_lib}"
            )

            with Chdir("Wrapper"):

                res = subprocess.Popen(args=["cargo", "test", "--", "--show-output"]).wait()

                if res != 0:
                    logger.error(f"integration tests failed for backend {backend}")
                    sys.exit(-1)

        logger.info("integration tests successful")

