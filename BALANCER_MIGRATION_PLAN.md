# V3 to Balancer Pool Migration Plan

## Overview

This document outlines the complete migration from Uniswap V3-style concentrated liquidity pools to Balancer-style weighted pools for the TAO/Alpha swap mechanism.

---

## Why Balancer Pools?

### Advantages over V3
1. **Simplicity**: No tick system, no bitmap management, no position tracking
2. **Flexible Liquidity**: Add/remove liquidity in any ratio (unbalanced)
3. **Single Pool State**: One global pool per subnet vs. many positions
4. **Simpler Math**: Weighted constant product vs. sqrt price curves
5. **Lower Gas**: Fewer storage operations and simpler calculations
6. **Easier Maintenance**: Less complex code, easier to audit

### Balancer Pool Mechanics

**Weighted Constant Product Formula**:
```
V = ∏ (balance_i ^ weight_i)
V remains constant during swaps
```

**Spot Price Formula**:
```
spot_price = (balance_out / weight_out) / (balance_in / weight_in)
```

**Swap Formula** (amount out given amount in):
```
amount_out = balance_out × (1 - (balance_in / (balance_in + amount_in)) ^ (weight_in / weight_out))
```

---

## Architecture Design

### New Storage Structure

```rust
// Per-subnet pool state
Pool<T> {
    tao_balance: TaoCurrency,        // TAO reserve
    alpha_balance: AlphaCurrency,     // Alpha reserve
    tao_weight: u64,                  // Weight for TAO (e.g., 50 = 50%)
    alpha_weight: u64,                // Weight for Alpha (e.g., 50 = 50%)
    total_shares: u128,               // Total LP shares
    swap_fee: u16,                    // Fee rate (normalized to u16::MAX)
}

// Per-account LP shares
LiquidityShares<T>: StorageDoubleMap<
    NetUid,
    AccountId,
    u128  // LP share amount
>

// Global protocol-owned shares (optional)
ProtocolShares<T>: StorageMap<NetUid, u128>
```

### Key Differences from V3

| Aspect | V3 (Uniswap) | Balancer |
|--------|--------------|----------|
| Liquidity | Concentrated in tick ranges | Uniform across all prices |
| Positions | Multiple positions per user | Single share balance per user |
| Storage | 14 storage items | 3-4 storage items |
| Price Discovery | Tick-based with sqrt pricing | Direct ratio with weights |
| Add Liquidity | Must be balanced to current price | Can be any ratio |
| Complexity | ~4,000 lines | ~800 lines (estimate) |

---

## Implementation Plan

### Phase 1: Core Balancer Pallet

**File Structure**:
```
pallets/balancer-swap/
├── Cargo.toml
├── src/
│   ├── lib.rs              // Main pallet definition
│   ├── math.rs             // Balancer math functions
│   ├── types.rs            // Pool and share types
│   ├── weights.rs          // Weight functions
│   ├── benchmarking.rs     // Benchmarks
│   └── tests.rs            // Unit tests
```

**Core Types**:
```rust
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Pool {
    pub tao_balance: TaoCurrency,
    pub alpha_balance: AlphaCurrency,
    pub tao_weight: Perbill,      // Using Perbill for weights
    pub alpha_weight: Perbill,
    pub total_shares: u128,
    pub swap_fee: Perbill,
}

impl Pool {
    // Calculate spot price: alpha price in TAO
    pub fn spot_price(&self) -> U96F32
    
    // Calculate swap amount out
    pub fn calc_out_given_in(&self, token_in: TokenType, amount_in: u64) -> u64
    
    // Calculate swap amount in (for exact out)
    pub fn calc_in_given_out(&self, token_out: TokenType, amount_out: u64) -> u64
    
    // Calculate shares to mint for liquidity addition
    pub fn calc_shares_for_liquidity(&self, tao: u64, alpha: u64) -> u128
    
    // Calculate token amounts for share redemption
    pub fn calc_liquidity_for_shares(&self, shares: u128) -> (u64, u64)
}

pub enum TokenType {
    Tao,
    Alpha,
}
```

### Phase 2: Extrinsics

**New Extrinsics** (simpler than V3):
```rust
// 1. Add liquidity (unbalanced allowed)
add_liquidity(
    origin,
    hotkey: AccountId,
    netuid: NetUid,
    tao_amount: TaoCurrency,
    alpha_amount: AlphaCurrency,
    min_shares: u128,  // Slippage protection
)

// 2. Remove liquidity
remove_liquidity(
    origin,
    hotkey: AccountId,
    netuid: NetUid,
    shares: u128,
    min_tao: TaoCurrency,      // Slippage protection
    min_alpha: AlphaCurrency,  // Slippage protection
)

// 3. Set pool parameters (admin only)
set_pool_weights(
    origin,
    netuid: NetUid,
    tao_weight: Perbill,
    alpha_weight: Perbill,
)

set_swap_fee(
    origin,
    netuid: NetUid,
    fee: Perbill,
)
```

