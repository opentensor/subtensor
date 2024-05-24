# Running subtensor node locally

- [Method 1: Using Docker](#method-1-using-docker)
- [Method 2: Using Source Code](#method-2-using-source-code)
- [Running on Cloud](#running-on-cloud)

## Method 1: Using Docker

To run a subtensor node with Docker, follow the below steps.

If you are already running a subtensor node using Docker, then go directly to [Step 5 Prepare to Run](#step-5-prepare-to-run) to restart the Docker container. The below steps 1 through 4 are for first time users only.

### Step 1: Install git

Make sure you installed `git` on your machine. See [GitHub docs](https://docs.github.com/en/get-started).

### Step 2: Install Docker

Follow Docker's [official installation guides](https://docs.docker.com/engine/install/) and install Docker.

**Run Docker first**
Before you proceed, make sure that Docker is running.

### Step 3: Clone the subtensor repo

Clone the subtensor repo:

```bash
git clone https://github.com/opentensor/subtensor.git
```

### Step 4: Go into subtensor directory

Then `cd` into the subtensor directory:

```bash
cd subtensor
```

### Step 5: Prepare to run

Execute the below three commands in this order:

Make sure you are on the `main` branch. If not, switch to it:

```bash
git checkout main
```

Pull the latest `main` branch contents:

```bash
git pull
```

Stop the currently running Docker containers:

```bash
docker compose down --volumes
```

### Run a lite node on mainchain

To run a lite node connected to the Bittensor mainchain, run the below command.

```bash
sudo ./scripts/run/subtensor.sh -e docker --network mainnet --node-type lite
```

### Run an archive node on mainchain

To run an archive node connected to the Bittensor mainchain, run the below command.

```bash
sudo ./scripts/run/subtensor.sh -e docker --network mainnet --node-type archive
```

### Run a lite node on testchain

To run a lite node connected to the Bittensor testchain, run the below command.

```bash
sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type lite
```

### Run an archive node on testchain

To run an archive node connected to the Bittensor testchain, run the below command.

```bash
sudo ./scripts/run/subtensor.sh -e docker --network testnet --node-type archive
```

---

## Method 2: Using Source Code

To install and run a subtensor node by compiling the source code, follow the below steps.

## Install basic packages

Install the basic requirements by running the below commands on a Linux terminal.

```bash title="On Linux"
sudo apt-get update
sudo apt install build-essential
sudo apt-get install clang
sudo apt-get install curl
sudo apt-get install git
sudo apt-get install make
sudo apt install --assume-yes git clang curl libssl-dev protobuf-compiler
sudo apt install --assume-yes git clang curl libssl-dev llvm libudev-dev make protobuf-compiler
```

## Install Rust

Next, install Rust and update the environment by running the following commands:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Next, install Rust toolchain:

```bash
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup target add --toolchain nightly wasm32-unknown-unknown
```

## Compile subtensor code

Next, to compile the subtensor source code, follow the below steps:

Clone the subtensor repo:

```bash
git clone https://github.com/opentensor/subtensor.git
```

`cd` into the subtensor directory:

```bash
cd subtensor
```

Make sure you are on the `main` branch. If not, switch to it:

```bash
git checkout main
```

Remove previous chain state:

```bash
rm -rf /tmp/blockchain
```

Install subtensor by compiling with `cargo`:

```bash
cargo build --release --features=runtime-benchmarks
```

## Run the subtensor node

You can now run the public subtensor node either as a lite node or as an archive node. See below:

### Lite node on mainchain

To run a lite node connected to the mainchain, execute the below command (note the `--sync=warp` flag which runs the subtensor node in lite mode):

```bash title="With --sync=warp setting, for lite node"
./target/release/node-subtensor --chain raw_spec.json --base-path /tmp/blockchain --sync=warp --execution wasm --wasm-execution compiled --port 30333 --max-runtime-instances 32 --rpc-max-response-size 2048 --rpc-cors all --rpc-port 9944 --bootnodes /ip4/13.58.175.193/tcp/30333/p2p/12D3KooWDe7g2JbNETiKypcKT1KsCEZJbTzEHCn8hpd4PHZ6pdz5 --no-mdns --in-peers 8000 --out-peers 8000 --prometheus-external --rpc-external 
```

### Archive node on mainchain

To run an archive node connected to the mainchain, execute the below command (note the `--sync=full` which syncs the node to the full chain and `--pruning archive` flags, which disables the node's automatic pruning of older historical data):

```bash title="With --sync=full and --pruning archive setting, for archive node"
./target/release/node-subtensor --chain raw_spec.json --base-path /tmp/blockchain --sync=full --pruning archive --execution wasm --wasm-execution compiled --port 30333 --max-runtime-instances 32 --rpc-max-response-size 2048 --rpc-cors all --rpc-port 9944 --bootnodes /ip4/13.58.175.193/tcp/30333/p2p/12D3KooWDe7g2JbNETiKypcKT1KsCEZJbTzEHCn8hpd4PHZ6pdz5 --no-mdns --in-peers 8000 --out-peers 8000 --prometheus-external --rpc-external 
```

### Lite node on testchain

To run a lite node connected to the testchain, execute the below command:

```bash title="With bootnodes set to testnet and --sync=warp setting, for lite node."
./target/release/node-subtensor --chain raw_testspec.json --base-path /tmp/blockchain --sync=warp --execution wasm --wasm-execution compiled --port 30333 --max-runtime-instances 32 --rpc-max-response-size 2048 --rpc-cors all --rpc-port  9944 --bootnodes /dns/bootnode.test.finney.opentensor.ai/tcp/30333/p2p/12D3KooWPM4mLcKJGtyVtkggqdG84zWrd7Rij6PGQDoijh1X86Vr --no-mdns --in-peers 8000 --out-peers 8000 --prometheus-external --rpc-external 
```

### Archive node on testchain

To run an archive node connected to the testchain, execute the below command:

```bash title="With bootnodes set to testnet and --sync=full and --pruning archive setting, for archive node"
./target/release/node-subtensor --chain raw_testspec.json --base-path /tmp/blockchain --sync=full --pruning archive --execution wasm --wasm-execution compiled --port 30333 --max-runtime-instances 32 --rpc-max-response-size 2048 --rpc-cors all --rpc-port 9944 --bootnodes /dns/bootnode.test.finney.opentensor.ai/tcp/30333/p2p/12D3KooWPM4mLcKJGtyVtkggqdG84zWrd7Rij6PGQDoijh1X86Vr --no-mdns --in-peers 8000 --out-peers 8000 --prometheus-external --rpc-external 
```

## Running on cloud
We have not tested these installation scripts on any cloud service. In addition, if you are using Runpod cloud service, then note that this service is already [containerized](https://docs.runpod.io/pods/overview). Hence, the only option available to you is to compile from the source, as described in the above [Method 2: Using Source Code](#method-2-using-source-code) section. Note that these scripts have not been tested on Runpod.
