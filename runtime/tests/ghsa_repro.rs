//! Reproduction tests for the June 2026 security audit proxy-filter findings.
//!
//! These are *reproductions*: a passing test demonstrates the vulnerability is live.
//! The filter returns `true` when a call is ALLOWED for a proxy type. The bug in each
//! case is that a fund/ownership-moving call is ALLOWED for a proxy type that is meant
//! to forbid it.
//!
//! - GHSA-2026-001: NonTransfer / NonFungible proxies allow the coldkey-swap lifecycle
//!   (announce_coldkey_swap + swap_coldkey_announced) -> full account takeover.
//! - GHSA-2026-002: NonFungible allows swap_hotkey_v2 (call 72) though it denies the
//!   deprecated swap_hotkey (call 70); SwapHotkey allows only call 70, not the live v2.
//! - GHSA-2026-003: Owner proxy allows sudo_set_subnet_owner_hotkey (call 64) even though
//!   it explicitly excepts the duplicate alias sudo_set_sn_owner_hotkey (call 67).
#![allow(clippy::unwrap_used, unused_imports, dead_code)]

use frame_support::traits::InstanceFilter;
use node_subtensor_runtime::RuntimeCall;
use subtensor_runtime_common::{AccountId, NetUid, ProxyType, TaoBalance};

fn acct() -> AccountId {
    AccountId::new([0u8; 32])
}

// ---- coldkey-swap lifecycle calls ----
fn announce_coldkey_swap() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::announce_coldkey_swap {
        new_coldkey_hash: Default::default(),
    })
}
fn swap_coldkey_announced() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey_announced { new_coldkey: acct() })
}
fn swap_coldkey_legacy() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey {
        old_coldkey: acct(),
        new_coldkey: acct(),
        swap_cost: TaoBalance::from(0u64),
    })
}
fn transfer_stake() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake {
        destination_coldkey: acct(),
        hotkey: acct(),
        origin_netuid: NetUid::from(1),
        destination_netuid: NetUid::from(1),
        alpha_amount: Default::default(),
    })
}

// ---- hotkey-swap calls ----
fn swap_hotkey_v1() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey {
        hotkey: acct(),
        new_hotkey: acct(),
        netuid: Default::default(),
    })
}
fn swap_hotkey_v2() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey_v2 {
        hotkey: acct(),
        new_hotkey: acct(),
        netuid: Default::default(),
        keep_stake: false,
    })
}

// ---- owner-hotkey setter aliases ----
fn set_sn_owner_hotkey_c67() -> RuntimeCall {
    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_sn_owner_hotkey {
        netuid: Default::default(),
        hotkey: acct(),
    })
}
fn set_subnet_owner_hotkey_c64() -> RuntimeCall {
    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_subnet_owner_hotkey {
        netuid: Default::default(),
        hotkey: acct(),
    })
}

/// GHSA-2026-003 — the Owner proxy excepts sudo_set_sn_owner_hotkey (call 67) but the
/// duplicate alias sudo_set_subnet_owner_hotkey (call 64) is allowed by the AdminUtils::*
/// wildcard, bypassing the carve-out.
#[test]
fn ghsa_2026_003_owner_proxy_set_owner_hotkey_alias_bypass() {
    assert!(
        !ProxyType::Owner.filter(&set_sn_owner_hotkey_c67()),
        "precondition: Owner correctly excepts sudo_set_sn_owner_hotkey (call 67)"
    );
    assert!(
        !ProxyType::Owner.filter(&set_subnet_owner_hotkey_c64()),
        "regression (GHSA-2026-003 fixed): Owner must DENY the alias sudo_set_subnet_owner_hotkey (call 64), \
         which calls the same do_set_sn_owner_hotkey backend"
    );
}
