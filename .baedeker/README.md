# What can baedeker do

Baedeker is a tool for running live substrate based chains in miscellaneous configurations.

## Testing runtime upgrades with local net

### Create chain spec that contains live chain data

In order to generate a chain spec (on a Linux machine):

1. Have rust installed and configured to build the subtensor project 
2. Install baedeker:

```bash
git clone https://github.com/UniqueNetwork/baedeker.git && cd baedeker
cargo build –release
sudo cp ./target/release/baedeker /usr/local/bin
sudo chmod +x /usr/local/bin/baedeker
```

3. Run baedeker from project root to generate the chain spec from finney state:

```bash
sudo .baedeker/up-local.sh .baedeker/forkless-data.jsonnet --tla-str=forked_spec=subtensor --tla-str=fork_source=wss://entrypoint-finney.opentensor.ai
```

It will output the chain spec file to 

```
.baedeker/.bdk-env/specs/subtensor.json
```

and also it will create secrets for misnamed alice, bob, and charlie and write them to this folder:

```
.baedeker/.bdk-env/secrets
```

### Testing process

In this process we test runtime upgrade from a chain running runtime build off commit A to commit B.

1. Run baedeker to build the spec with current data (see above)
   The baedeker generated files are .gitignored, so they will not be affected by further `git checkout` commands
2. Checkout the commit A
3. Build the binary for commit A
4. Launch the local network with node binary, spec, and keys:

```bash
./scripts/localnet-baedeker.sh
```
Note: For launching the older versions of network that use Polkadot pre-1.0 version, add  --ws-port 9947 to localnet-baedeker.sh

5. Stop the local network

6. Edit localnet-baedeker.sh so that it doesn't delete existing chain data when started again

7. Checkout the commit B that we want to upgrade the network to

8. Build the binary

9. Restart local network using edited localnet-baedeker.sh

10. Do runtime upgrade