**No Longer Needed** (from V3):
- ❌ `modify_position` - Not needed with shares
- ❌ `toggle_user_liquidity` - Always allowed with Balancer
- ❌ Position IDs and management

### Phase 3: Math Implementation

**Key Functions** (`math.rs`):
```rust
// Balancer weighted constant product math
pub fn calc_out_given_in(
    balance_in: u128,
    weight_in: Perbill,
    balance_out: u128,
    weight_out: Perbill,
    amount_in: u128,
) -> u128

pub fn calc_in_given_out(
    balance_in: u128,
    weight_in: Perbill,
    balance_out: u128,
    weight_out: Perbill,
    amount_out: u128,
) -> u128

pub fn calc_spot_price(
    balance_in: u128,
    weight_in: Perbill,
    balance_out: u128,
    weight_out: Perbill,
    swap_fee: Perbill,
) -> U96F32

// LP share calculations
pub fn calc_pool_out_given_single_in(
    balance_in: u128,
    weight_in: Perbill,
    pool_supply: u128,
    total_weight: Perbill,
    amount_in: u128,
    swap_fee: Perbill,
) -> u128

pub fn calc_single_out_given_pool_in(
    balance_out: u128,
    weight_out: Perbill,
    pool_supply: u128,
    total_weight: Perbill,
    shares_in: u128,
    swap_fee: Perbill,
) -> u128
```

### Phase 4: SwapHandler Trait Implementation

```rust
impl<T: Config> SwapHandler for Pallet<T> {
    fn swap<O: Order>(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError> {
        // 1. Get pool
        let pool = Pools::<T>::get(netuid)?;
        
        // 2. Calculate amount out using Balancer math
        let amount_out = pool.calc_out_given_in(...);
        
        // 3. Calculate fee
        let fee = if drop_fees { 0 } else { amount_out * pool.swap_fee };
        
        // 4. Update pool balances
        // 5. Update reserves
        // 6. Return result
    }
    
    fn sim_swap<O: Order>(...) -> Result<SwapResult<...>, DispatchError> {
        // Same as swap but in read-only mode
    }
    
    fn current_alpha_price(netuid: NetUid) -> U96F32 {
        Pools::<T>::get(netuid)
            .map(|pool| pool.spot_price())
            .unwrap_or_default()
    }
    
    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        // Add/remove protocol-owned liquidity
        // Much simpler than V3 version
    }
    
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        // Return all user shares as tokens
        // Simpler: iterate LiquidityShares, calculate amounts, transfer
    }
    
    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        // Remove protocol shares and reset pool
    }
    
    // Removed methods:
    // - is_user_liquidity_enabled (always true)
    // - toggle_user_liquidity (not needed)
}
```

---

## Migration Steps

### Step 1: Create Balancer Pallet ✓

1. Create `pallets/balancer-swap/` directory
2. Implement core types and math
3. Implement storage and extrinsics
4. Write basic tests

### Step 2: Update Swap Interface ✓

Modify `pallets/swap-interface/src/lib.rs`:
- Remove V3-specific methods
- Keep compatible method signatures
- Document Balancer behavior

### Step 3: Update Subtensor Integration ✓

Files to modify:
```
pallets/subtensor/src/coinbase/root.rs
  - Replace V3 dissolve/clear calls with Balancer equivalents

pallets/subtensor/src/coinbase/run_coinbase.rs
  - Update adjust_protocol_liquidity calls

pallets/subtensor/src/staking/claim_root.rs
  - Swap calls remain the same (trait compatibility)
```

### Step 4: Update Runtime ✓

`runtime/src/lib.rs`:
```rust
// Replace V3 config with Balancer config
impl pallet_balancer_swap::Config for Runtime {
    type SubnetInfo = SubtensorModule;
    type BalanceOps = SubtensorModule;
    type ProtocolId = SwapProtocolId;  // Reuse
    type TaoReserve = pallet_subtensor::TaoCurrencyReserve<Self>;
    type AlphaReserve = pallet_subtensor::AlphaCurrencyReserve<Self>;
    type DefaultTaoWeight = ConstU32<50>;  // 50% default
    type DefaultAlphaWeight = ConstU32<50>; // 50% default
    type DefaultSwapFee = DefaultSwapFee;
    type WeightInfo = pallet_balancer_swap::weights::DefaultWeight<Runtime>;
}

// In construct_runtime!
BalancerSwap: pallet_balancer_swap = 28,  // Replace Swap

// Update SwapInterface
type SwapInterface = BalancerSwap;
```

