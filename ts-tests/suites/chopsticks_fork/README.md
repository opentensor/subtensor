# Chopsticks mainnet-fork tests

These tests fork **live finney** with [Chopsticks](https://github.com/AcalaNetwork/chopsticks),
apply your locally-built runtime, and run extrinsics (currently an `addStake` and a
balance transfer) against
real mainnet state for a single subnet.

Vanilla Chopsticks forking of finney is unusably slow (~15 min) because the chain's
storage maps are enormous and Chopsticks lazily live-fetches them during block
production. `scripts/gen-chopsticks-fork.ts` works around this: it connects to a
finney archive node, fetches storage **for one netuid only**, and bakes it into a
slim, self-contained Chopsticks config that forks in ~1 min.

## Prerequisites

1. **Build the runtime wasm** (non-fast / release profile). Building the node also
   produces the runtime wasm as a byproduct:

   ```bash
   cargo build --release -p node-subtensor
   ```

   The fork uses `target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm`
   via `--wasm-override` (see `moonwall.config.json` → env `chopsticks_fork`).

2. **Network access** to a finney archive endpoint (configured in
   `configs/chopsticks-fork.yml`). This is why the suite is **not** part of the CI
   matrix — run it locally / on demand.

## Run

```bash
cd ts-tests
pnpm install

# Regenerates the slim config from the live chain, then runs the suite:
pnpm moonwall test chopsticks_fork
```

Moonwall runs the config generator as a **pre-test step** (the env's `runScripts`)
before the fork is launched, so the config is rebuilt fresh on every run and every
test in the suite shares it:

1. **Pre-test (moonwall `runScripts`):** `gen-chopsticks-fork.ts configs/chopsticks-fork.yml tmp/chopsticks-fork-slim.yml`
   - Connects to the live chain and writes `tmp/chopsticks-fork-slim.yml` (slim config,
     fork `block:` pinned) and `tmp/chopsticks-fork-slim.meta.json`
     (`{ blockNumber, netuid, hotkey }`).
2. **Launch + test (moonwall):** starts Chopsticks from the slim config with
   `--wasm-override`, then runs the tests in this directory.

### Targeting a specific subnet

By default the generator picks the first non-root registered netuid. The selection
flags also have env equivalents, because moonwall's `runScripts` entry is static and
can't take CLI args at `moonwall test` time:

| Flag (manual generator run) | Env (moonwall pre-test path) |
|---|---|
| `--netuid <n>` | `FORK_NETUID=<n>` |
| `--random-subnet` | `FORK_RANDOM_SUBNET=1` |

```bash
# Drive the moonwall pre-test step via env in one shot:
FORK_NETUID=5 pnpm moonwall test chopsticks_fork

# Or generate manually first (e.g. to inspect the config), then run:
pnpm tsx scripts/gen-chopsticks-fork.ts configs/chopsticks-fork.yml tmp/chopsticks-fork-slim.yml --netuid 5
pnpm moonwall test chopsticks_fork
```

The test reads the chosen netuid + an existing registered hotkey from
`tmp/chopsticks-fork-slim.meta.json`, so it always matches the generated config.

## How the runtime upgrade is applied

The fork boots **directly on your locally-built runtime** via Chopsticks
`--wasm-override` (`launchSpec.wasmOverride`). This replaces `:code` in the forked
state with no on-chain `spec_version` check, and `Executive::on_runtime_upgrade`
runs any bundled migrations against real mainnet state on the first produced block.
This mirrors the proven approach used in the `relayer` repo.

> **Alternative — real on-chain `setCode`:** moonwall's chopsticks context exposes
> `context.upgradeRuntime()`, which performs a true `system.applyAuthorizedUpgrade`.
> That path enforces `spec_version` **strictly greater** than mainnet's, so it only
> works when your branch has bumped `spec_version`. To use it, add
> `"rtUpgradePath": "../target/release/wbuild/.../node_subtensor_runtime.compact.compressed.wasm"`
> to the `chopsticks_fork` foundation, drop `wasmOverride`, and call
> `await context.upgradeRuntime()` in the test's `beforeAll`.

## Adding tests

Add more `*.ts` files to this directory using `foundationMethods: "chopsticks"`.
Useful context helpers: `context.polkadotJs()`, `context.createBlock()`,
`context.setStorage(...)`, `context.keyring.alice`.
