# Rate-Limiting Migration Scripts

These scripts validate the migration from legacy sparse rate-limiting to `pallet-rate-limiting` on a cloned Finney state.

They cover three things:
- preparing a patched mainnet clone spec and clone state
- applying a real `sudo(set_code)` runtime upgrade on the clone
- validating migrated grouped-call config, post-upgrade behavior, and migrated storage

Current scope:
- grouped calls only
- standalone-call migration is not covered here and is planned for a separate PR

## Preparation

Before using these scripts, make sure:
- the node binary is built:
  - `target/release/node-subtensor`
- the runtime wasm is built:
  - `target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm`
- `ts-tests` dependencies are installed

Typical build commands:

```bash
cargo build -p node-subtensor --release --features=metadata-hash
cd ts-tests && pnpm install
```

## Artifacts

By default everything is stored under:
- `target/rate-limits-test`

Main paths:
- patched spec: `target/rate-limits-test/patched-finney.json`
- cloned source state: `target/rate-limits-test/source`
- node db: `target/rate-limits-test/run/alice`

The default artifact root is:
- `/Users/alestsurko/Desktop/subtensor/target/rate-limits-test`

## Flow

1. Prepare the clone artifacts once:

```bash
scripts/rate-limiting-migration/prepare-rate-limits-clone.sh
```

2. Start the clone node in terminal 1:

```bash
scripts/rate-limiting-migration/start-rate-limits-clone-node.sh
```

3. Run the runtime upgrade in terminal 2:

```bash
scripts/rate-limiting-migration/upgrade-rate-limits-clone.sh
```

4. Stop the old node after the upgrade script finishes.

5. Start the upgraded node again in terminal 1:

```bash
scripts/rate-limiting-migration/start-rate-limits-clone-node.sh
```

6. Run validations in terminal 2:

```bash
scripts/rate-limiting-migration/validate-rate-limits-clone-config.sh
scripts/rate-limiting-migration/validate-rate-limits-clone-behavior.sh
scripts/rate-limiting-migration/validate-rate-limits-clone-storage.sh
```

## What each script does

- `prepare-rate-limits-clone.sh`
  - builds the patched clone spec
  - prepares synced clone state used for local replay
- `start-rate-limits-clone-node.sh`
  - runs the local clone node from the prepared spec and db path
- `upgrade-rate-limits-clone.sh`
  - submits the runtime upgrade to the running clone node
- `validate-rate-limits-clone-config.sh`
  - checks migrated grouped-call runtime API/config responses
- `validate-rate-limits-clone-behavior.sh`
  - runs real post-upgrade extrinsic probes
- `validate-rate-limits-clone-storage.sh`
  - audits migrated grouped-call rate-limiting storage directly

## Behavior phase filter

You can rerun only one behavior phase:

```bash
scripts/rate-limiting-migration/validate-rate-limits-clone-behavior.sh weights
```

Supported filters:
- `serving`
- `staking`
- `delegate-take`
- `weights`
- `swap-keys`
- `owner-hparams`

## Useful environment variables

- `BINARY_PATH`
- `RUNTIME_WASM`
- `CLONE_BASE_DIR`
- `PATCHED_CHAIN_SPEC`
- `CLONE_CHAIN_SPEC`
- `CLONE_SOURCE_BASE_PATH`
- `CLONE_RUN_BASE_PATH`
- `CLONE_NODE_PORT`
- `CLONE_NODE_RPC_PORT`
- `CLONE_PREP_RPC_PORT`
- `CLONE_PREP_P2P_PORT`
- `CLONE_SYNC_MODE`
- `CLONE_SYNC_TIMEOUT_SEC`
- `SKIP_CLONE_PREPARE=1`
- `SKIP_NODE_RESET=1`

## Notes

- `validate-rate-limits-clone-config.sh` should finish quickly. It only reads runtime API/config.
- `validate-rate-limits-clone-behavior.sh` is slower. It submits real extrinsics and waits across rate-limit windows.
