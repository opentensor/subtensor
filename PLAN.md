# Implementation Plan: Emission Scaling and Subnet Limiting Hyperparameters

## Overview

Three new root-level hyperparameters for `get_subnet_block_emissions()` in `src/coinbase/subnet_emissions.rs`:

1. **EffectiveRootPropEmissionScaling** (bool) - When enabled, multiply each subnet's emission share by its `EffectiveRootProp`
2. **EmissionTopSubnetProportion** (u16, default 50% = 5000/10000) - Only top-K% of subnets by share receive emission
3. **EmissionTopSubnetAbsoluteLimit** (u16, default 0 = disabled) - Hard cap on number of subnets receiving emission

## Subfeature 1: EffectiveRootProp Storage & Computation

### 1A. Storage declarations in `pallets/subtensor/src/lib.rs`

Add near the existing flow-related storage items (~line 1489):

```rust
/// --- MAP ( netuid ) --> EffectiveRootProp for a subnet.
/// Computed during epoch as:
///   sum(RootAlphaDividendsPerSubnet[netuid]) /
///   (sum(AlphaDividendsPerSubnet[netuid]) + sum(RootAlphaDividendsPerSubnet[netuid]))
/// This measures the proportion of dividends on a subnet that go to root stakers.
#[pallet::storage]
pub type EffectiveRootProp<T: Config> =
    StorageMap<_, Identity, NetUid, U64F64, ValueQuery>;
```

### 1B. Compute and store EffectiveRootProp during epoch

In `pallets/subtensor/src/coinbase/run_coinbase.rs`, inside `distribute_dividends_and_incentives()` (after the alpha dividends and root alpha dividends loops at ~line 638), add computation:

```rust
// After distributing both alpha_divs and root_alpha_divs, compute EffectiveRootProp
let total_root_alpha_divs: U64F64 = /* sum of root_alpha values from root_alpha_dividends map */;
let total_alpha_divs: U64F64 = /* sum of alpha_divs values from alpha_dividends map */;
let total = total_root_alpha_divs + total_alpha_divs;
let effective_root_prop = if total > 0 { total_root_alpha_divs / total } else { 0 };
EffectiveRootProp::<T>::insert(netuid, effective_root_prop);
```

Create a helper function `compute_and_store_effective_root_prop()` that takes netuid and the two dividend maps, computes the ratio, stores it, and returns the value. This keeps `distribute_dividends_and_incentives` clean.

### 1C. Tests for EffectiveRootProp computation

In `pallets/subtensor/src/tests/subnet_emissions.rs`:
- Test that `EffectiveRootProp` is 0.0 when there are no root alpha dividends
- Test that `EffectiveRootProp` is 1.0 when there are no alpha dividends (all root)
- Test that `EffectiveRootProp` is ~0.5 when root and alpha dividends are equal
- Test that correct values are stored per-subnet after `distribute_dividends_and_incentives` runs

## Subfeature 2: EffectiveRootPropEmissionScaling Hyperparameter

### 2A. Storage declaration in `pallets/subtensor/src/lib.rs`

Add near the flow-related storage items:

```rust
#[pallet::type_value]
pub fn DefaultEffectiveRootPropEmissionScaling<T: Config>() -> bool {
    false
}
#[pallet::storage]
/// When enabled, multiply each subnet's emission share by its EffectiveRootProp
pub type EffectiveRootPropEmissionScaling<T: Config> =
    StorageValue<_, bool, ValueQuery, DefaultEffectiveRootPropEmissionScaling<T>>;
```

### 2B. Setter in `pallets/subtensor/src/utils/misc.rs`

Add setter function following the pattern of `set_tao_flow_cutoff`:
```rust
pub fn set_effective_root_prop_emission_scaling(enabled: bool) {
    EffectiveRootPropEmissionScaling::<T>::set(enabled);
}
```

### 2C. Admin extrinsic in `pallets/admin-utils/src/lib.rs`

Add `sudo_set_effective_root_prop_emission_scaling` with next available call_index (88):
```rust
#[pallet::call_index(88)]
pub fn sudo_set_effective_root_prop_emission_scaling(
    origin: OriginFor<T>,
    enabled: bool,
) -> DispatchResult {
    ensure_root(origin)?;
    pallet_subtensor::Pallet::<T>::set_effective_root_prop_emission_scaling(enabled);
    Ok(())
}
```

### 2D. Apply scaling in `get_subnet_block_emissions()`

In `pallets/subtensor/src/coinbase/subnet_emissions.rs`, modify `get_subnet_block_emissions()`:

After `get_shares()` returns shares, add a step:
1. If `EffectiveRootPropEmissionScaling` is enabled, multiply each share by `EffectiveRootProp::<T>::get(netuid)`
2. Re-normalize shares so they sum to 1.0

Extract this into a helper function `apply_effective_root_prop_scaling(shares: &mut BTreeMap<NetUid, U64F64>)`.

### 2E. Event for scaling application

