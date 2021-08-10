# Dockerized FMU

- how to add dependency
- for each process a new container is instantiated, when are images rebuilt?
- explain what the container bundle is

## When does the image rebuild?

By default the image is only build once which means that updates to the FMU's resources on the host machine are not automatically updated inside the image.
Two strategies for rebuilding the image is:

1. Delete the existing image through docker.
2. Force docker to rebuild every time by passing `--no-cache` to `docker build` defined in `deploy.ps1` and `deploy.sh`.

## How to choose a backend