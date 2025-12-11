# V3 Pool Feature Inventory

Complete inventory of all V3 pool-related code in the subtensor repository for systematic removal.

Last Updated: 2025-12-05

---

## Table of Contents
1. [Storage Maps & State](#storage-maps--state)
2. [Core Types & Structs](#core-types--structs)
3. [Pallet Extrinsics](#pallet-extrinsics)
4. [Internal Functions](#internal-functions)
5. [Trait Implementations](#trait-implementations)
6. [Tests](#tests)
7. [Precompiles & External Interfaces](#precompiles--external-interfaces)
8. [Runtime Integration](#runtime-integration)
9. [Dependencies & Imports](#dependencies--imports)
10. [Related Pallets](#related-pallets)

---

## 1. Storage Maps & State

### In `pallets/swap/src/pallet/mod.rs`

#### V3-Specific Storage Items
```rust
// Lines 82-157
- FeeRate<T>              // Fee rate per subnet (u16), Line 83
- FeeGlobalTao<T>         // Global accumulated TAO fees per subnet, Line 87
- FeeGlobalAlpha<T>       // Global accumulated Alpha fees per subnet, Line 91
- Ticks<T>                // All tick data (double map: netuid + tick index), Line 95
- SwapV3Initialized<T>    // Whether V3 is initialized per subnet, Line 99
- AlphaSqrtPrice<T>       // Current sqrt price per subnet, Line 103
- CurrentTick<T>          // Current tick index per subnet, Line 107
- CurrentLiquidity<T>     // Current liquidity per subnet, Line 111
- EnabledUserLiquidity<T> // Whether user liquidity operations are enabled, Line 117
- Positions<T>            // User liquidity positions (NMap: netuid + account + position ID), Line 122
- LastPositionId<T>       // Counter for position IDs, Line 135
- TickIndexBitmapWords<T> // Bitmap for efficient tick searches, Line 139
- ScrapReservoirTao<T>    // TAO scraps from protocol fees, Line 152
- ScrapReservoirAlpha<T>  // Alpha scraps from protocol fees, Line 156
```

---

## 2. Core Types & Structs

### In `pallets/swap/src/tick.rs`
```rust
- Tick                    // Lines 82-89: Liquidity and fee tracking per tick
- TickIndex               // Lines 98-475: Tick index with bounds checking
- ActiveTickIndexManager  // Lines 477-701: Manages active tick bitmap
- LayerLevel              // Lines 704-712: Enum for bitmap layers (Top/Middle/Bottom)
- BitmapLayer             // Lines 715-725: Layer position in bitmap
- TickIndexBitmap         // Lines 728-842: 3-layer bitmap representation
- TickMathError           // Lines 1187-1208: Error types for tick math
```

### In `pallets/swap/src/position.rs`
```rust
- Position<T>             // Lines 18-160: Liquidity position struct
- PositionId              // Lines 162-198: Unique position identifier
```

### In `pallets/swap/src/pallet/swap_step.rs`
```rust
- BasicSwapStep<T, PaidIn, PaidOut>  // Lines 14-227: Swap step execution
- SwapStepResult                     // Lines 547-556: Results from a swap step
- SwapStepAction                     // Lines 558-562: Enum (Crossing/Stop)
- SwapStep trait                     // Lines 494-544: Trait for swap step logic
```

---

## 3. Pallet Extrinsics

### In `pallets/swap/src/pallet/mod.rs`

```rust
// All callable functions (Lines 291-603)
1. set_fee_rate(origin, netuid, rate)              // Line 301
2. toggle_user_liquidity(origin, netuid, enable)   // Line 334
3. add_liquidity(origin, hotkey, netuid, ...)      // Line 372
4. remove_liquidity(origin, hotkey, netuid, ...)   // Line 442
5. modify_position(origin, hotkey, netuid, ...)    // Line 501
```

---

## 4. Internal Functions

### In `pallets/swap/src/pallet/impls.rs`

#### V3 Initialization & Management
```rust
// Lines 48-1144
- current_price(netuid)                              // Line 49
- maybe_initialize_v3(netuid)                        // Line 71
- get_proportional_alpha_tao_and_remainders(...)    // Line 116
- adjust_protocol_liquidity(netuid, tao, alpha)     // Line 152
- do_swap<Order>(...)                                // Line 240
- swap_inner<Order>(...)                             // Line 277
- calculate_fee_amount(...)                          // Line 362
- find_closest_lower_active_tick(...)                // Line 384
- find_closest_higher_active_tick(...)               // Line 389
- current_liquidity_safe(netuid)                     // Line 395
- do_add_liquidity(...)                              // Line 430
- add_liquidity_not_insert(...)                      // Line 477
- do_remove_liquidity(...)                           // Line 518
- do_modify_position(...)                            // Line 559
- add_liquidity_at_index(...)                        // Line 689
- remove_liquidity_at_index(...)                     // Line 730
- update_liquidity_if_needed(...)                    // Line 765
- clamp_sqrt_price(...)                              // Line 786
- count_positions(...)                               // Line 801
- protocol_account_id()                              // Line 809
- min_price_inner<C>()                               // Line 813
- max_price_inner<C>()                               // Line 821
- do_dissolve_all_liquidity_providers(netuid)        // Line 831
- do_clear_protocol_liquidity(netuid)                // Line 947
```

---

## 5. Trait Implementations

### In `pallets/swap/src/pallet/impls.rs`

```rust
// Lines 1021-1143
- DefaultPriceLimit<TaoCurrency, AlphaCurrency> for Pallet<T>     // Line 1021
- DefaultPriceLimit<AlphaCurrency, TaoCurrency> for Pallet<T>     // Line 1027
- SwapEngine<O> for Pallet<T>                                      // Line 1033
- SwapHandler for Pallet<T>                                        // Line 1063
  - swap<O>(...)                                                   // Line 1064
  - sim_swap<O>(...)                                               // Line 1078
  - approx_fee_amount<C>(...)                                      // Line 1107
  - current_alpha_price(...)                                       // Line 1111
  - min_price<C>()                                                 // Line 1115
  - max_price<C>()                                                 // Line 1119
  - adjust_protocol_liquidity(...)                                 // Line 1123
  - is_user_liquidity_enabled(...)                                 // Line 1131
  - dissolve_all_liquidity_providers(...)                          // Line 1134
  - toggle_user_liquidity(...)                                     // Line 1137
  - clear_protocol_liquidity(...)                                  // Line 1140
```

### In `pallets/swap/src/pallet/swap_step.rs`

```rust
// Lines 229-492
- SwapStep<T, TaoCurrency, AlphaCurrency> for BasicSwapStep       // Line 229
- SwapStep<T, AlphaCurrency, TaoCurrency> for BasicSwapStep       // Line 348
```

---

## 6. Tests

### Test Files
```
pallets/swap/src/pallet/tests.rs              // All V3 pool tests
pallets/swap/src/tick.rs                      // Lines 1210-2198: Tick/bitmap tests
pallets/swap/src/benchmarking.rs              // V3 benchmarking
```

### Test Files Using V3 (in subtensor pallet)
```
pallets/subtensor/src/tests/claim_root.rs     // Uses SwapHandler
pallets/subtensor/src/tests/subnet.rs         // Uses V3 initialization
pallets/subtensor/src/tests/staking.rs        // Uses V3 features
pallets/subtensor/src/tests/networks.rs       // Tests network with V3
pallets/subtensor/src/tests/coinbase.rs       // Tests emissions with V3
pallets/subtensor/src/tests/migration.rs      // Tests V3 migrations
```

### Mock Files
```
pallets/swap/src/mock.rs                      // V3 test runtime config
pallets/subtensor/src/tests/mock.rs           // References pallet_subtensor_swap
pallets/transaction-fee/src/tests/mock.rs     // References swap
pallets/admin-utils/src/tests/mock.rs         // References swap
chain-extensions/src/mock.rs                  // References swap
```

---

## 7. Precompiles & External Interfaces

### In `precompiles/src/alpha.rs`
```rust
// Lines 1-217: All functions interact with SwapHandler
- get_alpha_price(netuid)                     // Line 37 - Uses current_alpha_price
- sim_swap_tao_for_alpha(netuid, tao)         // Line 103 - Uses sim_swap
- sim_swap_alpha_for_tao(netuid, alpha)       // Line 119 - Uses sim_swap
- get_sum_alpha_price()                       // Line 192 - Iterates all subnet prices
```

### Solidity Interfaces
```
precompiles/src/solidity/alpha.sol            // EVM interface definitions
precompiles/src/solidity/alpha.abi            // ABI for alpha precompile
```

---

## 8. Runtime Integration

### In `runtime/src/lib.rs`

```rust
// Lines 1119-1139: Swap pallet configuration
parameter_types! {
    pub const SwapProtocolId: PalletId = PalletId(*b"ten/swap");
    pub const SwapMaxFeeRate: u16 = 10000;
    pub const SwapMaxPositions: u32 = 100;
    pub const SwapMinimumLiquidity: u64 = 1_000;
    pub const SwapMinimumReserve: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1_000_000) };
}

impl pallet_subtensor_swap::Config for Runtime {
    type SubnetInfo = SubtensorModule;
    type BalanceOps = SubtensorModule;
    type ProtocolId = SwapProtocolId;
    type TaoReserve = pallet_subtensor::TaoCurrencyReserve<Self>;
    type AlphaReserve = pallet_subtensor::AlphaCurrencyReserve<Self>;
    type MaxFeeRate = SwapMaxFeeRate;
    type MaxPositions = SwapMaxPositions;
    type MinimumLiquidity = SwapMinimumLiquidity;
    type MinimumReserve = SwapMinimumReserve;
    type WeightInfo = pallet_subtensor_swap::weights::DefaultWeight<Runtime>;
}

// Line 1108: SwapInterface = Swap
type SwapInterface = Swap;

// Line 1570: Pallet declaration in construct_runtime!
Swap: pallet_subtensor_swap = 28,

// Lines 2480-2527: Runtime API implementation
impl pallet_subtensor_swap_runtime_api::SwapRuntimeApi<Block> for Runtime {
    fn current_alpha_price(netuid: NetUid) -> u64
    fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoCurrency) -> SimSwapResult
    fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaCurrency) -> SimSwapResult
}
```

### Proxy Types
```rust
// Line 664: ProxyType::SwapHotkey support
```

---

## 9. Dependencies & Imports

### Key Dependencies
```toml
# In Cargo.toml files
pallet-subtensor-swap         // Main V3 pallet
subtensor-swap-interface      // Trait definitions
pallet-subtensor-swap-rpc     // RPC interface (if exists)
pallet-subtensor-swap-runtime-api // Runtime API definitions
```

### Import Locations
```rust
// In pallets/subtensor/src files that use V3:
use subtensor_swap_interface::{SwapHandler, Order, SwapEngine}

// Major importers:
pallets/subtensor/src/coinbase/root.rs
pallets/subtensor/src/coinbase/run_coinbase.rs
pallets/subtensor/src/subnets/subnet.rs
pallets/subtensor/src/staking/claim_root.rs
precompiles/src/alpha.rs
runtime/src/lib.rs
```

---

## 10. Related Pallets

### `pallets/swap-interface/` (Trait Definitions)
```rust
// Lines 1-99
- SwapEngine<O: Order> trait
- SwapHandler trait
- DefaultPriceLimit trait
- SwapResult<PaidIn, PaidOut> struct
- Order trait module (order.rs)
```

### Integration Points in `pallets/subtensor/`

#### In `coinbase/root.rs`
```rust
// Lines 18-24, 200-400+
- Uses SwapHandler::dissolve_all_liquidity_providers()
- Uses SwapHandler::clear_protocol_liquidity()
- Uses SwapHandler::toggle_user_liquidity()
- Called during subnet removal (do_remove_network)
```

#### In `coinbase/run_coinbase.rs`
```rust
- Uses SwapHandler::adjust_protocol_liquidity()
- Called during emission distribution
```

#### In `subnets/subnet.rs`
```rust
// Line 4: use subtensor_swap_interface::SwapHandler;
- Network creation may initialize V3
```

#### In `staking/claim_root.rs`
```rust
- Uses SwapHandler for alpha->tao conversions
- swap() function calls
```

---

## 11. V3-Specific Math & Algorithms

### Uniswap V3 Math (in `tick.rs`)
```rust
// Lines 844-1130: Core Uniswap V3 formulas
- get_sqrt_ratio_at_tick(tick: i32)           // Line 845
- get_tick_at_sqrt_ratio(sqrt_price_x_96)     // Line 1025
- u64f64_to_u256_q64_96(value)                // Line 1133
- u256_q64_96_to_u64f64(value)                // Line 1157
- q_to_u64f64(x, frac_bits)                   // Line 1162
```

### Concentrated Liquidity Logic
- Price is encoded as square root (sqrt_price)
- Liquidity is concentrated in price ranges (ticks)
- Three-layer bitmap for efficient tick searching
- Fees accumulate globally and are tracked per tick

---

## 12. Events

### In `pallets/swap/src/pallet/mod.rs`
```rust
// Lines 159-243
- FeeRateSet { netuid, rate }
- UserLiquidityToggled { netuid, enable }
- LiquidityAdded { coldkey, hotkey, netuid, position_id, ... }
- LiquidityRemoved { coldkey, hotkey, netuid, position_id, ... }
- LiquidityModified { coldkey, hotkey, netuid, position_id, ... }
```

---

## 13. Errors

### In `pallets/swap/src/pallet/mod.rs`
```rust
// Lines 245-289
- FeeRateTooHigh
- InsufficientInputAmount
- InsufficientLiquidity
- PriceLimitExceeded
- InsufficientBalance
- LiquidityNotFound
- InvalidTickRange
- MaxPositionsExceeded
- TooManySwapSteps
- InvalidLiquidityValue
- ReservesTooLow
- MechanismDoesNotExist
- UserLiquidityDisabled
- SubtokenDisabled
```

---

## 14. Constants & Configuration

```rust
// In pallets/swap/src/pallet/impls.rs
const MAX_SWAP_ITERATIONS: u16 = 1000;          // Line 24

// In pallets/swap/src/tick.rs
const MIN_TICK: i32 = -887272;                  // Line 55
const MAX_TICK: i32 = -MIN_TICK;                // Line 56
const MIN_SQRT_RATIO: U256 = ...                // Line 58
const MAX_SQRT_RATIO: U256 = ...                // Line 59-60

// Plus many U256 constants for tick math (Lines 24-53)
```

---

## 15. Weight Functions

### In `pallets/swap/src/weights.rs`
```rust
- set_fee_rate()
- toggle_user_liquidity()
- add_liquidity()
- remove_liquidity()
- modify_position()
```

---

## 16. Benchmarking

### In `pallets/swap/src/benchmarking.rs`
```rust
- All benchmarks for V3 extrinsics
- Test scenarios for worst-case execution
```

---

## Summary Statistics

- **Storage Items**: 14 V3-specific storage maps
- **Extrinsics**: 5 callable functions
- **Internal Functions**: 30+ helper functions
- **Core Types**: 10 major structs/enums
- **Trait Implementations**: 6 trait impls
- **Test Files**: 8+ files with V3 tests
- **Integration Points**: 5+ locations in subtensor pallet
- **Precompile Functions**: 4 EVM-exposed functions
- **Events**: 5 events
- **Errors**: 14 error variants

---

## Removal Strategy Notes

1. **Start with**: Remove extrinsics and public interfaces
2. **Then**: Remove internal swap logic (swap_step, tick math)
3. **Next**: Remove storage items and state
4. **After**: Clean up tests and benchmarks
5. **Update**: Runtime configuration and pallet registration
6. **Remove**: Precompile integration
7. **Clean**: Trait implementations and interfaces
8. **Final**: Remove entire pallet directory and crate references

---

## Cross-Pallet Dependencies

**Critical**: Before removing V3, these must be addressed:

1. `pallets/subtensor/` calls `SwapHandler` trait methods
2. `runtime/src/lib.rs` configures the Swap pallet
3. `precompiles/src/alpha.rs` exposes V3 to EVM
4. Tests in `pallets/subtensor/src/tests/` depend on V3
5. Mock configurations reference the swap pallet

**Suggested Order**:
1. Replace SwapHandler calls with V2 equivalents in subtensor pallet
2. Update runtime to remove Swap pallet
3. Remove or update precompiles
4. Update/remove affected tests
5. Delete swap pallet directory
6. Remove swap-interface crate
7. Update Cargo.toml dependencies across workspace

---

End of Inventory