Add an event in `pallets/subtensor/src/macros/events.rs`:
```rust
/// Emission shares have been adjusted by EffectiveRootProp scaling.
EffectiveRootPropEmissionScalingApplied {
    /// Per-subnet shares after scaling (netuid, share)
    shares: Vec<(NetUid, u64)>,
},
```

Emit this event after applying the scaling so tests can verify it.

### 2F. Tests

- Test that with scaling disabled, shares are unchanged
- Test that with scaling enabled and known EffectiveRootProp values, shares are correctly multiplied and re-normalized
- Test that event is emitted when scaling is applied
- Test edge case: all EffectiveRootProp values are 0 (should result in equal shares or all zeros)
- Test edge case: single subnet

## Subfeature 3: EmissionTopSubnetProportion

### 3A. Storage declaration in `pallets/subtensor/src/lib.rs`

```rust
#[pallet::type_value]
pub fn DefaultEmissionTopSubnetProportion<T: Config>() -> u16 {
    5000  // 50% = 5000/10000
}
#[pallet::storage]
/// Proportion of subnets (by count, ranked by share) that receive emission.
/// Value is in basis points: 5000 = 50%. Subnets outside top-K% get shares zeroed.
/// Round up: ceil(count * proportion / 10000).
pub type EmissionTopSubnetProportion<T: Config> =
    StorageValue<_, u16, ValueQuery, DefaultEmissionTopSubnetProportion<T>>;
```

### 3B. Setter in `pallets/subtensor/src/utils/misc.rs`

```rust
pub fn set_emission_top_subnet_proportion(proportion: u16) {
    EmissionTopSubnetProportion::<T>::set(proportion);
}
```

### 3C. Admin extrinsic in `pallets/admin-utils/src/lib.rs`

Add `sudo_set_emission_top_subnet_proportion` with call_index 89:
```rust
#[pallet::call_index(89)]
pub fn sudo_set_emission_top_subnet_proportion(
    origin: OriginFor<T>,
    proportion: u16,
) -> DispatchResult {
    ensure_root(origin)?;
    ensure!(proportion <= 10000, Error::<T>::InvalidValue);
    ensure!(proportion > 0, Error::<T>::InvalidValue);
    pallet_subtensor::Pallet::<T>::set_emission_top_subnet_proportion(proportion);
    Ok(())
}
```

### 3D. Apply in `get_subnet_block_emissions()`

After getting shares (and after EffectiveRootProp scaling if enabled):

1. Sort subnets by share descending
2. Compute K = ceil(total_subnets * proportion / 10000)
3. Zero out shares for subnets not in top-K
4. Re-normalize remaining shares to sum to 1.0

Extract into helper function `apply_top_subnet_proportion_filter(shares: &mut BTreeMap<NetUid, U64F64>)`.

### 3E. Event

```rust
/// Subnet emission shares zeroed for subnets outside top proportion.
EmissionTopSubnetFilterApplied {
    /// Number of subnets that kept emission
    top_k: u16,
    /// Total number of subnets considered
    total: u16,
},
```

### 3F. Tests

- Test default 50%: 4 subnets -> top 2 get emission (ceil(4*0.5)=2)
- Test default 50%: 1 subnet -> still gets emission (ceil(1*0.5)=1)
- Test default 50%: 3 subnets -> top 2 get emission (ceil(3*0.5)=2)
- Test 100%: all subnets get emission
- Test that shares are re-normalized after filtering
- Test that event is emitted with correct top_k and total
- Test that zeroed subnets get zero emission in final output

## Subfeature 4: EmissionTopSubnetAbsoluteLimit

### 4A. Storage declaration in `pallets/subtensor/src/lib.rs`

```rust
#[pallet::type_value]
pub fn DefaultEmissionTopSubnetAbsoluteLimit<T: Config>() -> u16 {
    0  // 0 means no limit
}
#[pallet::storage]
/// Absolute maximum number of subnets that can receive emission.
/// 0 means no limit (disabled). When set, only the top N subnets by share receive emission.
pub type EmissionTopSubnetAbsoluteLimit<T: Config> =
    StorageValue<_, u16, ValueQuery, DefaultEmissionTopSubnetAbsoluteLimit<T>>;
```

### 4B. Setter in `pallets/subtensor/src/utils/misc.rs`

```rust
pub fn set_emission_top_subnet_absolute_limit(limit: u16) {
    EmissionTopSubnetAbsoluteLimit::<T>::set(limit);
}
```

### 4C. Admin extrinsic in `pallets/admin-utils/src/lib.rs`

Add `sudo_set_emission_top_subnet_absolute_limit` with call_index 90:
```rust
#[pallet::call_index(90)]
pub fn sudo_set_emission_top_subnet_absolute_limit(
    origin: OriginFor<T>,
    limit: u16,
) -> DispatchResult {
    ensure_root(origin)?;
    pallet_subtensor::Pallet::<T>::set_emission_top_subnet_absolute_limit(limit);
    Ok(())
}
```

