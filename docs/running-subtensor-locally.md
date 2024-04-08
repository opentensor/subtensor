# Running subtensor locally

- [Running docker](#running-docker)
- [Compiling your own binary](#compiling-your-own-binary)
- [Running on cloud](#running-on-cloud)

## Running docker

### Install git
`sudo apt install git`

### Install Docker Engine
 You can follow Docker's [oficial installation guides](https://docs.docker.com/engine/install/)

### Run node-subtensor container
```bash
git clone https://github.com/opentensor/subtensor.git
cd subtensor
# to run a lite node on the mainnet:
sudo ./scripts/run/subtensor.sh -e docker --network mainnet --node-type lite
# or mainnet archive node: sudo ./scripts/run/subtensor.sh -e docker --network mainnet --node-type archive
# or testnet lite node:    sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type lite
# or testnet archive node: sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type archive
```

## Compiling your own binary
### Requirements
```bash
sudo apt install build-essential git make clang libssl-dev llvm libudev-dev protobuf-compiler -y
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup-init.sh
chmod +x rustup-init.sh
./rustup-init.sh # you can select default options in the prompts you will be given
source "$HOME/.cargo/env"
```

### Rustup update
```bash
rustup default stable && \
 rustup update && \
 rustup update nightly && \
 rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Compiling
```bash
git clone https://github.com/opentensor/subtensor.git
cd subtensor
cargo build --release --features runtime-benchmarks
```

### Running the node
#### Mainnet / Lite node
```bash
sudo ./scripts/run/subtensor.sh -e binary --network mainnet --node-type lite
``` 

#### Mainnet / Archive node
```bash
sudo ./scripts/run/subtensor.sh -e docker --network mainnet --node-type archive
```

#### Testnet / Lite node
```bash
sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type lite
```

#### Testnet / Archive node
```bash
sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type archive
```

## Running on cloud
We have not tested these installation scripts on any cloud service. In addition, if you are using Runpod cloud service, then note that this service is already [containerized](https://docs.runpod.io/pods/overview). Hence, the only option available to you is to compile from the source, as described in the above [Compiling your own binary](#compiling-your-own-binary) section. Note that these scripts have not been tested on Runpod.
