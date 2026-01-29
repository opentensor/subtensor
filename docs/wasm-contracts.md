# WebAssembly Smart Contracts

## Overview

Subtensor now supports WebAssembly (WASM) smart contract functionality through the integration of `pallet-contracts`, enabling developers to deploy and execute WASM smart contracts on the network. Contracts are written in [ink!](https://use.ink/), a Rust-based embedded domain-specific language (eDSL) for writing smart contracts on Substrate-based chains. For compatibility, WASM contracts can also be compiled from Solidity using [Solang](https://github.com/hyperledger-solang/solang).

> [!NOTE]
> If you're looking for information on EVM contracts, please see the documentation: https://docs.learnbittensor.org/evm-tutorials

## Getting Started

For general smart contract development on Subtensor, please refer to the official ink! documentation:
- [ink! Documentation](https://use.ink/docs/v5/)
- [ink! Getting Started Guide](https://use.ink/docs/v5/getting-started/setup)
- [ink! Examples](https://github.com/use-ink/ink-examples/tree/v5.x.x)

> [!WARNING]
> ink! `>= 6.0` drops support for `pallet-contracts`, please use `ink < 6.0`.
> See: https://github.com/use-ink/ink/releases/tag/v6.0.0-alpha

## Subtensor-Specific Features

### Chain Extension

Subtensor provides a custom chain extension that allows smart contracts to interact with Subtensor-specific functionality:

#### Available Functions

| Function ID | Name | Description | Parameters | Returns |
|------------|------|-------------|------------|---------|
| 0 | `get_stake_info_for_hotkey_coldkey_netuid` | Query stake information | `(AccountId, AccountId, NetUid)` | `Option<StakeInfo>` |
| 1 | `add_stake` | Delegate stake from coldkey to hotkey | `(AccountId, NetUid, TaoCurrency)` | Error code |
| 2 | `remove_stake` | Withdraw stake from hotkey back to coldkey | `(AccountId, NetUid, AlphaCurrency)` | Error code |
| 3 | `unstake_all` | Unstake all TAO from a hotkey | `(AccountId)` | Error code |
| 4 | `unstake_all_alpha` | Unstake all Alpha from a hotkey | `(AccountId)` | Error code |
| 5 | `move_stake` | Move stake between hotkeys | `(AccountId, AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 6 | `transfer_stake` | Transfer stake between coldkeys | `(AccountId, AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 7 | `swap_stake` | Swap stake allocations between subnets | `(AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 8 | `add_stake_limit` | Delegate stake with a price limit | `(AccountId, NetUid, TaoCurrency, TaoCurrency, bool)` | Error code |
| 9 | `remove_stake_limit` | Withdraw stake with a price limit | `(AccountId, NetUid, AlphaCurrency, TaoCurrency, bool)` | Error code |
| 10 | `swap_stake_limit` | Swap stake between subnets with price limit | `(AccountId, NetUid, NetUid, AlphaCurrency, TaoCurrency, bool)` | Error code |
| 11 | `remove_stake_full_limit` | Fully withdraw stake with optional price limit | `(AccountId, NetUid, Option<TaoCurrency>)` | Error code |
| 12 | `set_coldkey_auto_stake_hotkey` | Configure automatic stake destination | `(NetUid, AccountId)` | Error code |
| 13 | `add_proxy` | Add a staking proxy for the caller | `(AccountId)` | Error code |
| 14 | `remove_proxy` | Remove a staking proxy for the caller | `(AccountId)` | Error code |
| 15 | `get_alpha_price` | Get the current alpha price for a subnet | `(NetUid)` | `u64` (price × 10⁹) |
| 16 | `add_stake_v2` | Add stake with explicit coldkey (proxy-aware) | `(AccountId, AccountId, NetUid, TaoCurrency)` | Error code |
| 17 | `remove_stake_v2` | Remove stake with explicit coldkey | `(AccountId, AccountId, NetUid, AlphaCurrency)` | Error code |
| 18 | `unstake_all_v2` | Unstake all TAO with explicit coldkey | `(AccountId, AccountId)` | Error code |
| 19 | `unstake_all_alpha_v2` | Unstake all Alpha with explicit coldkey | `(AccountId, AccountId)` | Error code |
| 20 | `move_stake_v2` | Move stake between hotkeys with explicit coldkey | `(AccountId, AccountId, AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 21 | `transfer_stake_v2` | Transfer stake between coldkeys (requires Transfer proxy) | `(AccountId, AccountId, AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 22 | `swap_stake_v2` | Swap stake between subnets with explicit coldkey | `(AccountId, AccountId, NetUid, NetUid, AlphaCurrency)` | Error code |
| 23 | `add_stake_limit_v2` | Add stake with price limit and explicit coldkey | `(AccountId, AccountId, NetUid, TaoCurrency, TaoCurrency, bool)` | Error code |
| 24 | `remove_stake_limit_v2` | Remove stake with price limit and explicit coldkey | `(AccountId, AccountId, NetUid, AlphaCurrency, TaoCurrency, bool)` | Error code |
| 25 | `swap_stake_limit_v2` | Swap stake with price limit and explicit coldkey | `(AccountId, AccountId, NetUid, NetUid, AlphaCurrency, TaoCurrency, bool)` | Error code |
| 26 | `remove_stake_full_limit_v2` | Full unstake with price limit and explicit coldkey | `(AccountId, AccountId, NetUid, Option<TaoCurrency>)` | Error code |
| 27 | `set_coldkey_auto_stake_hotkey_v2` | Set auto-stake hotkey with explicit coldkey | `(AccountId, NetUid, AccountId)` | Error code |

Example usage in your ink! contract:
```rust
#[ink::chain_extension(extension = 0)]
pub trait SubtensorExtension {
    type ErrorCode = SubtensorError;

    #[ink(function = 0)]
    fn get_stake_info(
        hotkey: AccountId,
        coldkey: AccountId,
        netuid: u16,
    ) -> Result<Option<StakeInfo>, SubtensorError>;
}
```

#### Error Codes

Chain extension functions that modify state return error codes as `u32` values. The following codes are defined:

| Code | Name | Description |
|------|------|-------------|
| 0 | `Success` | Operation completed successfully |
| 1 | `RuntimeError` | Unknown runtime error occurred |
| 2 | `NotEnoughBalanceToStake` | Insufficient balance to complete stake operation |
| 3 | `NonAssociatedColdKey` | Coldkey is not associated with the hotkey |
| 4 | `BalanceWithdrawalError` | Error occurred during balance withdrawal |
| 5 | `NotRegistered` | Hotkey is not registered in the subnet |
| 6 | `NotEnoughStakeToWithdraw` | Insufficient stake available for withdrawal |
| 7 | `TxRateLimitExceeded` | Transaction rate limit has been exceeded |
| 8 | `SlippageTooHigh` | Price slippage exceeds acceptable threshold |
| 9 | `SubnetNotExists` | Specified subnet does not exist |
| 10 | `HotKeyNotRegisteredInSubNet` | Hotkey is not registered in the specified subnet |
| 11 | `SameAutoStakeHotkeyAlreadySet` | Auto-stake hotkey is already configured |
| 12 | `InsufficientBalance` | Account has insufficient balance |
| 13 | `AmountTooLow` | Transaction amount is below minimum threshold |
| 14 | `InsufficientLiquidity` | Insufficient liquidity for swap operation |
| 15 | `SameNetuid` | Source and destination subnets are the same |
| 16 | `ProxyTooMany` | Too many proxies registered |
| 17 | `ProxyDuplicate` | Proxy already exists |
| 18 | `ProxyNoSelfProxy` | Cannot add self as proxy |
| 19 | `ProxyNotFound` | Proxy relationship not found |
| 20 | `NotAuthorizedProxy` | Caller is not an authorized proxy for the account |

#### V2 Functions (Proxy-Aware)

Functions 16-27 are V2 versions that accept an explicit `coldkey` parameter as the first argument. These functions:

- If `coldkey == caller`: Execute directly (no proxy check needed)
- If `coldkey != caller`: Verify caller has appropriate proxy permissions for coldkey

**Proxy Types Required:**
- Most V2 functions require `ProxyType::Staking`
- `transfer_stake_v2` (ID 21) requires `ProxyType::Transfer`

### Call Filter

For security, contracts can only directly dispatch a limited set of runtime calls:

**Whitelisted Calls:**
- `Proxy::proxy` - Execute proxy calls

All other runtime calls are restricted and cannot be dispatched from contracts.

### Configuration Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Maximum code size | 128 KB | Maximum size of contract WASM code |
| Call stack depth | 5 frames | Maximum nested contract call depth |
| Runtime memory | 1 GB | Memory available during contract execution |
| Validator runtime memory | 2 GB | Memory available for validators |
| Transient storage | 1 MB | Maximum transient storage size |


## Additional Resources

- [cargo-contract CLI Tool](https://github.com/paritytech/cargo-contract)
- [Contracts UI](https://contracts-ui.substrate.io/)
