import argparse
from contextlib import contextmanager
from pathlib import Path
import logging
import shutil
from shutil import SameFileError, rmtree
import subprocess
import os
import sys
import platform
from os import makedirs
from distutils.util import strtobool


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


@contextmanager
def silent_temp_dir(logger, prefix, keep):
    # Code to acquire resource, e.g.:
    temp_path = Path(mkdtemp(prefix=prefix))
    try:
        yield temp_path
    finally:
        if not keep:
            logger.debug(f"attempting to remove {temp_path}")
            rmtree(
                temp_path,
                lambda func, path, exc_info: logger.warn(
                    f"Failed to delete {path}: {func}, {exc_info}."
                ),
            )


if __name__ == "__main__":

    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger(__file__)

    parser = argparse.ArgumentParser("Builds and tests UniFMU project.")

    # ----------- here we resolve file paths -----------------------

    s = platform.system()

    binary_basename_in = "fmi2api"
    binary_basename_out = "unifmu"

    # note that lib prefix is removed
    input_path, output_path = {
        "Linux": (f"lib{binary_basename_in}.so", f"linux64/{binary_basename_out}.so"),
        "Windows": (f"{binary_basename_in}.dll", f"win64/{binary_basename_out}.dll"),
        "Darwin": (
            f"lib{binary_basename_in}.dylib",
            f"darwin64/{binary_basename_out}.dylib",
        ),
    }[s]

    wrapper_in = Path(f"wrapper/target/debug/{input_path}").absolute().__fspath__()
    wrapper_lib = Path(
        f"tool/unifmu/resources/common/unifmu_binaries/{output_path}"
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
        "--keep-generated-fmus",
        dest="keep_generated",
        action="store_true",
        help="Use with --test-integration. Keeps the temporary folders (useful for debugging).",
        default=False,
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

    parser.add_argument(
        "--github-update-wrapper",
        dest="github_update_wrapper",
        help="utility used by build automation to commit and push compiled wrapper. "
        "If another repository has pushed changes these will be pulled before attempting "
        "to push again. This command is only meant to be used by GitHub actions!",
        action="store_true",
    )

    parser.add_argument(
        "--publish-pypi",
        dest="publish_pypi",
        help="utility for publishing package to python package index (pipy) such that a "
        " new version of UniFMU can be installed using 'pip install unifmu'.",
        action="store_true",
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

        subprocess.check_call(
            [
                "protoc",
                "-I=schemas",
                "--python_out=tool/unifmu/resources/backends/python/schemas",
                "--csharp_out=tool/unifmu/resources/backends/csharp/schemas",
                # "--java_out=tool/unifmu/resources/backends/java/",
                "unifmu_fmi2.proto",
            ]
        )
        logger.info("updated schemas")

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

        test_cases = ["csharp"]
        logger.info(
            f"Starting integration test of the following backends: {test_cases}"
        )

        with silent_temp_dir(
            logger, prefix="unifmu_test", keep=args.keep_generated
        ) as tmpdir:
            for backend in test_cases:
                fmu_path = tmpdir / backend
                generate_fmu_from_backend(backend, fmu_path)

                resources_uri = (fmu_path / "resources").absolute().as_uri()
                os.environ["UNIFMU_ADDER_RESOURCES_URI"] = resources_uri
                os.environ["UNIFMU_ADDER_LIBRARY"] = wrapper_lib
                logger.info(
                    f"running integration tests, with resource-uri: {resources_uri} and library: {wrapper_lib}"
                )

                with Chdir("wrapper"):
                    res = subprocess.run(
                        args=[
                            "cargo",
                            "test",
                            "--package",
                            "wrapper-tests",
                            "--",
                            "--nocapture",
                        ]  #  "--show-output",
                    )

                    if res.returncode != 0:
                        logger.error(f"integration tests failed for backend {backend}")
                        sys.exit(-1)

        logger.info("integration tests successful")

    if args.github_update_wrapper:

        res = subprocess.run(["git", "diff", "--quiet", "--exit-code", wrapper_lib])

        # check if wrapper has actually changed
        if res.returncode == 1:

            logger.info(f"wrapper has changed, updating wrapper for {s}")

            subprocess.run(["git", "config", "user.name", "github-actions"], check=True)
            subprocess.run(
                ["git", "config", "user.email", "github-actions@github.com"],
                check=True,
            )
            subprocess.run(["git", "pull"], check=True)
            subprocess.run(["git", "add", wrapper_lib], check=True)
            subprocess.run(
                ["git", "commit", "-m", "updated wrapper for {s} platforms"],
                check=True,
            )

            # if another repository has pushed since last pull
            # we need to do a pull followed by a push
            # since 3 binaries are being build 2 clashes are possible
            success = False
            n_tries = 5
            for i in range(n_tries):

                try:
                    subprocess.run(["git", "push"], check=True)
                    success = True
                except subprocess.CalledProcessError:
                    logger.info(
                        f"Another repository has pushed in the mean time, retry '{i+1} of '{n_tries}'"
                    )
                    subprocess.run(["git", "pull", "--rebase"], check=True)

            if success:
                logger.info("wrapper pushed succesfully")
            else:
                logger.error(f"wrapper was not pushed after '{n_tries}'")
                exit(-1)

        elif res.returncode == 0:
            logger.info(f"wrapper unchanged for {s}, no need to update")

        else:
            logger.error(
                "Git diff returned error code. There is an error in the build automation."
            )
            exit(-1)

    if args.publish_pypi:

        logger.info("building distribution using setuptools")
        subprocess.run(["python", "setup.py", "sdist", "bdist_wheel"], check=True)

        logger.info(
            "commencing publish to test repository. "
            "This step allows you to preview the changes before publishing to the real package index. "
            "Note that you need a separate account for the test package index. "
            "Go to 'https://test.pypi.org/account/register/' to register your account."
        )

        choice = input("do you want to publish to test repo? [Y/n]")

        if choice == "" or strtobool(choice):
            subprocess.run(
                [
                    "twine",
                    "upload",
                    "-r",
                    "testpypi",
                    "dist/*",
                ],
                check=True,
            )
        else:
            logger.info("skipping publish to test repo")

        choice = input(
            "do you want to publish the distribution to pypi? [y/N] (please review test repo first the link from the shell)."
        )

        if choice == "" or not strtobool(choice):
            logger.info("publishing to pypi")
        else:
            logger.info("publishing to pypi")
            subprocess.run(["twine", "upload", "dist/*"])
