use super::*;
use crate::subtensor_migration;
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;
use substrate_fixed::types::U64F64;

pub fn decode_account_id32<T: Config>(ss58_string: &str) -> Option<T::AccountId> {
    let account_id32: AccountId32 = AccountId32::from_ss58check(ss58_string).ok()?;
    let mut account_id32_slice: &[u8] = account_id32.as_ref();
    T::AccountId::decode(&mut account_id32_slice).ok()
}

struct HotkeySwapFix {
    old_hotkey_ss58: &'static str,
    new_hotkey_ss58: &'static str,
    netuid: u16,
}

subtensor_migration! {
    /// Fixes the consequences of a bug in `perform_hotkey_swap_on_one_subnet` where
    /// `transfer_root_claimable_for_new_hotkey` unconditionally transferred the **entire**
    /// `RootClaimable` BTreeMap (all subnets) from the old hotkey to the new hotkey, even
    /// during a single-subnet swap.
    ///
    /// This left the old hotkey with:
    ///   - `RootClaimable[old_hotkey]` = empty (wiped for ALL subnets)
    ///   - `RootClaimed[(subnet, old_hotkey, coldkey)]` = old watermarks (for non-swapped subnets)
    ///
    /// Resulting in `owed = claimable_rate * root_stake - root_claimed = 0 - positive = negative → 0`,
    /// effectively freezing root dividends for the old hotkey.
    ///
    /// Remediation: restore the pre-swap `RootClaimable` and `RootClaimed` storage maps
    mainnet_only fn migrate_fix_root_claimed_overclaim<T: Config>() -> Weight {
        let mut weight = Weight::zero();
        let mut claimed_restored: u64 = 0;

        let fixes: &[HotkeySwapFix] = &[
            HotkeySwapFix {
                old_hotkey_ss58: "5GmvyePN9aYErXBBhBnxZKGoGk4LKZApE4NkaSzW62CYCYNA",
                new_hotkey_ss58: "5H6BqkzjYvViiqp7rQLXjpnaEmW7U9CoKxXhQ4efMqtX1mQw",
                netuid: 27,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5CmKE9k1z1DDQBh81nfwRtbLq22mgS8wMPS9h36LVe4oGJTK",
                new_hotkey_ss58: "5EnpBz2DoMTzMztFSVPSpi8jP2yfGadU6kgZgsjqnfvonMgu",
                netuid: 9,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5C4s95N2JJbWwPPAr8JYwQBZQwxbZTYGjYbm6XtH2LgYV8Zx",
                new_hotkey_ss58: "5ChzWkapDYgVxT88ZmBQS8QM63V9VWSA3eFpSipsX2xbTNZN",
                netuid: 13,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5GHrTeuFnJYjNJx773URbYb9Pk3bRRDiJHJFBNECZpjGqZPY",
                new_hotkey_ss58: "5DAmVrUgpTX9xmRyZ7R3UUFNSzh7ZNY6qYxv9N4VeCq6mHHL",
                netuid: 65,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5EtM9iXMAYRsmt6aoQAoWNDX6yaBnjhmnEQhWKv8HpwkVtML",
                new_hotkey_ss58: "5ECzcM7sixWNEeD6RbpeEHW1YcYMFejwHuvDBgQxVSjGyrMS",
                netuid: 11,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5DF3nhgzpr4EZas8dXZYa4mYZBxRCU7AuiCV7Qs2JWAGA6sY",
                netuid: 41,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5E4pFBKCyk2RxQqifEBu37jb5vgoj9ZrVS7iQdQy4PNr33Ge",
                netuid: 44,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5DhQbRT3ZfHcVumNtAm5BbzeGHrFRHHi7nofgu76VWipnGSb",
                netuid: 50,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5Gj37iVQG5hMSxU3AE89x5p3aEEfPZk6Rtmtbwepght4tbri",
                netuid: 51,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5DyM1rxnDu8QSjbbh5bPV2GMK6UTPRXdUM6mNViBBut9Ma6w",
                netuid: 54,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5Ci5t4vPK3eCGhFWneB58fodg3x9oS2m8seKoDApFKUqyw4e",
                netuid: 64,
            },
            HotkeySwapFix {
                old_hotkey_ss58: "5HK5tp6t2S59DywmHRWPBVJeJ86T61KjurYqeooqj8sREpeN",
                new_hotkey_ss58: "5Et5VQUMX7VqGyvZycjv5FBBC5FQbLGUJiRMWMnEVnMLXKm9",
                netuid: 93,
            },
        ];

        let root_netuid = NetUid::from(0);

        for fix in fixes {
            let netuid = NetUid::from(fix.netuid);

            let (old_hotkey, new_hotkey) = match (
                decode_account_id32::<T>(fix.old_hotkey_ss58),
                decode_account_id32::<T>(fix.new_hotkey_ss58),
            ) {
                (Some(old), Some(new)) => (old, new),
                _ => {
                    log::error!(
                        "Failed to decode hotkeys for netuid {}, skipping",
                        fix.netuid
                    );
                    continue;
                }
            };

            // Reverting the Root Claimable because it only should happen for root subnet
            Pallet::<T>::transfer_root_claimable_for_new_hotkey(&new_hotkey, &old_hotkey);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));

            // Collect all coldkeys that have non-zero alpha on root subnet
            // (meaning they had root stake at swap time)
            let alpha_on_swapped_subnet: alloc::vec::Vec<T::AccountId> =
                Alpha::<T>::iter_prefix((&new_hotkey,))
                    .filter(|((coldkey, netuid_alpha), _)| {
                        // Must be on the subnet that was swapped
                        if *netuid_alpha != netuid {
                            return false;
                        }
                        // Must have non-zero alpha on root subnet for old hotkey
                        // (guards against reverting claims for keys with no root stake)
                        let root_alpha = Alpha::<T>::get((&old_hotkey, coldkey, root_netuid));
                        root_alpha != U64F64::from_num(0u64)
                    })
                    .map(|((coldkey, _), _)| coldkey)
                    .collect();

            weight = weight.saturating_add(
                T::DbWeight::get().reads((alpha_on_swapped_subnet.len() as u64).saturating_mul(2)),
            );

            // Revert RootClaimed for each qualifying coldkey
            for coldkey in alpha_on_swapped_subnet {
                claimed_restored = claimed_restored.saturating_add(1);
                Pallet::<T>::transfer_root_claimed_for_new_keys(
                    netuid,
                    &new_hotkey,
                    &old_hotkey,
                    &coldkey,
                    &coldkey,
                );
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
            }
        }

        log::info!(
            "migrate_fix_root_claimed_overclaim: claimed restored: {claimed_restored}"
        );

        weight
    }
}
