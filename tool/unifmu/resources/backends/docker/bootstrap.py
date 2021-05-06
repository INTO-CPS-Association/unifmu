import subprocess
import sys
import platform

import toml

if __name__ == "__main__":
    data = toml.load("launch.toml")
    backend = data["backend"]

    # In cases where the binary is running on localhost
    # localhost or 127.0.0.1 must be replaced with
    # 'host.docker.internal' such that the container
    # knows to connect to the "host's" localhost.
    for idx, value in enumerate(sys.argv):
        if value == "--handshake-endpoint":
            handshake_endpoint = (
                sys.argv[idx + 1]
                .replace("localhost", "host.docker.internal")
                .replace("127.0.0.1", "host.docker.internal")
            )
            sys.argv[idx + 1] = handshake_endpoint

    # The command run by script is determined by the contents
    # of 'launch.toml' and the keyword arguments passed
    # to 'bootstrap.py'
    launch_command = data[backend]["linux"]
    final_command = launch_command + sys.argv[1:]

    subprocess.run(final_command, shell=False, check=True)
