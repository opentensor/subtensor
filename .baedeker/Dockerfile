FROM ubuntu:20.04
WORKDIR /substrate

# show backtraces
ENV RUST_BACKTRACE 1

# Necessary libraries for Rust execution
RUN apt-get update && \
  apt-get install -y curl build-essential protobuf-compiler clang git && \
  rm -rf /var/lib/apt/lists/*

# Install cargo and Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Capture dependencies
COPY . .

# Cargo build
RUN cargo build --release && cp -R ./target /target
EXPOSE 9946

# Run the Substrate node binary
CMD ["./scripts/localnet-baedeker.sh"]
