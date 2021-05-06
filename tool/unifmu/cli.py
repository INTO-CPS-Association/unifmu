import argparse
from unifmu.generate import dockerize


from unifmu.generate import get_backends, generate_fmu_from_backend

# this is the function invoked when "unifmu" is invoked from the command line
# this is defined in the setup.py file in the root of the project
def main():

    parser = argparse.ArgumentParser(
        description="Utility tool for creating and editing FMUs for FMI based co-simulation"
    )

    subparsers = parser.add_subparsers(dest="subprogram", required=True)

    subparsers.add_parser("gui", help="open graphical user interface")

    generate_parser = subparsers.add_parser(
        "generate", help="create a new FMU using a specified language-backend"
    )
    generate_parser.add_argument(
        "backend",
        choices=get_backends(),
        help="the language adapter copied into the FMU",
    )
    generate_parser.add_argument(
        "outdir",
        type=str,
        help="directory into which the FMU's resources are written (new directories will be created if needed)",
    )
    generate_parser.add_argument(
        "--dockerize", action="store_true", help="enable docker support for FMU",
    )

    args = parser.parse_args()

    if args.subprogram == "generate":
        generate_fmu_from_backend(args.backend, args.outdir)

        if args.dockerize:
            dockerize(args.backend, args.outdir)

    if args.subprogram == "gui":

        try:
            from unifmu.gui import (
                show_gui,
            )  # this requires additional dependencies installed by 'pip install .[gui]'

            show_gui()
        except ModuleNotFoundError as e:
            print(
                "Unable to start gui due to missing dependencies. The missing dependencies can be added by installing unifmu using: 'pip install .[gui]'."
                " For more info see, the documentation: https://github.com/INTO-CPS-Association/unifmu"
            )

