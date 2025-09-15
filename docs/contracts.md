# Smart Contracts on Subtensor

## Overview

Subtensor now supports smart contract functionality through the integration of `pallet-contracts`, enabling developers to deploy and execute WebAssembly (WASM) smart contracts on the network. Contracts are written in [ink!](https://use.ink/), a Rust-based embedded domain-specific language (eDSL) for writing smart contracts on Substrate-based chains.

## Getting Started

For general smart contract development on Subtensor, please refer to the official ink! documentation:
- [ink! Documentation](https://use.ink/docs/v5/)
- [ink! Getting Started Guide](https://use.ink/docs/v5/getting-started/setup)
- [ink! Examples](https://github.com/use-ink/ink-examples/tree/v5.x.x)

## Subtensor-Specific Features

### Chain Extension

Subtensor provides a custom chain extension that allows smart contracts to interact with Subtensor-specific functionality:

#### Available Functions

| Function ID | Name | Description | Parameters | Returns |
|------------|------|-------------|------------|---------|
| 1001 | `get_stake_info_for_hotkey_coldkey_netuid` | Query stake information | `(AccountId32, AccountId32, NetUid)` | Stake information |

Example usage in your ink! contract:
```rust
#[ink::chain_extension(extension = 0)]
pub trait SubtensorExtension {
    type ErrorCode = SubtensorError;

    #[ink(function = 1001)]
    fn get_stake_info(
        hotkey: AccountId,
        coldkey: AccountId,
        netuid: u16,
    ) -> Result<Option<StakeInfo>, SubtensorError>;
}
```

### Call Filter

For security, contracts can only dispatch a limited set of runtime calls:

**Whitelisted Calls:**
- `SubtensorModule::move_stake` - Move stake between hotkeys
- `SubtensorModule::transfer_stake` - Transfer stake between accounts
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
