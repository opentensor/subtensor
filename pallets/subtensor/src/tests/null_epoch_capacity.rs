//! Null-epoch capacity measurements for claim-based miner emissions.
//!
//! Ignored by default. Run (release recommended for wall times):
//! ```text
//! cargo test -p pallet-subtensor --release null_epoch_capacity_report \
//!   -- --ignored --nocapture
//! ```
//!
//! Compares claim-based accrual vs immediate stake (main-like) at several UID
//! counts. Does not change `NULL_UID_BUDGET`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use super::mock::*;
use crate::*;
use sp_core::U256;
use sp_std::collections::btree_map::BTreeMap;
use std::time::Instant;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, MechId, NetUid, NetUidStorageIndex};

const CURRENT_NULL_MAX: u16 = crate::subnets::mechanism::NULL_UID_BUDGET;
const SIZES: [u16; 5] = [CURRENT_NULL_MAX, 4_000, 8_000, 16_000, 32_000];
const WARMUPS: u32 = 1;
const SAMPLES: u32 = 3;

fn count_keys() -> u64 {
    let mut n = 0u64;
    let mut key = sp_io::storage::next_key(&[]);
    while let Some(k) = key {
        n = n.saturating_add(1);
        key = sp_io::storage::next_key(&k);
    }
    n
}

fn setup_null_subnet(n: u16, claim_based: bool) -> NetUid {
    let netuid = NetUid::from(1);
    add_network_disable_commit_reveal(netuid, u16::MAX - 1, 0);
    SubtensorModule::set_max_allowed_uids(netuid, n);
    SubtensorModule::set_max_allowed_validators(netuid, 1);
    SubtensorModule::set_weights_set_rate_limit(netuid, 0);
    ClaimBasedMinerEmissions::<Test>::insert(netuid, claim_based);
    // Skip collateral / auto-stake side paths so the measured delta is miner
    // incentive accounting (pending accrue vs share-pool stake).
    RecycleOrBurn::<Test>::insert(netuid, RecycleOrBurnEnum::Burn);

    for uid in 0..n {
        let hotkey = U256::from(u32::from(uid).saturating_add(1));
        let coldkey = hotkey;
        Owner::<Test>::insert(hotkey, coldkey);
        Keys::<Test>::insert(netuid, uid, hotkey);
        Uids::<Test>::insert(netuid, hotkey, uid);
        IsNetworkMember::<Test>::insert(hotkey, netuid, true);
        BlockAtRegistration::<Test>::insert(netuid, uid, 0u64);
    }
    SubnetworkN::<Test>::insert(netuid, n);
    Active::<Test>::insert(netuid, vec![true; usize::from(n)]);
    Emission::<Test>::insert(netuid, vec![AlphaBalance::ZERO; usize::from(n)]);
    Consensus::<Test>::insert(netuid, vec![sp_runtime::PerU16::zero(); usize::from(n)]);
    Dividends::<Test>::insert(netuid, vec![sp_runtime::PerU16::zero(); usize::from(n)]);
    ValidatorTrust::<Test>::insert(netuid, vec![sp_runtime::PerU16::zero(); usize::from(n)]);
    let mut permits = vec![false; usize::from(n)];
    permits[0] = true;
    ValidatorPermit::<Test>::insert(netuid, permits);
    LastUpdate::<Test>::insert(NetUidStorageIndex::from(netuid), vec![1u64; usize::from(n)]);
    // Winner stake so Null elects UID 0; one dense weight row (Null loads only that).
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(1u32),
        &U256::from(1u32),
        netuid,
        1_000_000_000u64.into(),
    );
    let row: Vec<(u16, u16)> = (0..n).map(|uid| (uid, 1u16)).collect();
    Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0u16, row);

    System::set_block_number(1);
    // Bypass the Null UID-budget admission check so we can measure larger N
    // than today's `NULL_UID_BUDGET` without changing the production cap.
    SubtensorModule::set_epoch_consensus(netuid, EpochConsensus::Null);
    // Mint outstanding alpha so share-pool / recycle bookkeeping has headroom.
    SubnetAlphaOut::<Test>::insert(
        netuid,
        AlphaBalance::from(u64::from(n).saturating_mul(10_000)),
    );
    netuid
}

