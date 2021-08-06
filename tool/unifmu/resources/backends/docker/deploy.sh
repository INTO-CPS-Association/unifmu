#!/bin/sh

# move model description 
cp ../modelDescription.xml container_bundle/modelDescription.xml

echo "build container for $UNIFMU_GUID"

# build image
docker build -t "$UNIFMU_GUID" .

# run container
# docker run -p "$UNIFMU_DISPATCHER_ENDPOINT_PORT":"$UNIFMU_DISPATCHER_ENDPOINT_PORT" --rm "$UNIFMU_GUID" "$UNIFMU_LAUNCH_LINUX"
docker run --publish-all --rm "$UNIFMU_GUID" "$UNIFMU_LAUNCH_LINUX"
