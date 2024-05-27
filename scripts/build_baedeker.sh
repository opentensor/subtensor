#!/bin/bash

# Create a Dockerfile
cat <<EOF > Dockerfile.baedeker
FROM ubuntu:20.04

# Install dependencies
RUN apt-get update && \
    apt-get install -y clang curl libssl-dev llvm libudev-dev protobuf-compiler git

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:\$PATH"

# Clone Baedeker repository
RUN git clone https://github.com/UniqueNetwork/baedeker.git /baedeker
WORKDIR /baedeker

# Copy .baedeker directory
COPY .baedeker .baedeker

# Install Baedeker
RUN echo "[workspace]" >> Cargo.toml
RUN cargo install --path . --locked

# Create chain spec and secrets
RUN ./.baedeker/up.sh .baedeker/forkless-data.jsonnet --tla-str=forked_spec=subtensor --tla-str=fork_source=wss://entrypoint-finney.opentensor.ai

# Create the /baedeker-config directory
RUN mkdir -p /baedeker-config

# Copy the generated files to the /baedeker-config directory
RUN cp -r .baedeker/.bdk-env/specs /baedeker-config/
RUN cp -r .baedeker/.bdk-env/secret /baedeker-config/
EOF

# Build the Docker image
docker build -t baedeker-builder -f Dockerfile.baedeker .

# Run the Docker container to generate the files
container_id=$(docker run -d baedeker-builder)

# Copy the files from the Docker container to the local directory
docker cp $container_id:/baedeker-config/ $(pwd)/

# Stop and remove the container
docker rm -f $container_id

# Clean up
rm Dockerfile.baedeker