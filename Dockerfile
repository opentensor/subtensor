ARG BASE_IMAGE=rust:1.83

FROM $BASE_IMAGE AS builder

LABEL ai.opentensor.image.authors="operations@opentensor.ai" \
  ai.opentensor.image.vendor="Opentensor Foundation" \
  ai.opentensor.image.title="opentensor/subtensor" \
  ai.opentensor.image.description="Opentensor Subtensor Blockchain" \
  ai.opentensor.image.documentation="https://docs.bittensor.com"

RUN rustup update stable
RUN rustup target add wasm32-unknown-unknown --toolchain stable


# Set up Rust environment
ENV RUST_BACKTRACE=1
RUN apt-get update && apt-get install -y protobuf-compiler curl clang git
RUN rm -rf /var/lib/apt/lists/*

# Copy entire repository
COPY . /build
WORKDIR /build

# Build the project
RUN cargo build -p node-subtensor --profile production  --features="metadata-hash" --locked

# Verify the binary was produced
RUN test -e /build/target/production/node-subtensor

EXPOSE 30333 9933 9944

FROM $BASE_IMAGE AS subtensor

# Copy all chainspec files
COPY --from=builder /build/*.json /

# Copy final binary
COPY --from=builder /build/target/production/node-subtensor /usr/local/bin


FROM subtensor AS subtensor-local


RUN "node-subtensor" build-spec --disable-default-bootnode --raw --chain local > /localnet.json
