FROM ubuntu:latest
WORKDIR /fmi2api


ARG DEBIAN_FRONTEND=noninteractive

# get system wide dependencies
RUN apt -qq -y update
RUN apt -qq -y install \
    mingw-w64 \ 
    build-essential \ 
    clang \
    curl \
    cmake\
    git\
    wget\
    libxml2-dev\
    libssl-dev \
    libz-dev

# get rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-apple-darwin


RUN git clone https://github.com/tpoechtrager/osxcross
RUN wget -q https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz
RUN mv MacOSX11.3.sdk.tar.xz osxcross/tarballs/
WORKDIR /fmi2api/osxcross
RUN UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
WORKDIR /fmi2api

# ENV PATH="$(pwd)/osxcross/target/bin:$PATH"

# RUN cargo build --package fmi2api --target x86_64-pc-windows-gnu
# RUN cargo build --package fmi2api --target x86_64-pc-windows-gnu --release

# RUN cargo build --package fmi2api --target x86_64-apple-darwin --release 
# RUN cargo build --package fmi2api --target x86_64-apple-darwin