import argparse
from os import popen, system
from pathlib import Path

import logging
import shutil
from shutil import SameFileError
import subprocess
import os
import sys
from sys import platform
import platform


class Chdir:
    def __init__(self, wd):
        self.old_wd = Path.cwd()
        self.wd = wd

    def __enter__(self):
        os.chdir(self.wd)

    def __exit__(self, type, value, traceback):
        os.chdir(self.old_wd)


def path_to_host_binary() -> Path:

    binary_basename = "unifmu"

    system_to_binary_inout_tuple = {
        "Linux": (f"lib{binary_basename}.so", f"linux64/{binary_basename}.so"),
        "Windows": (f"{binary_basename}.dll", f"win64/{binary_basename}.dll"),
        "Darwin": (f"lib{binary_basename}.dylib", f"darwin64/{binary_basename}.dylib"),
    }

    input, output = system_to_binary_inout_tuple[platform.system()]
    binary_in = Path.cwd() / "target" / "debug" / input
    binary_out = Path.cwd() / "tests" / "examples" / "python_fmu" / "binaries" / output

    return binary_in, binary_out


def path_to_c_executable() -> Path:

    system_to_executable = {
        "Linux": "integration_tests",
        "Windows": "Debug/integration_tests.exe",
        "Darwin": "integration_tests",
    }

    return (
        Path.cwd()
        / "tests"
        / "c_tests"
        / "build"
        / system_to_executable[platform.system()]
    )


if __name__ == "__main__":

    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__file__)

    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--update-wrapper",
        "-u",
        dest="update_wrapper",
        action="store_true",
        help="updates the shared object inside the example FMUs",
    )

    parser.add_argument(
        "--test-rust",
        dest="test_rust",
        action="store_true",
        help="run rust integration tests",
    )

    parser.add_argument(
        "--test-c", dest="test_c", action="store_true", help="run C integration tests"
    )

    args = parser.parse_args()

    library_in, library_out = path_to_host_binary()

    if args.update_wrapper:

        logger.info("building wrapper")
        res = subprocess.Popen(args=["cargo", "build"]).wait()

        if res != 0:
            logger.error("wrapper failed to build")
            sys.exit(-1)

        logger.info(
            f"wrapper was build, copying from '{library_in}' to '{library_out}'"
        )

        try:
            shutil.copy(src=library_in, dst=library_out)
        except SameFileError:
            pass

        logger.info("wrapper updated")

    if args.test_rust:
        res = subprocess.Popen(["cargo", "test"]).wait()

        if res != 0:
            logger.error("Rust test failed")
            sys.exit(0)

    if args.test_c:

        build_dir = Path("tests/c_tests/build")
        old_wd = Path.cwd()

        build_dir.mkdir(exist_ok=True)

        with Chdir(build_dir):
            logger.info("configuring cmake")
            res = subprocess.Popen(args=["cmake", ".."]).wait()

            if res != 0:
                logger.error("unable to configure cmake")
                sys.exit(-1)

            logger.info("building tests")
            res = subprocess.Popen(args=["cmake", "--build", "."]).wait()

            if res != 0:
                logger.error("unable to compile C integration tests")
                sys.exit(-1)

        logger.info("running C integration tests")

        resources_uri = (Path.cwd() / "tests/examples/python_fmu/resources").as_uri()
        test_executable = path_to_c_executable()
        res = subprocess.Popen(
            args=[test_executable, library_out, resources_uri]
        ).wait()

        if res != 0:
            logger.error("C integration tests failed")
            sys.exit(-1)

        logger.info("C integration tests successful")
