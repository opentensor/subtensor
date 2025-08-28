#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

// Run all tests
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::subsubnet --show-output

// Test plan:
//   - [x] Netuid index math (with SubsubnetCountCurrent limiting)
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
fn test_subsubnet_emission_proportions() {
    new_test_ext(1).execute_with(|| {});
}
