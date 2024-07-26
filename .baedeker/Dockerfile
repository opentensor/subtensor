FROM ubuntu:20.04

WORKDIR /subtensor

COPY target/release/node-subtensor ./target/release/node-subtensor
COPY scripts ./scripts
COPY .baedeker ./.baedeker

# Expose the port the node will run on
EXPOSE 9946

# Run the Substrate node binary
CMD ["./scripts/localnet-baedeker.sh"]
