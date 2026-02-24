# E2E Tests

End-to-end tests that run against a local multi-node subtensor network.

## Quick start

```bash
cd e2e

# 1. Set up the development environment (nvm, node, pnpm, jq, yq).
./setup_env.sh

# 2. Build the node binary and generate polkadot-api type descriptors.
#    Does not require pnpm deps — uses `pnpm dlx` for the papi CLI.
#    Re-run this step whenever runtime metadata changes (new pallets,
#    modified storage/calls, etc.) to keep descriptors in sync.
./bootstrap_types.sh

# 3. Install dependencies (requires descriptors from step 2).
pnpm install

# 4. Run a test suite.
pnpm --filter e2e-shield test      # run the shield suite
pnpm --filter e2e-<name> test      # run any suite by name
pnpm -r test                       # run all suites
```

## Creating a new test package

```bash
./bootstrap_package.sh <name>
pnpm install
pnpm --filter e2e-<name> test
```

This creates a package with:
- `package.json` — depends on `e2e-shared` and `polkadot-api`
- `vitest.config.ts` — sequential execution, 120s timeout, alphabetical sequencer
- `setup.ts` — global setup/teardown that spawns a 2-node network
- `tests/00-basic.test.ts` — sample test

Edit `setup.ts` to configure the number of nodes, extra authorities, and
ports for your suite. Add test-specific dependencies to `package.json`.

## How it works

### Network lifecycle

Each test suite manages its own local network via vitest's `globalSetup`:

1. **setup()** generates a chain spec, optionally patches it with extra
   authorities, spawns validator nodes, waits for peering and finalization,
   then writes `NetworkState` to a JSON file under `/tmp/subtensor-e2e/`.
2. **Test files** read the state file in `beforeAll()` to get RPC ports and
   connect via polkadot-api. Tests run sequentially (alphabetical by filename),
   so later files can build on earlier state changes (e.g. scaling the network).
3. **teardown()** stops all nodes (including extras added mid-suite), cleans
   up temp directories and the state file.

### Shared utilities (`e2e-shared`)

The `shared/` package provides reusable helpers for all test suites:
spawning and monitoring substrate nodes, generating and patching chain specs,
connecting polkadot-api clients with dev signers, and a custom vitest
sequencer that ensures test files run in alphabetical order.

### Conventions

- **File prefixes** — Name test files `00-`, `01-`, `02-` etc. The custom
  sequencer sorts alphabetically, so numbering controls execution order.
- **State file** — Each suite writes to `/tmp/subtensor-e2e/<name>/`. Tests
  can update this file mid-suite (e.g. to register extra nodes).
- **Catalog versions** — To add a new dependency, first pin its version in
  `pnpm-workspace.yaml` under `catalog:`, then reference it in your
  package's `package.json` with `"catalog:"` as the version. This prevents
  version drift across packages.
- **Query at "best"** — Storage queries for values that change every block
  (e.g. rotating keys) should use `{ at: "best" }` instead of the default
  `"finalized"`, since finalized lags ~2 blocks behind with GRANDPA.
- **Built-in shortcuts** — Substrate dev accounts (`one`, `two`, `alice`,
  `bob`, etc.) have their keys auto-injected. Custom authorities need
  `insertKeys()` before starting the node.
