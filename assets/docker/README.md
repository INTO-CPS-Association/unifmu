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

## Networking

### Linux and macOS

On Linux and macOS the container the network is shared between the host machine and all containers.
This is done by setting `--net=host` when invoking `docker run`.
In effect, a dockerized-FMU is equivalent to its original counterpart from the perspective of the network for the two platforms.

### Windows

However, the `--net=host` option is currently not supported from windows.
To provide compatability with Windows another strategy is relying on publishing a port via the `--publish ${port_host}:${port_container}`.
Here the `deploy.ps1` will attempt to start the container with a random port which is used for both the host and container port, i.e. `${port_host} == ${port_container}`.

The availability and uniqueness of the port is ensured by `docker run`, since and error is raised in case of:

- host port is used by another process
- host port has already been published by another container instance.

Additionally, to distinguish the "localhost" address of the host from the localhost of the container, the special alias `host.docker.internal` is used to replace occurences of `127.0.0.1` or `localhost` in the handshake endpoint.
In this way the slave-backend running in the container knows to connect to the host's localhost and not the containers localhost.

The substitution is carried out by the `bootstrap.py` script invoked when the container is started.

```python
if platform.system() == "Windows":
    for idx, value in enumerate(sys.argv):
        if value == "--handshake-endpoint":
            handshake_endpoint = (
                sys.argv[idx + 1]
                .replace("localhost", "host.docker.internal")
                .replace("127.0.0.1", "host.docker.internal")
            )
            sys.argv[idx + 1] = handshake_endpoint
```


## Rebuilding image
https://docs.docker.com/compose/reference/up/
```
docker-compose up --build --force-recreate
```