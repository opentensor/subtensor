#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

// Run all tests
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::subsubnet --show-output

// Test plan:
//   - [x] Netuid index math (with SubsubnetCountCurrent limiting)
//   - [x] Sub-subnet validity tests
//   - [x] do_set_desired tests
//   - [ ] Emissions are split proportionally
//   - [ ] Sum of split emissions is equal to rao_emission passed to epoch
//   - [ ] Weights can be set/commited/revealed by subsubnet
//   - [ ] Rate limiting is enforced by subsubnet
//   - [ ] Bonds are applied per subsubnet
//   - [ ] Incentives are per subsubnet
//   - [ ] Subsubnet limit can be set up to 8 (with admin pallet)
//   - [ ] When subsubnet limit is reduced, reduction is GlobalSubsubnetDecreasePerSuperblock per super-block
//   - [ ] When reduction of subsubnet limit occurs, Weights, Incentive, LastUpdate, Bonds, and WeightCommits are cleared
//   - [ ] Epoch terms of subnet are weighted sum (or logical OR) of all subsubnet epoch terms
//   - [ ] Subnet epoch terms persist in state
//   - [ ] Subsubnet epoch terms persist in state

use super::mock::*;
use crate::subnets::subsubnet::{GLOBAL_MAX_SUBNET_COUNT, MAX_SUBSUBNET_COUNT_PER_SUBNET};
use crate::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use sp_std::collections::vec_deque::VecDeque;
use subtensor_runtime_common::{NetUid, NetUidStorageIndex, SubId};

#[test]
fn test_index_from_netuid_and_subnet() {
    new_test_ext(1).execute_with(|| {
        [
            (0_u16, 0_u8),
            (GLOBAL_MAX_SUBNET_COUNT / 2, 1),
            (GLOBAL_MAX_SUBNET_COUNT / 2, 7),
            (GLOBAL_MAX_SUBNET_COUNT / 2, 14),
            (GLOBAL_MAX_SUBNET_COUNT / 2, 15),
            (GLOBAL_MAX_SUBNET_COUNT - 1, 1),
            (GLOBAL_MAX_SUBNET_COUNT - 1, 7),
            (GLOBAL_MAX_SUBNET_COUNT - 1, 14),
            (GLOBAL_MAX_SUBNET_COUNT - 1, 15),
        ]
        .iter()
        .for_each(|(netuid, sub_id)| {
            let idx = SubtensorModule::get_subsubnet_storage_index(
                NetUid::from(*netuid),
                SubId::from(*sub_id),
            );
            let expected = *sub_id as u64 * GLOBAL_MAX_SUBNET_COUNT as u64 + *netuid as u64;
            assert_eq!(idx, NetUidStorageIndex::from(expected as u16));
        });
    });
}

#[test]
fn test_netuid_and_subnet_from_index() {
    new_test_ext(1).execute_with(|| {
        [
            0_u16,
            1,
            14,
            15,
            16,
            17,
            GLOBAL_MAX_SUBNET_COUNT - 1,
            GLOBAL_MAX_SUBNET_COUNT,
            GLOBAL_MAX_SUBNET_COUNT + 1,
            0xFFFE / 2,
            0xFFFE,
            0xFFFF,
        ]
        .iter()
        .for_each(|netuid_index| {
            let expected_netuid = (*netuid_index as u64 % GLOBAL_MAX_SUBNET_COUNT as u64) as u16;
            let expected_subid = (*netuid_index as u64 / GLOBAL_MAX_SUBNET_COUNT as u64) as u8;

            // Allow subnet ID
            NetworksAdded::<Test>::insert(NetUid::from(expected_netuid), true);
            SubsubnetCountCurrent::<Test>::insert(
                NetUid::from(expected_netuid),
                SubId::from(expected_subid + 1),
            );

            let (netuid, subid) =
                SubtensorModule::get_netuid_and_subid(NetUidStorageIndex::from(*netuid_index))
                    .unwrap();
            assert_eq!(netuid, NetUid::from(expected_netuid as u16));
            assert_eq!(subid, SubId::from(expected_subid as u8));
        });
    });
}

#[test]
fn test_netuid_index_math_constants() {
    assert_eq!(
        GLOBAL_MAX_SUBNET_COUNT as u64 * MAX_SUBSUBNET_COUNT_PER_SUBNET as u64,
        0x10000
    );
}

#[test]
fn ensure_subsubnet_exists_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 3u16.into();
        let sub_id = SubId::from(1u8);

        // ensure base subnet exists
        NetworksAdded::<Test>::insert(NetUid::from(netuid), true);

        // Allow at least 2 sub-subnets (so sub_id = 1 is valid)
        SubsubnetCountCurrent::<Test>::insert(netuid, SubId::from(2u8));
        assert_ok!(SubtensorModule::ensure_subsubnet_exists(netuid, sub_id));
    });
}

