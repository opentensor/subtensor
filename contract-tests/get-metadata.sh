SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
cd "$SCRIPT_DIR"

rm -rf .papi
npx papi add devnet -w ws://localhost:9944
npx papi ink add ./bittensor/target/ink/bittensor.json
# Yarn copies file: dependencies into node_modules, so reinstall to pick up new .papi/descriptors.
yarn install