fn miner_incentives(n: u16) -> BTreeMap<U256, AlphaBalance> {
    (0..n)
        .map(|uid| {
            let hotkey = U256::from(u32::from(uid).saturating_add(1));
            (hotkey, AlphaBalance::from(1_000u64))
        })
        .collect()
}

fn measure_epoch_ms(netuid: NetUid, emission: AlphaBalance) -> f64 {
    for _ in 0..WARMUPS {
        let _ = SubtensorModule::epoch_mechanism(netuid, MechId::MAIN, emission);
    }
    let mut total = 0.0f64;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let _ = SubtensorModule::epoch_mechanism(netuid, MechId::MAIN, emission);
        total += start.elapsed().as_secs_f64() * 1_000.0;
    }
    total / f64::from(SAMPLES)
}

struct DistStats {
    ms: f64,
    key_delta: i64,
    pending_total: AlphaBalance,
}

fn measure_distribute(netuid: NetUid, n: u16, claim_based: bool) -> DistStats {
    let incentives = miner_incentives(n);
    let empty_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
    // Warm code path (may create stake / pending keys).
    for _ in 0..WARMUPS {
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaBalance::ZERO,
            incentives.clone(),
            empty_divs.clone(),
            empty_divs.clone(),
        );
    }
    // Time steady-state updates (keys already exist from warmup).
    let start = Instant::now();
    for _ in 0..SAMPLES {
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaBalance::ZERO,
            incentives.clone(),
            empty_divs.clone(),
            empty_divs.clone(),
        );
    }
    let ms = start.elapsed().as_secs_f64() * 1_000.0 / f64::from(SAMPLES);

    // Fresh key-delta sample: wipe miner accounting maps, then one distribute.
    let _ = PendingMinerAlpha::<Test>::clear_prefix(netuid, u32::MAX, None);
    PendingMinerAlphaTotal::<Test>::remove(netuid);
    if !claim_based {
        // Immediate stake may have created AlphaV2 / share rows during warmup;
        // leave them — delta then reflects updates-only (often ~0). For a cold
        // first-deposit delta, measure on a virgin subnet below in the report.
    }
    let keys_before = count_keys();
    SubtensorModule::distribute_dividends_and_incentives(
        netuid,
        AlphaBalance::ZERO,
        incentives,
        empty_divs.clone(),
        empty_divs,
    );
    let keys_after = count_keys();
    DistStats {
        ms,
        key_delta: (keys_after as i64).saturating_sub(keys_before as i64),
        pending_total: SubtensorModule::get_pending_miner_emission_total(netuid),
    }
}

#[test]
#[ignore = "manual Null epoch capacity report; run with --ignored --nocapture"]
fn null_epoch_capacity_report() {
    println!("uid_count,mode,epoch_ms,distribute_ms,storage_key_delta,pending_total_rao");
    for &n in &SIZES {
        for claim_based in [false, true] {
            new_test_ext(1).execute_with(|| {
                let netuid = setup_null_subnet(n, claim_based);
                let emission = AlphaBalance::from(u64::from(n).saturating_mul(2));
                let epoch_ms = measure_epoch_ms(netuid, emission);
                let dist = measure_distribute(netuid, n, claim_based);
                let mode = if claim_based {
                    "claim_based"
                } else {
                    "immediate_stake"
                };
                println!(
                    "{n},{mode},{:.3},{:.3},{},{}",
                    epoch_ms,
                    dist.ms,
                    dist.key_delta,
                    dist.pending_total.to_u64()
                );
            });
            // Cold first-deposit key delta (immediate stake creates share-pool rows).
            if !claim_based {
                new_test_ext(1).execute_with(|| {
                    let netuid = setup_null_subnet(n, false);
                    let incentives = miner_incentives(n);
                    let empty_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
                    let keys_before = count_keys();
                    SubtensorModule::distribute_dividends_and_incentives(
                        netuid,
                        AlphaBalance::ZERO,
                        incentives,
                        empty_divs.clone(),
                        empty_divs,
                    );
                    let keys_after = count_keys();
                    println!(
                        "{n},immediate_stake_cold_keys,0.000,0.000,{},0",
                        (keys_after as i64).saturating_sub(keys_before as i64)
                    );
                });
            }
        }
    }
}