#[test]
fn ensure_subsubnet_fails_when_base_subnet_missing() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 7u16.into();
        let sub_id = SubId::from(0u8);

        // Intentionally DO NOT create the base subnet

        assert_noop!(
            SubtensorModule::ensure_subsubnet_exists(netuid, sub_id),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn ensure_subsubnet_fails_when_subid_out_of_range() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 9u16.into();
        NetworksAdded::<Test>::insert(NetUid::from(netuid), true);

        // Current allowed sub-subnet count is 2 => valid sub_ids: {0, 1}
        SubsubnetCountCurrent::<Test>::insert(netuid, SubId::from(2u8));

        // sub_id == 2 is out of range (must be < 2)
        let sub_id_eq = SubId::from(2u8);
        assert_noop!(
            SubtensorModule::ensure_subsubnet_exists(netuid, sub_id_eq),
            Error::<Test>::SubNetworkDoesNotExist
        );

        // sub_id > 2 is also out of range
        let sub_id_gt = SubId::from(3u8);
        assert_noop!(
            SubtensorModule::ensure_subsubnet_exists(netuid, sub_id_gt),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn do_set_desired_subsubnet_count_ok_minimal() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(3u16);
        NetworksAdded::<Test>::insert(NetUid::from(3u16), true); // base subnet exists

        assert_ok!(SubtensorModule::do_set_desired_subsubnet_count(
            netuid,
            SubId::from(1u8)
        ));

        assert_eq!(SubsubnetCountDesired::<Test>::get(netuid), SubId::from(1u8));
    });
}

#[test]
fn do_set_desired_subsubnet_count_ok_at_effective_cap() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(4u16);
        NetworksAdded::<Test>::insert(NetUid::from(4u16), true); // base subnet exists

        // Effective bound is min(runtime cap, compile-time cap)
        let runtime_cap = MaxSubsubnetCount::<Test>::get(); // e.g., SubId::from(8)
        let compile_cap = SubId::from(MAX_SUBSUBNET_COUNT_PER_SUBNET);
        let bound = if runtime_cap <= compile_cap {
            runtime_cap
        } else {
            compile_cap
        };

        assert_ok!(SubtensorModule::do_set_desired_subsubnet_count(
            netuid, bound
        ));
        assert_eq!(SubsubnetCountDesired::<Test>::get(netuid), bound);
    });
}

