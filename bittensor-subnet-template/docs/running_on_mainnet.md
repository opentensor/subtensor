# Running Subnet on Mainnet

This tutorial shows how to use the bittensor `btcli` to create a subnetwork and connect your incentive mechanism to it. 

**IMPORTANT:** Before attempting to register on mainnet, we strongly recommend that you:
- First run [Running Subnet Locally](running_on_staging.md), and
- Then run [Running on the Testnet](running_on_testnet.md).

Your incentive mechanisms running on the mainnet are open to anyone. They emit real TAO. Creating these mechanisms incur a `lock_cost` in TAO.

**DANGER**
- Do not expose your private keys.
- Only use your testnet wallet.
- Do not reuse the password of your mainnet wallet.
- Make sure your incentive mechanism is resistant to abuse. 

## Prerequisites

Before proceeding further, make sure that you have installed Bittensor. See the below instructions:

- [Install `bittensor`](https://github.com/opentensor/bittensor#install).

After installing `bittensor`, proceed as below:

## Steps

## 1. Install your subnet template

**NOTE: Skip this step if** you already did this during local testing and development.

In your project directory:

```bash
git clone https://github.com/opentensor/bittensor-subnet-template.git 
```

Next, `cd` into `bittensor-subnet-template` repo directory:

```bash
cd bittensor-subnet-template
```

Install the Bittensor subnet template package:

```bash
python -m pip install -e . # Install your subnet template package
```

## 2. Create wallets 

Create wallets for subnet owner, subnet validator and for subnet miner.
  
This step creates local coldkey and hotkey pairs for your three identities: subnet owner, subnet validator and subnet miner. 

The owner will create and control the subnet. The owner must have at least 100  TAO before the owner can run next steps. 

The validator and miner will be registered to the subnet created by the owner. This ensures that the validator and miner can run the respective validator and miner scripts.

**NOTE**: You can also use existing wallets to register. Creating new keys is shown here for reference.

Create a coldkey for the owner wallet:

```bash
btcli wallet new_coldkey --wallet.name owner
```

Create a coldkey and hotkey for the subnet miner wallet:
```bash
btcli wallet new_coldkey --wallet.name miner
```

and

```bash
btcli wallet new_hotkey --wallet.name miner --wallet.hotkey default
```

Create a coldkey and hotkey for the subnet validator wallet:

```bash
btcli wallet new_coldkey --wallet.name validator
```

and

```bash
btcli wallet new_hotkey --wallet.name validator --wallet.hotkey default
```

## 3. Getting the price of subnet creation

Creating subnets on mainnet is competitive. The cost is determined by the rate at which new subnets are being registered onto the Bittensor blockchain. 

By default you must have at least 100 TAO on your owner wallet to create a subnet. However, the exact amount will fluctuate based on demand. The below code shows how to get the current price of creating a subnet.

```bash
btcli subnet lock_cost 
```

The above command will show:

```bash
>> Subnet lock cost: τ100.000000000
```

## 4. Purchasing a slot

Using your TAO balance, you can register your subnet to the mainchain. This will create a new subnet on the mainchain and give you the owner permissions to it. The below command shows how to purchase a slot. 

**NOTE**: Slots cost TAO to lock. You will get this TAO back when the subnet is deregistered.

```bash
btcli subnet create  
```

Enter the owner wallet name. This gives permissions to the coldkey.

```bash
>> Enter wallet name (default): owner # Enter your owner wallet name
>> Enter password to unlock key: # Enter your wallet password.
>> Register subnet? [y/n]: <y/n> # Select yes (y)
>> ⠇ 📡 Registering subnet...
✅ Registered subnetwork with netuid: 1 # Your subnet netuid will show here, save this for later.
```

## 5. (Optional) Register keys 

**NOTE**: While this is not enforced, we recommend subnet owners to run a subnet validator and a subnet miner on the subnet to demonstrate proper use to the community.

This step registers your subnet validator and subnet miner keys to the subnet giving them the **first two slots** on the subnet.

Register your miner key to the subnet:

```bash
btcli subnet recycle_register --netuid 1 --subtensor.network finney --wallet.name miner --wallet.hotkey default
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
btcli subnet recycle_register --netuid 1 --subtensor.network finney --wallet.name validator --wallet.hotkey default
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

## 6. Check that your keys have been registered

Check that your subnet validator key has been registered:

```bash
btcli wallet overview --wallet.name validator 
```

The output will be similar to the below:

```bash
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  0      True   0.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ0.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0         
```

Check that your subnet miner has been registered:

```bash
btcli wallet overview --wallet.name miner 
```

The output will be similar to the below:

```bash
Subnet: 1                                                                                                                                                                
COLDKEY  HOTKEY   UID  ACTIVE  STAKE(τ)     RANK    TRUST  CONSENSUS  INCENTIVE  DIVIDENDS  EMISSION(ρ)   VTRUST  VPERMIT  UPDATED  AXON  HOTKEY_SS58                    
miner    default  1      True   0.00000  0.00000  0.00000    0.00000    0.00000    0.00000            0  0.00000                14  none  5GTFrsEQfvTsh3WjiEVFeKzFTc2xcf…
1        1        2            τ0.00000  0.00000  0.00000    0.00000    0.00000    0.00000           ρ0  0.00000                                                         
                                                                          Wallet balance: τ0.0   
```

## 7. Run subnet miner and subnet validator

Run the subnet miner:

```bash
python neurons/miner.py --netuid 1  --wallet.name miner --wallet.hotkey default --logging.debug
```

You will see the below terminal output:

```bash
>> 2023-08-08 16:58:11.223 |       INFO       | Running miner for subnet: 1 on network: wss://entrypoint-finney.opentensor.ai:443 with config: ...
```

Run the subnet validator:

```bash
python neurons/validator.py --netuid 1  --wallet.name validator --wallet.hotkey default --logging.debug
```

You will see the below terminal output:

```bash
>> 2023-08-08 16:58:11.223 |       INFO       | Running validator for subnet: 1 on network: wss://entrypoint-finney.opentensor.ai:443 with config: ...
```

## 8. Get emissions flowing

Register to the root subnet using the `btcli`:

```bash
btcli root register 
```

Then set your weights for the subnet:

```bash
btcli root weights 
```

## 9. Stopping your nodes

To stop your nodes, press CTRL + C in the terminal where the nodes are running.

---