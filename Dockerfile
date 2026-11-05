FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends curl zip unzip ca-certificates\
 && rm -rf /var/lib/apt/lists/*

# install protoc
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v27.3/protoc-27.3-linux-x86_64.zip
RUN unzip -oq protoc-27.3-linux-x86_64.zip -d /usr

WORKDIR /app