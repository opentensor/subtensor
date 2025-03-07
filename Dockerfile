ARG BASE_IMAGE=ubuntu:latest

FROM $BASE_IMAGE AS builder
SHELL ["/bin/bash", "-c"]

# Set noninteractive mode for apt-get
ARG DEBIAN_FRONTEND=noninteractive

LABEL ai.opentensor.image.authors="operations@opentensor.ai" \
  ai.opentensor.image.vendor="Opentensor Foundation" \
  ai.opentensor.image.title="opentensor/subtensor" \
  ai.opentensor.image.description="Opentensor Subtensor Blockchain" \
  ai.opentensor.image.documentation="https://docs.bittensor.com"

# Set up Rust environment
ENV RUST_BACKTRACE=1
RUN apt-get update && \
  apt-get install -y curl build-essential protobuf-compiler clang git pkg-config libssl-dev && \
  rm -rf /var/lib/apt/lists/*

# Copy entire repository
COPY . /build
WORKDIR /build

# Install Rust
RUN set -o pipefail && curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install
RUN rustup target add wasm32-unknown-unknown

# Build the project
RUN cargo build -p node-subtensor --profile production  --features="metadata-hash" --locked

# Slim down image
RUN rm -rf /root/.cargo

# Verify the binary was produced
RUN test -e /build/target/production/node-subtensor

EXPOSE 30333 9933 9944

FROM $BASE_IMAGE AS subtensor

# Copy all chainspec files
COPY --from=builder /build/chainspecs/*.json /

# Copy final binary
COPY --from=builder /build/target/production/node-subtensor /usr/local/bin
