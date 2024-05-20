Sure, here are the updated instructions using `just` commands to allow users to choose between downloading the CI artifact and building the artifacts locally using Docker.

### Instructions for Running Baedeker Locally on a Mac

We use Baedeker to run a clone of the Finney network. Unfortunately, Baedeker does not work on MacOS. We therefore leverage the CI to produce the build artifacts and download them to run locally, or alternatively, build the artifacts locally using Docker.

### Prerequisites
1. **Install GitHub CLI**: [Install GitHub CLI](https://cli.github.com/).
2. **Install Docker**: [Install Docker](https://docs.docker.com/get-docker/).
3. **Install Just**: [Install Just](https://github.com/casey/just#installation).

### Option 1: Download the Latest Artifacts from CI

#### Step 1: Understand the CI Job
The CI job in [.github/workflows/baedeker.yaml](file:///Users/samueldare/code/samtvlabs/bittensor/subtensor/.github/workflows/baedeker.yaml#1%2C1-1%2C1) creates the build artifacts for the mainnet:
- Checks out the repository.
- Installs dependencies and Rust.
- Clones and builds the Baedeker repository.
- Creates the chain spec and secrets.
- Uploads the chain spec and secrets as artifacts named `baedeker-config`.

#### Step 2: Download the Latest Artifacts
Use the `just download-baedeker-config` command to download the latest artifacts.

```bash
just baedeker-download
```

#### Step 3: Run the Local Blockchain

use `just baedeker-run` to run the local blockchain with the mainnet state.

```bash
just baedeker-run
```

### Option 2: Build the Artifacts Locally using Docker

#### Step 1: Run the Build Command
Use the `just build-baedeker` command to build the artifacts locally using Docker.

```bash
just baedeker-build
```

#### Step 2: Run the Local Blockchain

use `just baedeker-run` to run the local blockchain with the mainnet state.

```bash
just baedeker-run
```