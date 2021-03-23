from pathlib import Path, PurePosixPath
from setuptools import setup, find_packages


# setuptools does not support ** in glob
# https://github.com/pypa/setuptools/issues/1806
def get_resource_files():

    files = [
        "/".join(PurePosixPath(p).__fspath__().split("/")[2:])
        for p in Path("tool/unifmu/resources").rglob("*")
        if p.is_file()
    ]
    return files


long_description = """
# UniFMU
UniFMU is a set of tools for developing functional-mockup-units (FMUs) using the full power of Python.

## Documentation
See the online documentation on how to use to tool and FMI in general see the INTO-CPS online documentation:

https://into-cps-application.readthedocs.io
"""

_extras_require = {
    "docs": [
        "Sphinx",
        "sphinx_rtd_theme",
        "sphinx-autoapi",
        "sphinxcontrib-bibtex",
        "sphinxcontrib-programoutput",
    ],
    # purely used by wxwidget based gui in python, i.e. CLI use does not require these
    "gui": ["wxpython", "PyPubSub"],
    # dependencies necessary only for generating protobuf schemas,
    # i.e. not necessary at runtime when using fmus FMUs:
    # - protoc-wheel-0: used to obtain `protoc` program used to generate java code
    # - grpcio-tools: used to generate python code
    "protobuf-schema-generation": ["protoc-wheel-0", "grpcio-tools"],
    # dependencies used by at runtime by the Python backend.
    # there are two variants of the python backends, using the dependencies:
    # - grpc-based: used `protobuf` and `grpcio`
    # - schemaless: uses `pyzmq`
    "python-backend": ["protobuf", "grpcio", "pyzmq"],
}
_extras_require["dev"] = (
    _extras_require["docs"]
    + _extras_require["gui"]
    + _extras_require["python-backend"]
    + _extras_require["protobuf-schema-generation"]
)

setup(
    name="unifmu",
    version="0.0.1",
    author="INTO-CPS Association",
    description="A set of tools for developing functional-mockup-units (FMUs) implemented in any language.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/INTO-CPS-Association/unifmu",
    packages=find_packages("tool"),
    package_dir={"": "tool"},
    project_urls={
        "Bug Tracker": "https://github.com/INTO-CPS-Association/unifmu/issues",
        "Documentation": "https://into-cps-application.readthedocs.io/en/latest/submodules/unifmu/docs/index.html",
        "Source Code": "https://github.com/INTO-CPS-Association/unifmu",
    },
    install_requires=["lxml", "toml"],
    extras_require=_extras_require,
    # resources needed by the CLI to generate and export
    package_data={"unifmu": get_resource_files()},
    include_package_data=True,
    python_requires=">=3.7",
    entry_points={"console_scripts": ["unifmu=unifmu.cli:main"]},
)
