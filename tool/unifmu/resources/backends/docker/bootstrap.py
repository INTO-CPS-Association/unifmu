import subprocess
import sys
from argparse import ArgumentParser

import toml

if __name__ == "__main__":
    data = toml.load("launch.toml")
    backend = data["backend"]

    parser = ArgumentParser()
    parser.add_argument(
        "--use-docker-localhost",
        dest="use_docker_localhost",
        action="store_true",
        help="if true, replace occurences of 'localhost' and "
        " '127.0.0.1' with 'host.docker.internal' in --handshake-endpoint.",
    )

    args, _ = parser.parse_known_args()

    if args.use_docker_localhost:
        for idx, value in enumerate(sys.argv):
            if value == "--handshake-endpoint":
                handshake_endpoint = (
                    sys.argv[idx + 1]
                    .replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal")
                )
                sys.argv[idx + 1] = handshake_endpoint
        sys.argv.remove("--use-docker-localhost")

    # The command run by script is determined by the contents
    # of 'launch.toml' and the keyword arguments passed
    # to 'bootstrap.py'
    launch_command = data[backend]["linux"]
    final_command = launch_command + sys.argv[1:]

    subprocess.run(final_command, shell=False, check=True)
