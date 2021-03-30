#!/bin/sh

# fmu identifier from model description
uid=$(grep -oP 'guid="\K[^"]+' ../modelDescription.xml)

# additional arguments from wrapper
arg1=$1
arg2=$2

cp ../modelDescription.xml modelDescription.xml

echo "build container for $uid"

# build image
docker build -t "$uid" .

# run container
docker run --net=host --rm "$uid" "$arg1" "$arg2"