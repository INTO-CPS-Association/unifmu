FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG TARGETARCH

# .NET 8 runtime is still required by the net8.0 C# backend.
RUN apt-get update -qq \
    && apt-get install -qq -y --no-install-recommends \
      build-essential \
      bzip2 \
      ca-certificates \
      clang \
      cmake \
      cpio \
      curl \
      dotnet-sdk-10.0 \
      dotnet-runtime-8.0 \
      gcc-aarch64-linux-gnu \
      gcc-x86-64-linux-gnu \
      git \
      libc6-dev-amd64-cross \
      libc6-dev-arm64-cross \
      libbz2-dev \
      liblzma-dev \
      libssl-dev \
      libxml2-dev \
      libz-dev \
      mingw-w64 \
      openjdk-21-jdk-headless \
      python3 \
      python3-pip \
      python3-venv \
      unzip \
      xz-utils \
      zip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists

ENV PATH="/opt/unifmu-venv/bin:/root/.cargo/bin:/usr/osxcross/target/bin:${PATH}"
RUN python3 -m venv /opt/unifmu-venv \
    && pip install --no-cache-dir pyzmq protobuf==5.27.3 colorama coloredlogs toml fmpy

# install protoc
RUN case "${TARGETARCH}" in \
      amd64) protoc_arch=x86_64 ;; \
      arm64) protoc_arch=aarch_64 ;; \
      *) echo "Unsupported build host architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v27.3/protoc-27.3-linux-${protoc_arch}.zip" \
      -o /tmp/protoc.zip \
    && unzip -oq /tmp/protoc.zip -d /usr \
    && rm /tmp/protoc.zip

# get macos toolchain
WORKDIR /usr/
RUN git clone https://github.com/tpoechtrager/osxcross
RUN curl -LO https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz
RUN mv MacOSX11.3.sdk.tar.xz osxcross/tarballs/
WORKDIR /usr/osxcross/
RUN UNATTENDED=1 BUILD_FLAVOR=stable OSX_VERSION_MIN=11.0 ENABLE_ARCHS="arm64 x86_64" ./build.sh

# get rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu x86_64-apple-darwin \
      aarch64-apple-darwin aarch64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="/usr/osxcross/target/bin/x86_64-apple-darwin20.4-clang"
ENV CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="/usr/osxcross/target/bin/arm64-apple-darwin20.4-clang"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="x86_64-linux-gnu-gcc"
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"

WORKDIR /workdir