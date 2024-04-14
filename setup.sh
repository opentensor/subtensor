bittensor/bin/btcli wallet faucet --wallet.name owner --subtensor.chain_endpoint ws://104.171.201.172:9946
bittensor/bin/btcli wallet faucet --wallet.name validator --subtensor.chain_endpoint ws://104.171.201.172:9946
bittensor/bin/btcli wallet faucet --wallet.name miner --subtensor.chain_endpoint ws://104.171.201.172:9946
bittensor/bin/btcli subnet create --wallet.name owner --subtensor.chain_endpoint ws://127.0.0.1:9946 
bittensor/bin/btcli s register --wallet.name validator --netuid 1 

