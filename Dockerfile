# This is an example build stage for the node subtensor. Here we create the binary in a temporary image.

# This is a base image to build substrate nodes
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /node-subtensor
COPY . .
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the binary."
FROM docker.io/library/ubuntu:20.04
LABEL description="Multistage Docker image for Substrate Node Subtensor" \
  image.type="builder" \
  image.authors="you@email.com" \
  image.vendor="Substrate Developer Hub" \
  image.description="Multistage Docker image for Substrate Node Subtensor" \
  image.source="https://github.com/substrate-developer-hub/substrate-node-subtensor" \
  image.documentation="https://github.com/substrate-developer-hub/substrate-node-subtensor"

# Copy the node binary.
COPY --from=builder /node-subtensor/target/release/node-subtensor /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /node-dev node-dev && \
  mkdir -p /chain-data /node-dev/.local/share && \
  chown -R node-dev:node-dev /chain-data && \
  ln -s /chain-data /node-dev/.local/share/node-subtensor && \
  # unclutter and minimize the attack surface
  rm -rf /usr/bin /usr/sbin && \
  # check if executable works in this container
  /usr/local/bin/node-subtensor --version

USER node-dev

EXPOSE 30333 9933 9944 9615
VOLUME ["/chain-data"]

ENTRYPOINT ["/usr/local/bin/node-subtensor"]
