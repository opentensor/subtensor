# Benchmarks and Weights

## Overview

Every extrinsic in the runtime has a **weight** — a measure of the computational
resources it consumes. Weights are used to calculate transaction fees and to
prevent blocks from being overloaded.

Weights are defined in `weights.rs` files inside each pallet and are generated
by running benchmarks on reference hardware.

## Tools

| Tool | Purpose |
|------|---------|
| `scripts/benchmark_all.sh` | Generate `weights.rs` for one or all pallets (runs real benchmarks) |
| `scripts/benchmark_stub.sh` | Create/update placeholder `weights.rs` from a benchmarking file (no benchmarks needed) |
| `weight-compare` | Compare two `weights.rs` files and report drift (used by CI) |

All weight tools live in `support/weight-tools/` and have no heavy dependencies
(no runtime build required).

## Adding a new pallet

1. Write your benchmarks in `pallets/<name>/src/benchmarking.rs` using
   `#[benchmarks]` and `#[benchmark]` macros.

2. Generate the placeholder `weights.rs`:

   ```sh
   ./scripts/benchmark_stub.sh pallet_<name>
   ```

3. Add `pub mod weights;` to your pallet's `lib.rs`.

4. Add `type WeightInfo: crate::weights::WeightInfo;` to your pallet's
   `Config` trait.

5. Use `T::WeightInfo::extrinsic_name()` in `#[pallet::weight(...)]`
   annotations instead of hardcoded `Weight::from_parts(...)`.

6. Wire up in `runtime/src/lib.rs`:
   ```rust
   type WeightInfo = pallet_<name>::weights::SubstrateWeight<Runtime>;
   ```

7. Add `type WeightInfo = ();` to all test mocks implementing your pallet's
   `Config`.

8. Register the pallet in the `define_benchmarks!` macro in
   `runtime/src/lib.rs` so the benchmark runner can discover it:
   ```rust
   define_benchmarks!(
       // ...existing pallets...
       [pallet_<name>, PalletInstance]
   );
   ```

9. Register the pallet in `scripts/benchmark_all.sh` by adding an entry to
   `PALLET_OUTPUTS`.

CI will generate real weights automatically when the PR is opened.

## Adding a new extrinsic to an existing pallet

1. Write the benchmark in `benchmarking.rs`.

2. Update `weights.rs` with placeholder values:

   ```sh
   ./scripts/benchmark_stub.sh pallet_<name>
   ```

   This merges with the existing file: new benchmarks get placeholder values,
   existing ones keep their real values, removed benchmarks are cleaned up.

3. Add `#[pallet::weight(T::WeightInfo::new_extrinsic())]` to your new
   extrinsic.

CI will generate real weights automatically when the PR is opened.

## Parameterized weights

For extrinsics whose cost scales with an input, use `Linear<min, max>`
parameters in the benchmark. You can use one or more parameters:

```rust
// Single parameter
#[benchmark]
fn refund(k: Linear<1, 100>) {
    // setup with k contributors...
    #[extrinsic_call]
    _(origin, crowdloan_id);
}

// Multiple parameters
#[benchmark]
fn transfer_batch(n: Linear<1, 256>, m: Linear<1, 64>) {
    // setup with n recipients and m tokens each...
    #[extrinsic_call]
    _(origin, recipients, amounts);
}
```

This generates weight functions with matching signatures:

```rust
fn refund(k: u32) -> Weight;
fn transfer_batch(n: u32, m: u32) -> Weight;
```

The generated weight includes base values plus per-parameter slope terms
(e.g., `base + slope_k * k` for single parameter, or
`base + slope_n * n + slope_m * m` for multiple). Reference them as:

```rust
#[pallet::weight(T::WeightInfo::refund(T::MaxContributors::get()))]
#[pallet::weight(T::WeightInfo::transfer_batch(recipients.len() as u32, max_tokens))]
```

## CI workflow

The `Validate-Benchmarks` workflow (`.github/workflows/run-benchmarks.yml`)
runs on every PR:

1. Builds the node with `--features runtime-benchmarks`
2. Runs benchmarks for every pallet, generating new `weights.rs` to temp files
3. Uses `weight-compare` to compare old vs new values with a **40% threshold**
   - Base weight: threshold-based (allows measurement noise)
   - Reads/writes: exact match (these are deterministic)
   - Component slopes: threshold-based for weights, exact for reads/writes
4. If drift is detected, prepares a patch in `.bench_patch/`
5. Adding the `apply-benchmark-patch` label auto-applies the patch

To skip benchmarks on a PR, add the `skip-validate-benchmarks` label. This can
be added at any point during the job — it's checked between expensive steps.

## Running benchmarks locally

```sh
# Build with benchmarks enabled
cargo build --profile production -p node-subtensor --features runtime-benchmarks

# Generate weights for all pallets
./scripts/benchmark_all.sh

# Generate weights for a single pallet
./scripts/benchmark_all.sh pallet_subtensor

# Skip the build step (if already built)
SKIP_BUILD=1 ./scripts/benchmark_all.sh pallet_drand

# Compare two weight files
cargo run -p subtensor-weight-tools --bin weight-compare -- \
  --old pallets/foo/src/weights.rs \
  --new /tmp/new_weights.rs \
  --threshold 40
```

## Weight file structure

Generated `weights.rs` files contain:

- `WeightInfo` trait — one function per benchmarked extrinsic
- `SubstrateWeight<T>` impl — used in the runtime, references `T::DbWeight`
- `()` impl — used in tests, references `RocksDbWeight`

The `()` fallback uses `RocksDbWeight` constants even though the node defaults
to ParityDb. This is intentional — RocksDb is slower, so it serves as a
conservative upper bound. The production runtime uses `T::DbWeight::get()`
which is configured in `runtime/src/lib.rs` via `type DbWeight`.

The files are generated by the `frame-benchmarking-cli` using the Handlebars
template at `.maintain/frame-weight-template.hbs`.