### Step 5: Update Precompiles ✓

`precompiles/src/alpha.rs`:
- No changes needed if SwapHandler trait stays compatible
- Methods use same trait calls

### Step 6: Migration Function ✓

Create one-time migration to convert V3 state to Balancer:
```rust
// In runtime/src/migrations/
pub fn migrate_v3_to_balancer<T: Config>() -> Weight {
    // For each subnet with V3:
    // 1. Calculate total liquidity from all positions
    // 2. Create Balancer pool with that liquidity
    // 3. Distribute shares proportionally to position owners
    // 4. Clear V3 storage
}
```

### Step 7: Remove V3 Code ✓

Delete:
```
pallets/swap/                          // Entire directory
pallets/swap/src/tick.rs              // 2,199 lines
pallets/swap/src/position.rs          // 199 lines
pallets/swap/src/pallet/swap_step.rs  // 563 lines
pallets/swap/src/pallet/tests.rs      // All V3 tests
```

Update:
- Remove V3 imports from all files
- Remove V3 tests from subtensor
- Remove V3-specific mock configurations

### Step 8: Testing ✓

New tests needed:
```
tests/balancer_swap_tests.rs
  - test_add_balanced_liquidity
  - test_add_unbalanced_liquidity
  - test_remove_liquidity
  - test_swap_tao_for_alpha
  - test_swap_alpha_for_tao
  - test_spot_price_calculation
  - test_weighted_pools_80_20
  - test_fees_accumulation
  - test_slippage_protection
  - test_protocol_liquidity_adjustment

tests/integration_tests.rs
  - test_subnet_lifecycle_with_balancer
  - test_emission_with_balancer_swaps
  - test_liquidity_provider_rewards
```

---

## Code Size Comparison

| Component | V3 | Balancer | Reduction |
|-----------|-----|----------|-----------|
| Core Logic | 1,144 lines | ~400 lines | 65% |
| Tick System | 2,199 lines | 0 lines | 100% |
| Position Mgmt | 199 lines | 0 lines | 100% |
| Swap Step | 563 lines | ~150 lines | 73% |
| Storage Items | 14 items | 3-4 items | 71% |
| **Total** | **~4,105 lines** | **~800 lines** | **80%** |

---

## Key Benefits Summary

1. ✅ **80% code reduction** - Less to maintain and audit
2. ✅ **Unbalanced liquidity** - More flexible for users
3. ✅ **Simpler state** - 3-4 storage items vs. 14
4. ✅ **No position management** - Just share balances
5. ✅ **Standard DeFi pattern** - Battle-tested Balancer math
6. ✅ **Lower gas costs** - Fewer storage operations
7. ✅ **Easier integration** - Cleaner trait interface
8. ✅ **Configurable weights** - Can adjust pool ratios

---

## Risks & Mitigations

### Risk 1: Loss of Concentrated Liquidity Benefits
- **Impact**: Lower capital efficiency than V3
- **Mitigation**: Balancer pools are still efficient; most trading happens near current price anyway

### Risk 2: Migration Complexity
- **Impact**: Converting V3 positions to Balancer shares
- **Mitigation**: Thorough testing; phased migration; comprehensive state snapshots

### Risk 3: Different Price Curves
- **Impact**: Prices may differ slightly from V3
- **Mitigation**: Document differences; run simulations comparing outputs

### Risk 4: User Confusion
- **Impact**: Users with V3 positions need to understand new system
- **Mitigation**: Clear documentation; migration guide; support period

---

## Timeline Estimate

- **Phase 1** (Balancer Pallet): 2-3 days
- **Phase 2** (Integration): 1-2 days  
- **Phase 3** (Migration Logic): 1 day
- **Phase 4** (Testing): 2-3 days
- **Phase 5** (V3 Removal): 1 day
- **Phase 6** (Documentation): 1 day

**Total**: 8-12 days of development

---

## Success Criteria

✅ All swaps work correctly with Balancer math
✅ Liquidity add/remove functions properly
✅ Prices match expected Balancer formulas
✅ All existing integration points work
✅ Tests pass with >95% coverage
✅ No V3 code remains in codebase
✅ Gas costs reduced vs. V3
✅ Documentation complete

---

## Next Steps

1. ✅ Get approval for Balancer approach
2. → Create balancer-swap pallet skeleton
3. → Implement core Balancer math
4. → Implement storage and extrinsics
5. → Update SwapHandler implementation
6. → Update integration points
7. → Write migration function
8. → Comprehensive testing
9. → Remove V3 code
10. → Deploy and monitor

---

End of Migration Plan



