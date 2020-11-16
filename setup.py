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
    "tests": [],
    "gui": ["wxpython"]
}
_extras_require["dev"] = _extras_require["docs"] + _extras_require["tests"]

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
    install_requires=["zmq", "lxml", "toml"],
    extras_require=_extras_require,
    # resources needed by the CLI to generate and export
    package_data={"unifmu": get_resource_files()},
    include_package_data=True,
    python_requires=">=3.5",
    entry_points={"console_scripts": ["unifmu=unifmu.cli:main"]},
)
