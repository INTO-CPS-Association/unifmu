FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

# get system wide dependencies
RUN apt -qq -y update
RUN apt -qq -y install \
  mingw-w64 \
  # build-essential \
  clang \
  curl \
  cmake\
  git\
  libxml2-dev\
  libssl-dev \
  libz-dev \
  zip \
  unzip \
  xz-utils

# Install packages required for testing and backends
RUN apt -qq -y install openjdk-17-jre python3-wheel python3 dotnet-sdk-8.0

# install protoc
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v27.3/protoc-27.3-linux-x86_64.zip
RUN unzip -oq protoc-27.3-linux-x86_64.zip -d /usr

# get macos toolchain
WORKDIR /usr/
RUN git clone https://github.com/tpoechtrager/osxcross
RUN curl -LO https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz
RUN mv MacOSX11.3.sdk.tar.xz osxcross/tarballs/
WORKDIR /usr/osxcross/
RUN UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh

# get rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-apple-darwin

WORKDIR /workdir