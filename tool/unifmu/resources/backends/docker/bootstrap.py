import subprocess

import toml

if __name__ == "__main__":
    data = toml.load("launch.toml")
    backend = data["backend"]
    entrypoint = data["command"][backend]["linux"]
    subprocess.run(entrypoint, shell=False, check=True)
