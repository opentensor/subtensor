# Running Subnet Locally

This tutorial will guide you through:

- Setting up a local blockchain that is not connected to either Bittensor testchain or mainchain
- Creating a subnet
- Run your incentive mechanism on the subnet.

## Local blockchain vs local subtensor node 

Running a local blockchain is sometimes synonymously referred as running on staging. This is **different** from running a local subtensor node that connects to the Bittensor mainchain. 

A local subtensor node will connect to the mainchain and sync with the mainchain, giving you your own access point to the mainchain. 

Running a local blockchain spins up two authority nodes locally, not connected to any other nodes or testchain or mainchain. This tutorial is for running a local blockchain. 

## Prerequisites

Before proceeding further, make sure that you have installed Bittensor. See the below instructions:

- [Install `bittensor`](https://github.com/opentensor/bittensor#install).

After installing `bittensor`, proceed as below:

## 1. Install Substrate dependencies

Begin by installing the required dependencies for running a Substrate node.

Update your system packages:

```bash
sudo apt update 
```

Install additional required libraries and tools

```bash
sudo apt install --assume-yes make build-essential git clang curl libssl-dev llvm libudev-dev protobuf-compiler
```

## 2. Install Rust and Cargo

Rust is the programming language used in Substrate development. Cargo is Rust package manager.

Install rust and cargo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Update your shell's source to include Cargo's path:

```bash
source "$HOME/.cargo/env"
```

## 3. Clone the subtensor repository

This step fetches the subtensor codebase to your local machine.

```bash
git clone https://github.com/opentensor/subtensor.git
```

## 4. Setup Rust

This step ensures that you have the nightly toolchain and the WebAssembly (wasm) compilation target. Note that this step will run the subtensor chain on your terminal directly, hence we advise that you run this as a background process using PM2 or other software.

Update to the nightly version of Rust:

```bash
./subtensor/scripts/init.sh
```

## 5. Initialize 

These steps initialize your local subtensor chain in development mode. These commands will set up and run a local subtensor.

Build the binary with the faucet feature enabled:

```bash
cargo build --release --features pow-faucet
```

**NOTE**: The `--features pow-faucet` option in the above is required if we want to use the command `btcli wallet faucet` [See the below Mint tokens step](#8-mint-tokens-from-faucet).

Next, run the localnet script and turn off the attempt to build the binary (as we have already done this above):

```bash
BUILD_BINARY=0 ./scripts/localnet.sh 
```

**NOTE**: Watch for any build or initialization outputs in this step. If you are building the project for the first time, this step will take a while to finish building, depending on your hardware.

## 6. Install subnet template

`cd` to your project directory and clone the bittensor subnet template repository:

```bash
git clone https://github.com/opentensor/bittensor-subnet-template.git
```

Navigate to the cloned repository:

```bash
cd bittensor-subnet-template
```

Install the bittensor-subnet-template Python package:

```bash
python -m pip install -e .
```

## 7. Set up wallets

You will need wallets for the different roles, i.e., subnet owner, subnet validator and subnet miner, in the subnet. 

- The owner wallet creates and controls the subnet. 
- The validator and miner will be registered to the subnet created by the owner. This ensures that the validator and miner can run the respective validator and miner scripts.

Create a coldkey for the owner role:

```bash
btcli wallet new_coldkey --wallet.name owner
```

Set up the miner's wallets:

```bash
btcli wallet new_coldkey --wallet.name miner
```

```bash
btcli wallet new_hotkey --wallet.name miner --wallet.hotkey default
```

Set up the validator's wallets:

```bash
btcli wallet new_coldkey --wallet.name validator
```
```bash
btcli wallet new_hotkey --wallet.name validator --wallet.hotkey default
```

## 8. Mint tokens from faucet

You will need tokens to initialize the intentive mechanism on the chain as well as for registering the subnet. 

Run the following commands to mint faucet tokens for the owner and for the validator.

Mint faucet tokens for the owner:

```bash
btcli wallet faucet --wallet.name owner --subtensor.chain_endpoint ws://127.0.0.1:9946 
```

You will see:

```bash
>> Balance: τ0.000000000 ➡ τ100.000000000
```

Mint tokens for the validator:

```bash
btcli wallet faucet --wallet.name validator --subtensor.chain_endpoint ws://127.0.0.1:9946 
```

You will see:

```bash
>> Balance: τ0.000000000 ➡ τ100.000000000
```

## 9. Create a subnet

The below commands establish a new subnet on the local chain. The cost will be exactly τ1000.000000000 for the first subnet you create and you'll have to run the faucet several times to get enough tokens.

```bash
btcli subnet create --wallet.name owner --subtensor.chain_endpoint ws://127.0.0.1:9946 
```

You will see:

```bash
>> Your balance is: τ200.000000000
>> Do you want to register a subnet for τ1000.000000000? [y/n]: 
>> Enter password to unlock key: [YOUR_PASSWORD]
>> ✅ Registered subnetwork with netuid: 1
```

**NOTE**: The local chain will now have a default `netuid` of 1. The second registration will create a `netuid` 2 and so on, until you reach the subnet limit of 8. If you register more than 8 subnets, then a subnet with the least staked TAO will be replaced by the 9th subnet you register.

## 10. Register keys

Register your subnet validator and subnet miner on the subnet. This gives your two keys unique slots on the subnet. The subnet has a current limit of 128 slots.

Register the subnet miner:

```bash
btcli subnet register --wallet.name miner --wallet.hotkey default --subtensor.chain_endpoint ws://127.0.0.1:9946
```

Follow the below prompts:

```bash
>> Enter netuid [1] (1): 1
>> Continue Registration? [y/n]: y
>> ✅ Registered
```

Register the subnet validator:

```bash

btcli subnet register --wallet.name validator --wallet.hotkey default --subtensor.chain_endpoint ws://127.0.0.1:9946
```

Follow the below prompts:

```
>> Enter netuid [1] (1): 1
>> Continue Registration? [y/n]: y
>> ✅ Registered
```

## 11. Add stake 

This step bootstraps the incentives on your new subnet by adding stake into its incentive mechanism.

```bash
btcli stake add --wallet.name validator --wallet.hotkey default --subtensor.chain_endpoint ws://127.0.0.1:9946
```

Follow the below prompts:

```bash
>> Stake all Tao from account: 'validator'? [y/n]: y
>> Stake:
    τ0.000000000 ➡ τ100.000000000
```

## 12. Validate key registrations

Verify that both the miner and validator keys are successfully registered:

```bash
btcli subnet list --subtensor.chain_endpoint ws://127.0.0.1:9946
```

You will see the `2` entry under `NEURONS` column for the `NETUID` of 1, indicating that you have registered a validator and a miner in this subnet:

```bash
NETUID  NEURONS  MAX_N   DIFFICULTY  TEMPO  CON_REQ  EMISSION  BURN(τ)  
   1        2     256.00   10.00 M    1000    None     0.00%    τ1.00000 
   2      128    
```

See the subnet validator's registered details:

```bash
btcli wallet overview --wallet.name validator --subtensor.chain_endpoint ws://127.0.0.1:9946
```

You will see:

```
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  0      True   100.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ100.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0         
```

See the subnet miner's registered details:

```bash
btcli wallet overview --wallet.name miner --subtensor.chain_endpoint ws://127.0.0.1:9946
```

You will see:

```bash
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  1      True   0.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ0.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0   

```

## 13. Run subnet miner and subnet validator

Run the subnet miner and subnet validator. Make sure to specify your subnet parameters.

Run the subnet miner:

```bash
python neurons/miner.py --netuid 1 --subtensor.chain_endpoint ws://127.0.0.1:9946 --wallet.name miner --wallet.hotkey default --logging.debug
```

Run the subnet validator:

```bash
python neurons/validator.py --netuid 1 --subtensor.chain_endpoint ws://127.0.0.1:9946 --wallet.name validator --wallet.hotkey default --logging.debug
```

## 14. Set weights for your subnet

Register a validator on the root subnet and boost to set weights for your subnet. This is a necessary step to ensure that the subnet is able to receive emmissions.

### Register your validator on the root subnet

```bash
btcli root register --wallet.name validator --wallet.hotkey default --subtensor.chain_endpoint ws://127.0.0.1:9946
```

### Boost your subnet on the root subnet
```bash
btcli root boost --netuid 1 --increase 1 --wallet.name validator --wallet.hotkey default --subtensor.chain_endpoint ws://127.0.0.1:9946
```

## 15. Verify your incentive mechanism

After a few blocks the subnet validator will set weights. This indicates that the incentive mechanism is active. Then after a subnet tempo elapses (360 blocks or 72 minutes) you will see your incentive mechanism beginning to distribute TAO to the subnet miner.

```bash
btcli wallet overview --wallet.name miner --subtensor.chain_endpoint ws://127.0.0.1:9946
```

## Ending your session

To halt your nodes:
```bash
# Press CTRL + C keys in the terminal.
```

---
