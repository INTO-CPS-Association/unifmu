import argparse
from pathlib import Path

import logging
import shutil
import subprocess
import os
import sys


class Chdir():
    def __init__(self, wd):
        self.old_wd = Path.cwd()
        self.wd = wd

    def __enter__(self):
        os.chdir(self.wd)

    def __exit__(self, type, value, traceback):
        os.chdir(self.old_wd)


def path_to_host_binary() -> Path:
    return Path("tests/examples/python_fmu/binaries/linux64/libunifmu.so").absolute()


if __name__ == "__main__":

    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__file__)

    parser = argparse.ArgumentParser()

    parser.add_argument("--update-wrapper", "-u", dest="update_wrapper", action="store_true",
                        help="updates the shared object inside the example FMUs")

    parser.add_argument("--test-rust", dest="test_rust",
                        action="store_true", help="run rust integration tests")

    parser.add_argument("--test-c", dest="test_c",
                        action="store_true", help="run C integration tests")

    args = parser.parse_args()

    if(args.update_wrapper):

        logger.info("building wrapper")
        res = subprocess.Popen(args=["cargo", "build"]).wait()

        if(res != 0):
            logger.error("wrapper failed to build")
            sys.exit(-1)

        binary_in = Path("target/debug/libunifmu.so")
        binary_out = path_to_host_binary()
        logger.info(
            f"wrapper was build, copying from '{binary_in}' to '{binary_out}'")
        shutil.copy(src=binary_in, dst=binary_out)
        logger.info("wrapper updated")

    if(args.test_c):

        build_dir = Path("tests/c_tests/build")
        old_wd = Path.cwd()

        build_dir.mkdir(exist_ok=True)

        with Chdir(build_dir):
            logger.info("configuring cmake")
            res = subprocess.Popen(args=["cmake", ".."]).wait()

            if(res != 0):
                logger.error("unable to configure cmake")
                sys.exit(-1)

            logger.info("building tests")
            res = subprocess.Popen(args=["cmake", "--build", "."]).wait()

            if(res != 0):
                logger.error("unable to compile C integration tests")
                sys.exit(-1)

        logger.info("running C integration tests")

        library_path = path_to_host_binary()
        resources_uri = (
            Path.cwd() / "tests/examples/python_fmu/resources").as_uri()
        test_executable = (
            Path.cwd() / "tests/c_tests/build/integration_tests").absolute()
        res = subprocess.Popen(
            args=[test_executable, library_path, resources_uri]).wait()

        if(res != 0):
            logger.error("C integration tests failed")
            sys.exit(-1)
