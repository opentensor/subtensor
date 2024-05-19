## Instructions for Running Baedeker Locally on a Mac

We use Baedeker to run the a clone of the finney network. Unfortunately , Baedeker does not work on MacOS. We therefore leverage the CI to produce the build artifacts and download them to run locally.

### Prerequisites
1. **Install GitHub CLI**: [Install GitHub CLI](https://cli.github.com/).

### Step 1: Understand the CI Job
The CI job in `.github/workflows/baedeker.yaml` creates the build artifacts for the mainnet:
- Checks out the repository.
- Installs dependencies and Rust.
- Clones and builds the Baedeker repository.
- Creates the chain spec and secrets.
- Uploads the chain spec and secrets as artifacts named `baedeker-config`.

### Step 2: Download the Latest Artifacts
Use the `scripts/download_baedeker_config.sh` script to download the latest artifacts.

```bash
./scripts/download_baedeker_config.sh
```

### Step 3: Run the Local Blockchain
Use the `scripts/localnet-baedeker.sh` script to run the local blockchain with the mainnet state.

```bash
    ./scripts/localnet-baedeker.sh
```
