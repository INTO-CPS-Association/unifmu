#!/bin/sh

# fmu identifier from model description
uid=$(grep -oP 'guid="\K[^"]+' ../modelDescription.xml)

# additional arguments from wrapper
arg1=$1
arg2=$2

# grep from 'backend = "grpc"'
# get command from container's launch.toml


cp ../modelDescription.xml container_bundle/modelDescription.xml

echo "build container for $uid"

# build image
docker build -t "$uid" .

# run container
docker run --net=host --rm "$uid" --entrypoint $args