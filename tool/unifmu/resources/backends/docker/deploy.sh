#!/bin/sh

# fmu identifier from model description
uid=$(grep -oP 'guid="\K[^"]+' ../modelDescription.xml)

cp ../modelDescription.xml container_bundle/modelDescription.xml

echo "build container for $uid"

# build image
docker build -t "$uid" .

# run container
docker run --net=host --rm "$uid" python3 bootstrap.py "$1" "$2"