#[test]
fn do_set_desired_fails_when_base_subnet_missing() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(7u16);
        // No NetworksAdded insert => base subnet absent

        assert_noop!(
            SubtensorModule::do_set_desired_subsubnet_count(netuid, SubId::from(1u8)),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn do_set_desired_fails_for_zero() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(9u16);
        NetworksAdded::<Test>::insert(NetUid::from(9u16), true); // base subnet exists

        assert_noop!(
            SubtensorModule::do_set_desired_subsubnet_count(netuid, SubId::from(0u8)),
            Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn do_set_desired_fails_when_over_runtime_cap() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(11u16);
        NetworksAdded::<Test>::insert(NetUid::from(11u16), true); // base subnet exists

        // Runtime cap is 8 (per function), so 9 must fail
        assert_noop!(
            SubtensorModule::do_set_desired_subsubnet_count(netuid, SubId::from(9u8)),
            Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn do_set_desired_fails_when_over_compile_time_cap() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(12u16);
        NetworksAdded::<Test>::insert(NetUid::from(12u16), true); // base subnet exists

        let too_big = SubId::from(MAX_SUBSUBNET_COUNT_PER_SUBNET + 1);
        assert_noop!(
            SubtensorModule::do_set_desired_subsubnet_count(netuid, too_big),
            Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn update_subsubnet_counts_decreases_and_cleans_on_superblock() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);

        // Base subnet exists
        let netuid = NetUid::from(42u16);
        NetworksAdded::<Test>::insert(NetUid::from(42u16), true);

        // super_block = SuperBlockTempos() * Tempo(netuid)
        Tempo::<Test>::insert(netuid, 1u16);
        let super_block =
            u64::from(SuperBlockTempos::<Test>::get()) * u64::from(Tempo::<Test>::get(netuid));

        // Choose counts so result is deterministic for ANY decrease-per-superblock.
        // Let dec = GlobalSubsubnetDecreasePerSuperblock(); set old = dec + 3.
        let dec: u8 = u8::from(GlobalSubsubnetDecreasePerSuperblock::<Test>::get());
        let old = SubId::from(dec.saturating_add(3)); // ≥3
        let desired = SubId::from(1u8);
        // min_possible = max(old - dec, 1) = 3 → new_count = 3
        SubsubnetCountCurrent::<Test>::insert(netuid, old);
        SubsubnetCountDesired::<Test>::insert(netuid, desired);

        // Seed data at a kept subid (2) and a removed subid (3)
        let idx_keep = SubtensorModule::get_subsubnet_storage_index(netuid, SubId::from(2u8));
        let idx_rm3 = SubtensorModule::get_subsubnet_storage_index(netuid, SubId::from(3u8));

        Weights::<Test>::insert(idx_keep, 0u16, vec![(1u16, 1u16)]);
        Incentive::<Test>::insert(idx_keep, vec![1u16]);
        LastUpdate::<Test>::insert(idx_keep, vec![123u64]);
        Bonds::<Test>::insert(idx_keep, 0u16, vec![(1u16, 2u16)]);
        WeightCommits::<Test>::insert(
            idx_keep,
            hotkey,
            VecDeque::from([(sp_core::H256::zero(), 1u64, 2u64, 3u64)]),
        );
        TimelockedWeightCommits::<Test>::insert(
            idx_keep,
            1u64,
            VecDeque::from([(hotkey, 1u64, Default::default(), Default::default())]),
        );

        Weights::<Test>::insert(idx_rm3, 0u16, vec![(9u16, 9u16)]);
        Incentive::<Test>::insert(idx_rm3, vec![9u16]);
        LastUpdate::<Test>::insert(idx_rm3, vec![999u64]);
        Bonds::<Test>::insert(idx_rm3, 0u16, vec![(9u16, 9u16)]);
        WeightCommits::<Test>::insert(
            idx_rm3,
            hotkey,
            VecDeque::from([(sp_core::H256::zero(), 1u64, 2u64, 3u64)]),
        );
        TimelockedWeightCommits::<Test>::insert(
            idx_rm3,
            1u64,
            VecDeque::from([(hotkey, 1u64, Default::default(), Default::default())]),
        );

        // Act exactly on a super-block boundary
        SubtensorModule::update_subsubnet_counts_if_needed(2 * super_block);

        // New count is 3
        assert_eq!(SubsubnetCountCurrent::<Test>::get(netuid), SubId::from(3u8));

        // Kept prefix intact
        assert_eq!(Incentive::<Test>::get(idx_keep), vec![1u16]);
        assert!(Weights::<Test>::iter_prefix(idx_keep).next().is_some());
        assert!(LastUpdate::<Test>::contains_key(idx_keep));
        assert!(Bonds::<Test>::iter_prefix(idx_keep).next().is_some());
        assert!(WeightCommits::<Test>::contains_key(idx_keep, hotkey));
        assert!(TimelockedWeightCommits::<Test>::contains_key(
            idx_keep, 1u64
        ));

        // Removed prefix (subid 3) cleared
        assert!(Weights::<Test>::iter_prefix(idx_rm3).next().is_none());
        assert_eq!(Incentive::<Test>::get(idx_rm3), Vec::<u16>::new());
        assert!(!LastUpdate::<Test>::contains_key(idx_rm3));
        assert!(Bonds::<Test>::iter_prefix(idx_rm3).next().is_none());
        assert!(!WeightCommits::<Test>::contains_key(idx_rm3, hotkey));
        assert!(!TimelockedWeightCommits::<Test>::contains_key(
            idx_rm3, 1u64
        ));
    });
}

#[test]
fn update_subsubnet_counts_no_change_when_not_superblock() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(100u16);
        NetworksAdded::<Test>::insert(NetUid::from(100u16), true);

        Tempo::<Test>::insert(netuid, 1u16);
        let super_block =
            u64::from(SuperBlockTempos::<Test>::get()) * u64::from(Tempo::<Test>::get(netuid));

        // Setup counts as in the previous test
        let dec: u8 = u8::from(GlobalSubsubnetDecreasePerSuperblock::<Test>::get());
        let old = SubId::from(dec.saturating_add(3));
        let desired = SubId::from(1u8);
        SubsubnetCountCurrent::<Test>::insert(netuid, old);
        SubsubnetCountDesired::<Test>::insert(netuid, desired);

        // Marker value at a subid that would be kept if a change happened
        let idx_mark = SubtensorModule::get_subsubnet_storage_index(netuid, SubId::from(2u8));
        Incentive::<Test>::insert(idx_mark, vec![77u16]);

        // Act on a non-boundary
        SubtensorModule::update_subsubnet_counts_if_needed(super_block - 1);

        // Nothing changes
        assert_eq!(SubsubnetCountCurrent::<Test>::get(netuid), old);
        assert_eq!(Incentive::<Test>::get(idx_mark), vec![77u16]);
    });
}

#[test]
fn test_subsubnet_emission_proportions() {
    new_test_ext(1).execute_with(|| {});
}
