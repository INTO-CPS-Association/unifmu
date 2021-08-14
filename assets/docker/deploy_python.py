import os
import subprocess

if __name__ == "__main__":
    endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"].replace(
        "127.0.0.1", "host.docker.internal"
    )
    os.environ["UNIFMU_DISPATCHER_ENDPOINT"] = dispatcher_endpoint

    subprocess.call(["python", "backend.py"])
