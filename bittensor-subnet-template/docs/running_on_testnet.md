# Running Subnet on Testnet

This tutorial shows how to use the Bittensor testnet to create a subnet and run your incentive mechanism on it. 

**IMPORTANT:** We strongly recommend that you first run [Running Subnet Locally](running_on_staging.md) before running on the testnet. Incentive mechanisms running on the testnet are open to anyone, and although these mechanisms on testnet do not emit real TAO, they cost you test TAO which you must create. 

**DANGER**
- Do not expose your private keys.
- Only use your testnet wallet.
- Do not reuse the password of your mainnet wallet.
- Make sure your incentive mechanism is resistant to abuse. 

## Prerequisites

Before proceeding further, make sure that you have installed Bittensor. See the below instructions:

- [Install `bittensor`](https://github.com/opentensor/bittensor#install).

After installing `bittensor`, proceed as below:

## 1. Install Bittensor subnet template

**NOTE: Skip this step if** you already did this during local testing and development.

`cd` into your project directory and clone the bittensor-subnet-template repo:

```bash
git clone https://github.com/opentensor/bittensor-subnet-template.git 
```

Next, `cd` into bittensor-subnet-template repo directory:

```bash
cd bittensor-subnet-template # Enter the 
```

Install the bittensor-subnet-template package:

```bash
python -m pip install -e . 
```

## 2. Create wallets 

Create wallets for subnet owner, subnet validator and for subnet miner.
  
This step creates local coldkey and hotkey pairs for your three identities: subnet owner, subnet validator and subnet miner. 

The owner will create and control the subnet. The owner must have at least 100 testnet TAO before the owner can run next steps. 

The validator and miner will be registered to the subnet created by the owner. This ensures that the validator and miner can run the respective validator and miner scripts.

Create a coldkey for your owner wallet:

```bash
btcli wallet new_coldkey --wallet.name owner
```

Create a coldkey and hotkey for your miner wallet:

```bash
btcli wallet new_coldkey --wallet.name miner
```

and

```bash
btcli wallet new_hotkey --wallet.name miner --wallet.hotkey default
```

Create a coldkey and hotkey for your validator wallet:

```bash
btcli wallet new_coldkey --wallet.name validator
```

and

```bash
btcli wallet new_hotkey --wallet.name validator --wallet.hotkey default
```

## 3. Get the price of subnet creation

Creating subnets on the testnet is competitive. The cost is determined by the rate at which new subnets are being registered onto the chain. 

By default you must have at least 100 testnet TAO in your owner wallet to create a subnet. However, the exact amount will fluctuate based on demand. The below command shows how to get the current price of creating a subnet.

```bash
btcli subnet lock_cost --subtensor.network test
```

The above command will show:

```bash
>> Subnet lock cost: τ100.000000000
```

## 4. (Optional) Get faucet tokens
   
Faucet is disabled on the testnet. Hence, if you don't have sufficient faucet tokens, ask the [Bittensor Discord community](https://discord.com/channels/799672011265015819/830068283314929684) for faucet tokens.

## 5. Purchase a slot

Using the test TAO from the previous step you can register your subnet on the testnet. This will create a new subnet on the testnet and give you the owner permissions to it. 

The below command shows how to purchase a slot. 

**NOTE**: Slots cost TAO to lock. You will get this TAO back when the subnet is deregistered.

```bash
btcli subnet create --subtensor.network test 
```

Enter the owner wallet name which gives permissions to the coldkey:

```bash
>> Enter wallet name (default): owner # Enter your owner wallet name
>> Enter password to unlock key: # Enter your wallet password.
>> Register subnet? [y/n]: <y/n> # Select yes (y)
>> ⠇ 📡 Registering subnet...
✅ Registered subnetwork with netuid: 1 # Your subnet netuid will show here, save this for later.
```

## 6. Register keys

This step registers your subnet validator and subnet miner keys to the subnet, giving them the **first two slots** on the subnet.

Register your miner key to the subnet:

```bash
btcli subnet recycle_register --netuid 13 --subtensor.network test --wallet.name miner --wallet.hotkey default
```

Follow the below prompts:

```bash
>> Enter netuid [1] (1): # Enter netuid 1 to specify the subnet you just created.
>> Continue Registration?
  hotkey:     ...
  coldkey:    ...
  network:    finney [y/n]: # Select yes (y)
>> ✅ Registered
```

Next, register your validator key to the subnet:

```bash
btcli subnet recycle_register --netuid 13 --subtensor.network test --wallet.name validator --wallet.hotkey default
```

Follow the prompts:

```bash
>> Enter netuid [1] (1): # Enter netuid 1 to specify the subnet you just created.
>> Continue Registration?
  hotkey:     ...
  coldkey:    ...
  network:    finney [y/n]: # Select yes (y)
>> ✅ Registered
```

## 7. Check that your keys have been registered

This step returns information about your registered keys.

Check that your validator key has been registered:

```bash
btcli wallet overview --wallet.name validator --subtensor.network test
```

The above command will display the below:

```bash
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  0      True   0.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ0.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0         
```

Check that your miner has been registered:

```bash
btcli wallet overview --wallet.name miner --subtensor.network test
```

The above command will display the below:

```bash
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  1      True   0.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ0.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0   
```

## 8. Run subnet miner and subnet validator

Run the subnet miner:

```bash
python neurons/miner.py --netuid 1 --subtensor.network test --wallet.name miner --wallet.hotkey default --logging.debug
```

You will see the below terminal output:

```bash
>> 2023-08-08 16:58:11.223 |       INFO       | Running miner for subnet: 1 on network: ws://127.0.0.1:9946 with config: ...
```

Next, run the subnet validator:

```bash
python neurons/validator.py --netuid 1 --subtensor.network test --wallet.name validator --wallet.hotkey default --logging.debug
```

You will see the below terminal output:

```bash
>> 2023-08-08 16:58:11.223 |       INFO       | Running validator for subnet: 1 on network: ws://127.0.0.1:9946 with config: ...
```


## 9. Get emissions flowing

Register to the root network using the `btcli`:

```bash
btcli root register --subtensor.network test
```

Then set your weights for the subnet:

```bash
btcli root weights --subtensor.network test
```

## 10. Stopping your nodes

To stop your nodes, press CTRL + C in the terminal where the nodes are running.