### 4D. Apply in `get_subnet_block_emissions()`

After proportion filter, apply absolute limit:

1. If limit > 0 and limit < number of remaining nonzero subnets:
   - Sort by share descending
   - Zero out shares beyond position `limit`
   - Re-normalize

Extract into helper `apply_top_subnet_absolute_limit(shares: &mut BTreeMap<NetUid, U64F64>)`.

### 4E. Event

```rust
/// Subnet emission shares zeroed for subnets beyond absolute limit.
EmissionAbsoluteLimitApplied {
    /// The absolute limit applied
    limit: u16,
    /// Number of subnets that had nonzero shares before limiting
    before_count: u16,
},
```

### 4F. Tests

- Test with limit=0: no filtering occurs
- Test with limit=2 and 5 subnets: only top 2 get emission
- Test with limit=10 and 3 subnets: all 3 get emission (limit > count)
- Test interaction with proportion filter: both applied, stricter one wins
- Test event emission

## Execution Order in `get_subnet_block_emissions()`

The final `get_subnet_block_emissions()` function should:

```
1. let shares = get_shares(subnets)
2. apply_effective_root_prop_scaling(&mut shares)  // if enabled
3. apply_top_subnet_proportion_filter(&mut shares)
4. apply_top_subnet_absolute_limit(&mut shares)
5. convert shares to emissions
```

Each step is a separate function that is easy to test independently.

## Shared helper: normalize_shares()

Create a utility `normalize_shares(shares: &mut BTreeMap<NetUid, U64F64>)` that normalizes values to sum to 1.0. Used by multiple scaling/filtering steps.

## Shared helper: zero_and_redistribute_bottom_shares()

Create `zero_and_redistribute_bottom_shares(shares: &mut BTreeMap<NetUid, U64F64>, top_k: usize)` that:
1. Sorts entries by value descending
2. Zeros out entries beyond top_k
3. Calls `normalize_shares`

This is reused by both the proportion filter and the absolute limit.

## Implementation Order (4 commits)

### Commit 1: EffectiveRootProp storage + computation
- Add `EffectiveRootProp` storage map to `lib.rs`
- Add `compute_and_store_effective_root_prop()` helper in `run_coinbase.rs`
- Call it from `distribute_dividends_and_incentives()`
- Add tests for EffectiveRootProp computation
- Run `scripts/fix_rust.sh`

### Commit 2: EffectiveRootPropEmissionScaling hyperparameter
- Add `EffectiveRootPropEmissionScaling` storage to `lib.rs`
- Add setter in `utils/misc.rs`
- Add admin extrinsic in `admin-utils/src/lib.rs`
- Add `normalize_shares()` and `apply_effective_root_prop_scaling()` to `subnet_emissions.rs`
- Add event + emit it
- Wire into `get_subnet_block_emissions()`
- Add tests
- Run `scripts/fix_rust.sh`

### Commit 3: EmissionTopSubnetProportion hyperparameter
- Add `EmissionTopSubnetProportion` storage to `lib.rs`
- Add setter in `utils/misc.rs`
- Add admin extrinsic in `admin-utils/src/lib.rs`
- Add `zero_and_redistribute_bottom_shares()` and `apply_top_subnet_proportion_filter()` to `subnet_emissions.rs`
- Add event + emit it
- Wire into `get_subnet_block_emissions()`
- Add tests
- Run `scripts/fix_rust.sh`

### Commit 4: EmissionTopSubnetAbsoluteLimit hyperparameter
- Add `EmissionTopSubnetAbsoluteLimit` storage to `lib.rs`
- Add setter in `utils/misc.rs`
- Add admin extrinsic in `admin-utils/src/lib.rs`
- Add `apply_top_subnet_absolute_limit()` to `subnet_emissions.rs`
- Add event + emit it
- Wire into `get_subnet_block_emissions()`
- Add tests (including interaction with proportion filter)
- Run `scripts/fix_rust.sh`

## Files Modified

| File | Changes |
|------|---------|
| `pallets/subtensor/src/lib.rs` | 4 new storage items + 3 type_value defaults |
| `pallets/subtensor/src/coinbase/run_coinbase.rs` | `compute_and_store_effective_root_prop()` helper + call site |
| `pallets/subtensor/src/coinbase/subnet_emissions.rs` | `normalize_shares()`, `apply_effective_root_prop_scaling()`, `zero_and_redistribute_bottom_shares()`, `apply_top_subnet_proportion_filter()`, `apply_top_subnet_absolute_limit()`, modified `get_subnet_block_emissions()` |
| `pallets/subtensor/src/utils/misc.rs` | 3 setter functions |
| `pallets/admin-utils/src/lib.rs` | 3 new extrinsics (call_index 88, 89, 90) |
| `pallets/subtensor/src/macros/events.rs` | 3 new events |
| `pallets/subtensor/src/tests/subnet_emissions.rs` | ~15 new tests |
