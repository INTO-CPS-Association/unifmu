import os
import subprocess

if __name__ == "__main__":
    endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"]
    dispatcher_endpoint = os.environ["UNIFMU_DISPATCHER_ENDPOINT"].replace("0.0.0.0", "host.docker.internal")
    os.environ["UNIFMU_DISPATCHER_ENDPOINT"] = dispatcher_endpoint
    subprocess.call(["python", "backend.py"])
    