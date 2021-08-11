# replace localhost address in container to use host machine
export UNIFMU_DISPATCHER_ENDPOINT=${UNIFMU_DISPATCHER_ENDPOINT//127.0.0.1/host.docker.internal}

python backend.py