import argparse

from unifmu.gui import show_gui

# this is the function invoked when "unifmu" is invoked from the command line
# this is defined in the setup.py file in the root of the project
def main():

    parser = argparse.ArgumentParser(
        description="Utility tool for creating and editing FMUs for FMI based co-simulation"
    )

    subparsers = parser.add_subparsers(dest="subprogram", required=True)

    subparsers.add_parser("gui")

    args = parser.parse_args()

    if args.subprogram == "gui":
        show_gui()

