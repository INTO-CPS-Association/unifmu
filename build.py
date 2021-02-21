import argparse
from pathlib import Path
import logging
import shutil
from shutil import SameFileError, rmtree
import subprocess
import os
import sys
from sys import platform
import platform
from tempfile import TemporaryDirectory
from os import makedirs

from grpc_tools.protoc import _protoc_compiler

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
        help="update resource files generated based on Protobuf schemas",
    )

    args = parser.parse_args()

    if args.update_wrapper:

        logger.info("building wrapper")
        with Chdir("wrapper"):
            res = subprocess.run(args=["cargo", "build"])

        if res.returncode != 0:
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
        schema = "unifmu_fmi2.proto"
        schema_include_dir = "schemas"

        # the protoc compiler requires language specific extensions to generate grpc
        # code for different targets. The recommended way of getting these is using
        # the de-facto package manager of the language such.
        #
        # For example:
        # * python: pip install grpc-tools
        # * C#: dotnet add package Grpc.Tools
        #
        # For C# the compilation of *.proto files is integrated in build process
        # as such it must be copied into the resources for the C# backend.

        def generate_python(outdir):
            """Generating rpc components requires a plugin for the protocol buffer compiler
            The recommended way is to get the compiler bundled with a plugin trough 'grpc-tools' package on PyPI."""
            protoc_args = [
                s.encode()
                for s in [
                    f"--proto_path={schema_include_dir}",
                    f"--python_out={outdir}",
                    f"--grpc_python_out={outdir}",
                    (
                        Path(schema_include_dir) / schema
                    ).__fspath__(),  # unlike invoking protoc, it seems schema needs absolute path
                ]
            ]
            _protoc_compiler.run_main(protoc_args)

        def generate_java(outdir):
            res = subprocess.run(
                [
                    "protoc",
                    "-I",
                    schema_include_dir,
                    f"--java_out={outdir}",
                    schema,
                ]
            ).check_returncode()

        def generate_csharp(outdir):
            shutil.copyfile(Path(schema_include_dir) / schema, Path(outdir) / schema)

        generate_commands = [
            ("python", "tool/unifmu/resources/backends/python/", generate_python),
            (
                "java",
                "tool/unifmu/resources/backends/java_fmu/src/main/java/",
                generate_java,
            ),
            (
                "csharp",
                "tool/unifmu/resources/backends/csharp/schemas",
                generate_csharp,
            ),
        ]
        logger.info(
            f"updating schemas for target languages '{[lang for lang, _ ,_ in generate_commands]}'"
        )
        for lang, outdir, cmd in generate_commands:
            try:
                makedirs(outdir, exist_ok=True)
                cmd(outdir)
                logger.info(f"Updated schemas for target '{lang}'")
            except Exception:
                logger.critical(
                    f"Failed to update schemas for target language '{lang}', an exception was raised during the process.",
                    exc_info=True,
                )
                sys.exit(1)

    ####################################### EXPORT FMU EXAMPLES #################################################
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

        for backend in ["python_schemaless_rpc", "python_grpc"]:  # "python_grpc" "python_schemaless_rpc" "csharp"
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

                res = subprocess.run(
                    args=["cargo", "test", "--", "--nocapture"]  #  "--show-output",
                )

                if res.returncode != 0:
                    logger.error(f"integration tests failed for backend {backend}")
                    sys.exit(-1)

        logger.info("integration tests successful")
