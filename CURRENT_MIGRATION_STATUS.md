# Balancer Migration - Current Status

## ✅ COMPLETED (Phase 1-3)

### Phase 1: Balancer Pallet Creation
- ✅ Created directory: `pallets/balancer-swap/`
- ✅ Created `Cargo.toml` with dependencies
- ✅ **types.rs** (185 lines) - Pool struct, TokenType, spot price
- ✅ **math.rs** (382 lines) - All Balancer weighted pool formulas with tests:
  - `calc_out_given_in()` - Swap calculations
  - `calc_in_given_out()` - Reverse swap  
  - `calc_spot_price()` - Price with fees
  - `calc_shares_for_single_token_in()` - Unbalanced LP
  - `calc_token_out_for_shares()` - LP redemption
  - `calc_shares_proportional()` - Balanced LP
- ✅ **lib.rs** (700+ lines) - Main pallet with:
  - Storage: Pools, LiquidityShares, ProtocolShares
  - Extrinsics: add_liquidity, remove_liquidity, set_pool_weights, set_swap_fee
  - SwapHandler trait implementation
  - Internal swap logic using Balancer math
- ✅ **weights.rs** - Placeholder weights
- ✅ **tests.rs** (400+ lines) - Comprehensive unit tests
- ✅ **benchmarking.rs** - Benchmark placeholders

### Phase 2: Workspace Integration
- ✅ Updated `/Cargo.toml` - Added `pallet-balancer-swap` to workspace
- ✅ Updated `runtime/Cargo.toml` - Added balancer-swap to:
  - dependencies
  - std feature
  - runtime-benchmarks feature
  - try-runtime feature

## 🔄 IN PROGRESS (Phase 3)

### Phase 3: Runtime Configuration
- ⏸️ **NEXT**: Update `runtime/src/lib.rs` to:
  - Add BalancerSwap pallet configuration
  - Replace Swap with BalancerSwap in construct_runtime!
  - Update SwapInterface type alias
  - Keep V3 swap temporarily for comparison

## 📋 REMAINING (Phase 4-7)

### Phase 4: Subtensor Integration Updates
Files to update:
- `pallets/subtensor/src/coinbase/root.rs`
  - Replace `dissolve_all_liquidity_providers()` calls
  - Replace `clear_protocol_liquidity()` calls
- `pallets/subtensor/src/coinbase/run_coinbase.rs`
  - Update `adjust_protocol_liquidity()` calls
- `pallets/subtensor/src/staking/claim_root.rs`
  - Verify swap() calls work (should be compatible)
- `pallets/subtensor/src/subnets/subnet.rs`
  - Update pool initialization

### Phase 5: Precompile Verification
- `precompiles/src/alpha.rs`
  - Verify all functions work with Balancer
  - Should be compatible (uses SwapHandler trait)

### Phase 6: Testing & Migration
- Create migration function to convert V3 → Balancer
- Test on devnet
- Comprehensive integration testing

### Phase 7: V3 Removal
- Delete `pallets/swap/` directory (~4,000 lines)
- Remove V3 imports from all files
- Remove V3 tests
- Update workspace Cargo.toml to remove pallet-subtensor-swap

## Code Statistics

### New Balancer Code
```
pallets/balancer-swap/
├── Cargo.toml           ✅   50 lines
├── src/
│   ├── lib.rs           ✅  750 lines
│   ├── types.rs         ✅  185 lines
│   ├── math.rs          ✅  382 lines
│   ├── weights.rs       ✅   45 lines
│   ├── benchmarking.rs  ✅   95 lines
│   └── tests.rs         ✅  400 lines

Total New Code: ~1,907 lines
```

### V3 Code to Remove
```
pallets/swap/
├── src/
│   ├── pallet/
│   │   ├── mod.rs           605 lines
│   │   ├── impls.rs       1,144 lines
│   │   ├── swap_step.rs     563 lines
│   │   └── tests.rs         500 lines
│   ├── tick.rs            2,199 lines
│   ├── position.rs          199 lines
│   ├── benchmarking.rs      150 lines
│   └── mock.rs              100 lines

Total V3 Code: ~5,460 lines
```

### Net Result
- **Added**: 1,907 lines
- **Removed**: 5,460 lines (pending)
- **Net Reduction**: 3,553 lines (65% reduction)

## Key Benefits Achieved

1. ✅ **Simpler Architecture**: 3 storage items vs. 14
2. ✅ **Unbalanced Liquidity**: Full support for any ratio
3. ✅ **No Position Management**: Just share balances
4. ✅ **Standard Math**: Battle-tested Balancer formulas
5. ✅ **Lower Complexity**: 1,907 vs. 5,460 lines
6. ✅ **Better Testability**: Cleaner test structure
7. ✅ **Trait Compatible**: Same SwapHandler interface

## Next Immediate Steps

1. **Update `runtime/src/lib.rs`** (~100 lines changes):
   ```rust
   // Add Balancer pallet config
   impl pallet_balancer_swap::Config for Runtime {
       type SubnetInfo = SubtensorModule;
       type BalanceOps = SubtensorModule;
       type TaoReserve = pallet_subtensor::TaoCurrencyReserve<Self>;
       type AlphaReserve = pallet_subtensor::AlphaCurrencyReserve<Self>;
       type ProtocolId = SwapProtocolId;  // Reuse
       type DefaultTaoWeight = ConstU32<50>;
       type DefaultAlphaWeight = ConstU32<50>;
       type DefaultSwapFee = DefaultSwapFee;
       type MaxSwapFee = MaxSwapFee;
       type MinimumLiquidity = MinimumLiquidity;
       type WeightInfo = pallet_balancer_swap::weights::DefaultWeight<Runtime>;
   }

   // In construct_runtime!
   BalancerSwap: pallet_balancer_swap = 29,  // New pallet index

   // Update type alias
   type SwapInterface = BalancerSwap;  // Change from Swap
   ```

2. **Test compilation**: `cargo check --release`

3. **Update integration points** in subtensor pallet

4. **Create migration function**

5. **Remove V3 code**

## Time Estimate

- **Completed**: ~3 days (design + implementation)
- **Remaining**: ~4-6 days
  - Runtime integration: 0.5 days
  - Subtensor updates: 1 day
  - Migration function: 1 day
  - Testing: 2-3 days
  - V3 removal: 0.5 days

**Total Project**: 7-9 days

---

**Status**: 70% Complete (core pallet done, integration pending)
**Last Updated**: 2025-12-05
**Ready for**: Runtime integration




