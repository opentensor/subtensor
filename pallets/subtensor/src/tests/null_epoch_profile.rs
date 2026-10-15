//! Phase breakdown profiler for Null `epoch_mechanism` + persist.
//!
//! Analysis-only (ignored). Run:
//! ```text
//! cargo test -p pallet-subtensor --release null_epoch_phase_profile \
//!   -- --ignored --nocapture
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use super::mock::*;
use crate::*;
use sp_core::U256;
use std::time::Instant;
use subtensor_runtime_common::{AlphaBalance, MechId, NetUid, NetUidStorageIndex};

const SIZES: [u16; 4] = [2_500, 8_000, 16_000, 32_000];
const WARMUPS: u32 = 1;
const SAMPLES: u32 = 3;

fn setup_null(n: u16) -> NetUid {
    let netuid = NetUid::from(1);
    add_network_disable_commit_reveal(netuid, u16::MAX - 1, 0);
    SubtensorModule::set_max_allowed_uids(netuid, n);
    SubtensorModule::set_max_allowed_validators(netuid, 1);
    SubtensorModule::set_weights_set_rate_limit(netuid, 0);
    ClaimBasedMinerEmissions::<Test>::insert(netuid, true);
    for uid in 0..n {
        let hotkey = U256::from(u32::from(uid).saturating_add(1));
        Owner::<Test>::insert(hotkey, hotkey);
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
    // Stake winner UID 0 heavily; a few other stakers for dividend path.
    for uid in 0..n.min(64) {
        let hot = U256::from(u32::from(uid).saturating_add(1));
        let amount = if uid == 0 {
            1_000_000_000u64
        } else {
            1_000_000u64
        };
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hot,
            &hot,
            netuid,
            amount.into(),
        );
    }
    let row: Vec<(u16, u16)> = (0..n).map(|uid| (uid, 1u16)).collect();
    Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0u16, row);
    System::set_block_number(10);
    SubtensorModule::set_epoch_consensus(netuid, EpochConsensus::Null);
    netuid
}

fn avg_ms(samples: u32, mut f: impl FnMut()) -> f64 {
    for _ in 0..WARMUPS {
        f();
    }
    let start = Instant::now();
    for _ in 0..samples {
        f();
    }
    start.elapsed().as_secs_f64() * 1_000.0 / f64::from(samples)
}

#[test]
#[ignore = "Null epoch phase profile; run with --ignored --nocapture"]
fn null_epoch_phase_profile() {
    println!(
        "n,keys_iter_ms,last_update_ms,block_at_reg_ms,null_stake_ms,weight_row_ms,epoch_total_ms,persist_ms,voting_power_ms,epoch_plus_persist_ms"
    );
    for &n in &SIZES {
        new_test_ext(1).execute_with(|| {
            let netuid = setup_null(n);
            let index = NetUidStorageIndex::from(netuid);
            let emission = AlphaBalance::from(u64::from(n).saturating_mul(2));

            let keys_ms = avg_ms(SAMPLES, || {
                let _: alloc::collections::BTreeMap<_, _> = Keys::<Test>::iter_prefix(netuid)
                    .map(|(uid, hotkey)| (hotkey, uid))
                    .collect();
            });
            let last_ms = avg_ms(SAMPLES, || {
                let _ = SubtensorModule::get_last_update(index);
            });
            let reg_ms = avg_ms(SAMPLES, || {
                let _ = SubtensorModule::get_block_at_registration(netuid);
            });
            let stake_ms = avg_ms(SAMPLES, || {
                let _ = SubtensorModule::get_null_stake_weights_for_network(netuid);
            });
            let weight_ms = avg_ms(SAMPLES, || {
                let _ = Weights::<Test>::get(index, 0u16);
            });
            let epoch_ms = avg_ms(SAMPLES, || {
                let _ = SubtensorModule::epoch_mechanism(netuid, MechId::MAIN, emission);
            });
            // Persist + voting power on a fresh output each sample.
            let mut persist_ms = 0.0f64;
            let mut vote_ms = 0.0f64;
            for _ in 0..WARMUPS {
                let out = SubtensorModule::epoch_mechanism(netuid, MechId::MAIN, emission);
                SubtensorModule::persist_mechanism_epoch_terms(netuid, MechId::MAIN, out.as_map());
                SubtensorModule::persist_netuid_epoch_terms(netuid, out.as_map());
                SubtensorModule::update_voting_power_for_subnet(netuid, out.as_map());
            }
            for _ in 0..SAMPLES {
                let out = SubtensorModule::epoch_mechanism(netuid, MechId::MAIN, emission);
                let t0 = Instant::now();
                SubtensorModule::persist_mechanism_epoch_terms(netuid, MechId::MAIN, out.as_map());
                SubtensorModule::persist_netuid_epoch_terms(netuid, out.as_map());
                persist_ms += t0.elapsed().as_secs_f64() * 1_000.0;
                let t1 = Instant::now();
                SubtensorModule::update_voting_power_for_subnet(netuid, out.as_map());
                vote_ms += t1.elapsed().as_secs_f64() * 1_000.0;
            }
            persist_ms /= f64::from(SAMPLES);
            vote_ms /= f64::from(SAMPLES);
            let total = epoch_ms + persist_ms + vote_ms;
            println!(
                "{n},{keys_ms:.3},{last_ms:.3},{reg_ms:.3},{stake_ms:.3},{weight_ms:.3},{epoch_ms:.3},{persist_ms:.3},{vote_ms:.3},{total:.3}"
            );
        });
    }
}
